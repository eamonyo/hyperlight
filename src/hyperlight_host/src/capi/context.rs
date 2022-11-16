use super::handle::{new_key, Handle, Key};
use super::hdl::Hdl;
#[cfg(target_os = "linux")]
use crate::capi::hyperv_linux::mshv_run_message;
#[cfg(target_os = "linux")]
use crate::hypervisor::kvm;
#[cfg(target_os = "linux")]
use crate::hypervisor::kvm_regs;
use crate::mem::{guest_mem::GuestMemory, pe::PEInfo};
use crate::sandbox::Sandbox;
use crate::{func::args::Val, mem::config::SandboxMemoryConfiguration};
use crate::{func::def::HostFunc, mem::layout::SandboxMemoryLayout};
use anyhow::{bail, Error, Result};
#[cfg(target_os = "linux")]
use kvm_ioctls::Kvm;
#[cfg(target_os = "linux")]
use mshv_bindings::mshv_user_mem_region;
#[cfg(target_os = "linux")]
use mshv_ioctls::{Mshv, VcpuFd, VmFd};
use std::collections::HashMap;

/// Context is a memory storage mechanism used in the Hyperlight C API
/// functions.
///
/// It is intended to be referred to by `Handle`s, which are passed
/// between C code and Rust implementation herein as the rough equivalent
/// of pointers.
///
/// Using this `Handle` and `Context` scheme to refer to allocated
/// memory provides a somewhat safer, though less efficient, way
/// to refer to memory on the heap than "raw" C pointers do.
///
/// # Safety
///
/// - Wherever a `Context` pointer is expected in a C function, you should
/// always pass a pointer returned to you by the `context_new` function,
/// that you have not modified in any way or passed to `context_free`.
/// - Functions that return a `Handle` often write new data into the
/// `Context`
/// - `Context` is not thread-safe. Do not share one between threads
#[derive(Default)]
pub struct Context {
    /// All `anyhow::Error`s stored in this context.
    pub errs: HashMap<Key, Error>,
    /// All `Sandbox`es stored in this context
    pub sandboxes: HashMap<Key, Sandbox>,
    /// All `Val`s stored in this context
    pub vals: HashMap<Key, Val>,
    /// All `HostFunc`s stored in this context
    pub host_funcs: HashMap<Key, HostFunc>,
    /// All `String`s stored in this context
    pub strings: HashMap<Key, String>,
    /// All `Vec<u8>`s stored in this context
    pub byte_arrays: HashMap<Key, Vec<u8>>,
    /// All `PEInfo`s stored in this context
    pub pe_infos: HashMap<Key, PEInfo>,
    /// All `SandboxMemoryConfiguration`s stored in this context
    pub mem_configs: HashMap<Key, SandboxMemoryConfiguration>,
    /// All `SandboxMemoryLayout`s stored in this context
    pub mem_layouts: HashMap<Key, SandboxMemoryLayout>,
    /// All `Mshv`s stored in this context
    #[cfg(target_os = "linux")]
    pub mshvs: HashMap<Key, Mshv>,
    /// All `VmFd`s stored in this context
    #[cfg(target_os = "linux")]
    pub vmfds: HashMap<Key, VmFd>,
    /// All `VcpuFd`s stored in this context
    #[cfg(target_os = "linux")]
    pub vcpufds: HashMap<Key, VcpuFd>,
    /// All `mshv_user_mem_region`s stored in this context
    #[cfg(target_os = "linux")]
    pub mshv_user_mem_regions: HashMap<Key, mshv_user_mem_region>,
    /// All `mshv_run_message`s stored in this context
    #[cfg(target_os = "linux")]
    pub mshv_run_messages: HashMap<Key, mshv_run_message>,
    /// All the `GuestMemory`s stored in this context
    pub guest_mems: HashMap<Key, GuestMemory>,
    /// All the `i64`s stored in this context
    pub int64s: HashMap<Key, i64>,
    /// All the `i32`s stored in this context
    pub int32s: HashMap<Key, i32>,
    #[cfg(target_os = "linux")]
    /// All the `kvm`s stored in this context
    pub kvms: HashMap<Key, Kvm>,
    #[cfg(target_os = "linux")]
    /// All the KVM `VmFd`s stored in this context
    pub kvm_vmfds: HashMap<Key, kvm_ioctls::VmFd>,
    #[cfg(target_os = "linux")]
    /// All the KVM `VcpuFd`s stored in this context
    pub kvm_vcpufds: HashMap<Key, kvm_ioctls::VcpuFd>,
    #[cfg(target_os = "linux")]
    /// All the KVM `kvm_userspace_memory_region`s stored in this context
    pub kvm_user_mem_regions: HashMap<Key, kvm_bindings::kvm_userspace_memory_region>,
    #[cfg(target_os = "linux")]
    /// All the KVM run results stored in this context
    pub kvm_run_messages: HashMap<Key, kvm::KvmRunMessage>,
    #[cfg(target_os = "linux")]
    /// All the KVM registers stored in this context
    pub kvm_regs: HashMap<Key, kvm_regs::Regs>,
    #[cfg(target_os = "linux")]
    /// All the KVM segment registers stored in this context
    pub kvm_sregs: HashMap<Key, kvm_regs::SRegs>,
}

