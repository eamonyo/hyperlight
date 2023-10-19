#[cfg(target_os = "windows")]
use crate::hypervisor::handlers::{OutBHandlerCaller, OutBHandlerWrapper};
#[cfg(target_os = "windows")]
use crate::mem::custom_drop::CustomPtrDrop;
#[cfg(target_os = "windows")]
use crate::mem::mgr::SandboxMemoryManager;
#[cfg(target_os = "windows")]
use crate::Result;
#[cfg(target_os = "linux")]
use std::marker::PhantomData;
#[cfg(target_os = "windows")]
use std::{
    os::raw::c_void,
    sync::{Arc, Mutex},
};

#[cfg(target_os = "windows")]
/// This function allows us to call the OutBHandler from the guest when running
/// in process.
///
/// NOTE: This is not part of the C Hyperlight API , it is intended only to be
/// called in proc through a pointer passed to the guest.
extern "C" fn call_outb(ptr: *mut Arc<Mutex<dyn OutBHandlerCaller>>, port: u16, data: u64) {
    let outb_handlercaller = unsafe { Box::from_raw(ptr) };
    let res = outb_handlercaller
        .lock()
        .expect("Error Locking")
        .call(port, data);
    assert!(res.is_ok());
    // Leak the box so that it is not dropped when the function returns
    // the box will be dropped when the sandbox is dropped
    Box::leak(outb_handlercaller);
}

/// A container to store and safely drop leaked outb handlers when executing
/// with in-process mode on windows.
///
///
/// # Explanation of why we need to leak with this struct
///
/// We need to leak the outb handler for in-process mode because, for this
/// execution mode, we need to write the address of an in-memory closure
/// (e.g. a `FnMut`) to memory, so the guest binary (which, again, is executing
/// in memory rather than in a hypervisor) can look up that address and make
/// function calls to the host.
///
/// In this setup, however, Rust will drop the outb function before the guest
/// can make these calls, thus resulting in invalid memory accesses
/// (e.g. segmentation faults or whatever your favorite platform calls invalid
///  accesses). Thus, we need to leak the outb handler so it doesn't get
/// dropped before it's used.
///
/// This struct also ensures that, when _it_ gets dropped -- which is later
/// than the contained `FnMut` would have been -- it properly cleans up the
/// the previously-leaked memory.
///
/// # Note for in-hypervisor mode or Linux
///
/// If not executing with in-process mode, or not on windows, this struct
/// has no functionality. It's purposely available on windows and linux,
/// however, to ease internal implementation of the evolve methods.
///
/// # Final note
///
/// In the future, this struct _should_ be behind a compile flag, as described
/// in https://github.com/deislabs/hyperlight/issues/533.
pub(super) struct LeakedOutBWrapper<'a> {
    #[cfg(target_os = "windows")]
    hdl_ptr: Arc<Mutex<CustomPtrDrop<'a, OutBHandlerWrapper<'a>>>>,
    /// This `PhantomData` will never be used, since it's impossible to
    /// actually create a `LeakedOutBWrapper` on Linux. It is only in place
    /// to prevent clippy from complaining that the lifetime parameter
    /// in this struct is unused
    #[cfg(target_os = "linux")]
    p: PhantomData<&'a u64>,
}

#[cfg(target_os = "windows")]
impl<'a> LeakedOutBWrapper<'a> {
    pub(super) fn new(
        mgr: &mut SandboxMemoryManager,
        wrapper: OutBHandlerWrapper<'a>,
    ) -> Result<Self> {
        let hdl_box = Box::new(wrapper.clone());
        let hdl_ptr = Box::into_raw(hdl_box);
        let cd = CustomPtrDrop::new(
            hdl_ptr,
            Box::new(|ptr| {
                unsafe { Box::from_raw(ptr) };
            }),
        );
        let res = Self {
            hdl_ptr: Arc::new(Mutex::new(cd)),
        };

        let addr = res.hdl_wrapper_addr()?;
        mgr.set_outb_address_and_context(Self::outb_addr(), addr)?;
        Ok(res)
    }

    /// Get the address to the internally-stored `OutBHandlerWrapper`.
    ///
    /// This pointer is referred to by the `SandboxMemoryManager` as
    /// the outb "context"
    pub(super) fn hdl_wrapper_addr(&self) -> Result<u64> {
        let ptr = self.hdl_ptr.lock()?;
        Ok(ptr.as_mut_ptr() as u64)
    }

    pub(super) fn outb_addr() -> u64 {
        call_outb as *const c_void as u64
    }
}