impl Context {
    /// Create a new key and register the given `obj` in the given
    /// collection `coll`.
    ///
    /// The given `FnOnce` called `make_handle` can be used to
    /// create a new `Handle` from the newly created key, and to
    /// verify that the given `obj` is of the correct type.
    pub fn register<T, HandleFn: FnOnce(Key) -> Hdl>(
        obj: T,
        coll: &mut HashMap<Key, T>,
        make_handle: HandleFn,
    ) -> Handle {
        let key = new_key();
        let handle = Handle::from(make_handle(key));
        coll.insert(handle.key(), obj);
        handle
    }

    /// A convenience function for `register`, typed specifically
    /// for `Error` types.
    pub fn register_err(&mut self, err: Error) -> Handle {
        Self::register(err, &mut self.errs, Hdl::Err)
    }

    /// Convenience method for:
    /// ```
    /// self.register_err(anyhow::Error::msg(err_msg))
    /// ```
    pub fn register_err_msg(&mut self, err_msg: &str) -> Handle {
        self.register_err(anyhow::Error::msg(err_msg.to_string()))
    }

    /// Get a type `T` from the given collection `coll` using
    /// `handle.key()` as the index to `coll`.
    ///
    /// The `chk` function will be called with the `Hdl` created
    /// from the given `handle`, and if it returns `false`, an
    /// `Err` will be returned.
    ///
    /// This function is only suitable for immutable operations on
    /// `coll`. If you intend to mutate `coll`, use `get_mut`.
    pub fn get<T, ChkFn: FnOnce(&Hdl) -> bool>(
        handle: Handle,
        coll: &HashMap<Key, T>,
        chk: ChkFn,
    ) -> Result<&T> {
        let hdl = Hdl::try_from(handle)?;
        if !chk(&hdl) {
            bail!("invalid handle")
        }
        match coll.get(&handle.key()) {
            Some(obj) => Ok(obj),
            None => bail!("object {} not found for key {}", hdl, handle.key()),
        }
    }

    /// Similar to `get`, except returns a `WriteResult` rather than
    /// a `ReadResult`, making this function suitable for mutating
    /// `coll` in a thread-safe manner.
    pub fn get_mut<T, ChkFn: FnOnce(&Hdl) -> bool>(
        handle: Handle,
        coll: &mut HashMap<Key, T>,
        chk: ChkFn,
    ) -> Result<&mut T> {
        let hdl = Hdl::try_from(handle)?;
        if !chk(&hdl) {
            bail!("invalid handle")
        }
        match coll.get_mut(&handle.key()) {
            Some(obj) => Ok(obj),
            None => bail!("object {} not found for key {}", hdl, handle.key()),
        }
    }

    /// Convert the given `Handle` parameter to a `Hdl` type (returning
    /// an `Err` if the conversion fails), then call `chk_fn` and
    /// immediately return an `Err` if it returns `false`, and finally
    /// remove that `Hdl`'s key from the collection that corresponds to
    /// it, returning `true` if an element was removed and `false`
    /// otherwise.
    pub fn remove<ChkFn: FnOnce(&Hdl) -> bool>(&mut self, handle: Handle, chk_fn: ChkFn) -> bool {
        match Hdl::try_from(handle) {
            Ok(hdl) => {
                if !chk_fn(&hdl) {
                    return false;
                }
                match hdl {
                    Hdl::Err(key) => self.errs.remove(&key).is_some(),
                    Hdl::Sandbox(key) => self.sandboxes.remove(&key).is_some(),
                    Hdl::Empty() => true,
                    Hdl::Val(key) => self.vals.remove(&key).is_some(),
                    Hdl::HostFunc(key) => self.host_funcs.remove(&key).is_some(),
                    Hdl::String(key) => self.strings.remove(&key).is_some(),
                    Hdl::ByteArray(key) => self.byte_arrays.remove(&key).is_some(),
                    Hdl::PEInfo(key) => self.pe_infos.remove(&key).is_some(),
                    Hdl::MemConfig(key) => self.mem_configs.remove(&key).is_some(),
                    Hdl::MemLayout(key) => self.mem_layouts.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::Mshv(key) => self.mshvs.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::VmFd(key) => self.vmfds.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::VcpuFd(key) => self.vcpufds.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::MshvUserMemRegion(key) => {
                        self.mshv_user_mem_regions.remove(&key).is_some()
                    }
                    #[cfg(target_os = "linux")]
                    Hdl::MshvRunMessage(key) => self.mshv_run_messages.remove(&key).is_some(),
                    Hdl::GuestMemory(key) => self.guest_mems.remove(&key).is_some(),
                    Hdl::Int64(key) => self.int64s.remove(&key).is_some(),
                    Hdl::Int32(key) => self.int32s.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::Kvm(key) => self.kvms.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::KvmVmFd(key) => self.kvm_vmfds.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::KvmVcpuFd(key) => self.kvm_vcpufds.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::KvmUserMemRegion(key) => self.kvm_user_mem_regions.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::KvmRunMessage(key) => self.kvm_run_messages.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::KvmRegisters(key) => self.kvm_regs.remove(&key).is_some(),
                    #[cfg(target_os = "linux")]
                    Hdl::KvmSRegisters(key) => self.kvm_sregs.remove(&key).is_some(),
                }
            }
            Err(_) => false,
        }
    }
}

/// Create a new context for use in the C API.
#[no_mangle]
pub extern "C" fn context_new() -> *mut Context {
    Box::into_raw(Box::new(Context::default()))
}

/// Free the memory referenced by with `ctx`.
///
/// # Safety
///
/// You must only call this function:
///
/// - Exactly once per `ctx` parameter
/// - Only after a given `ctx` is done being used
/// - With `Context`s created by `context_new`
#[no_mangle]
pub unsafe extern "C" fn context_free(ctx: *mut Context) {
    drop(Box::from_raw(ctx))
}

#[cfg(test)]
mod tests {
    use super::Context;
    use crate::capi::byte_array::get_byte_array_mut;
    use crate::capi::hdl::Hdl;
    use crate::capi::strings::get_string;
    use crate::capi::val_ref::get_val;
    use crate::func::args::Val;
    use crate::func::SerializationType;
    use anyhow::Result;

    #[test]
    fn round_trip_string() -> Result<()> {
        let mut ctx = Context::default();
        let start = "hello".to_string();
        let hdl_res = Context::register(start, &mut ctx.strings, Hdl::String);
        Context::get(hdl_res, &ctx.strings, |s| matches!(s, Hdl::String(_)))?;
        Ok(())
    }

    #[test]
    fn round_trip_val() -> Result<()> {
        let mut ctx = Context::default();
        let start = Val::new(Vec::new(), SerializationType::Raw);
        let start_clone = start.clone();
        let hdl_res = Context::register(start, &mut ctx.vals, Hdl::Val);
        get_val(&ctx, hdl_res).map(|f| assert_eq!(*f, start_clone))
    }

    #[test]
    fn round_trip_byte_array() -> Result<()> {
        let mut ctx = Context::default();
        let start = vec![1, 2, 3, 4, 5];
        let start_clone = start.clone();
        let hdl_res = Context::register(start, &mut ctx.byte_arrays, Hdl::ByteArray);
        get_byte_array_mut(&mut ctx, hdl_res).map(|b| assert_eq!(**b, start_clone))
    }

    #[test]
    fn remove_handle() -> Result<()> {
        let mut ctx = Context::default();
        let hdl = Context::register("hello".to_string(), &mut ctx.strings, Hdl::String);
        ctx.remove(hdl, |h| matches!(h, Hdl::String(_)));
        assert!(get_string(&ctx, hdl).is_err());
        Ok(())
    }
}
