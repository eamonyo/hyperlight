use super::context::Context;
use super::handle::Handle;
use super::hdl::Hdl;
use super::mem_access_handler::get_mem_access_handler_func;
use super::mem_access_handler::MemAccessHandlerWrapper;
use super::outb_handler::get_outb_handler_func;
use crate::validate_context;
use crate::{c_func::CFunc, outb_handler::OutBHandlerWrapper};
use anyhow::Result;
use hyperlight_host::hypervisor::hyperv_linux::{is_hypervisor_present, HypervLinuxDriver};
use hyperlight_host::hypervisor::hypervisor_mem::HypervisorAddrs;
use hyperlight_host::hypervisor::Hypervisor;
use hyperlight_host::mem::ptr::{GuestPtr, RawPtr};
use mshv_bindings::hv_register_name_HV_X64_REGISTER_RSP;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

fn get_driver_mut(ctx: &mut Context, hdl: Handle) -> Result<&mut HypervLinuxDriver> {
    Context::get_mut(hdl, &mut ctx.hyperv_linux_drivers, |h| {
        matches!(h, Hdl::HypervLinuxDriver(_))
    })
}

fn get_driver(ctx: &Context, hdl: Handle) -> Result<&HypervLinuxDriver> {
    Context::get(hdl, &ctx.hyperv_linux_drivers, |h| {
        matches!(h, Hdl::HypervLinuxDriver(_))
    })
}

/// Returns a bool indicating if hyperv is present on the machine
/// Takes an argument to indicate if the hypervisor api must be stable
/// If the hypervisor api is not stable, the function will return false even if the hypervisor is present
///
/// # Examples
///
/// ```
/// use hyperlight_host::capi::hyperv_linux::is_hyperv_linux_present;
///
/// assert_eq!(is_hyperv_linux_present(require_stable_api), true );
/// ```
#[no_mangle]
pub extern "C" fn is_hyperv_linux_present() -> bool {
    // At this point we dont have any way to report the error if one occurs.
    is_hypervisor_present().unwrap_or(false)
}

/// Creates a new HyperV-Linux driver with the given parameters and
/// "advanced" registers, suitable for a guest program that access
/// memory.
///
/// If the driver was created successfully, returns a `Handle` referencing the
/// new driver. Otherwise, returns a new `Handle` that references a descriptive
/// error.
///
/// # Safety
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn hyperv_linux_create_driver(
    ctx: *mut Context,
    addrs: HypervisorAddrs,
    rsp: u64,
    pml4: u64,
) -> Handle {
    CFunc::new("hyperv_linux_create_driver", ctx)
        .and_then_mut(|ctx, _| {
            let rsp_ptr = GuestPtr::try_from(RawPtr::from(rsp))?;
            let pml4_ptr = GuestPtr::try_from(RawPtr::from(pml4))?;

            let driver = HypervLinuxDriver::new(&addrs, rsp_ptr, pml4_ptr)?;
            Ok(Context::register(
                driver,
                &mut ctx.hyperv_linux_drivers,
                Hdl::HypervLinuxDriver,
            ))
        })
        .ok_or_err_hdl()
}

/// Apply all drivers to the vCPU stored within the HypervLinuxDriver
/// referenced by `driver_hdl` that were previously added but not already
/// set.
///
/// Some functions will do this for you, and thus if you use one of those
/// you won't need to call this. See the below list for details.
///
/// - `hyperv_linux_execute_until_halt`: does not call this function for you.
/// Call this function prior to calling that one.
/// - `hyperv_linux_initialise`: calls this function for you. Calling it again
/// is a no-op.
/// - `hyperv_linux_dispatch_call_from_host`: calls this function for you.
/// Calling it again is a no-op.
///
/// # Safety
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn hyperv_linux_apply_registers(
    ctx_ptr: *mut Context,
    driver_hdl: Handle,
) -> Handle {
    validate_context!(ctx_ptr);

    let res = {
        let ctx = &*ctx_ptr;
        get_driver(ctx, driver_hdl).and_then(|driver| driver.apply_registers())
    };
    match res {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx_ptr).register_err(e),
    }
}

/// Set, but do not apply, the stack pointer register.
///
/// # Safety
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn hyperv_linux_set_rsp(
    ctx_ptr: *mut Context,
    driver_hdl: Handle,
    rsp_val: u64,
) -> Handle {
    validate_context!(ctx_ptr);
    let driver = match get_driver_mut(&mut *ctx_ptr, driver_hdl) {
        Ok(d) => d,
        Err(e) => return (*ctx_ptr).register_err(e),
    };
    match driver.update_register_u64(hv_register_name_HV_X64_REGISTER_RSP, rsp_val) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx_ptr).register_err(e),
    }
}

pub(crate) fn get_handler_funcs(
    ctx: &Context,
    outb_func_hdl: Handle,
    mem_access_func_hdl: Handle,
) -> Result<(OutBHandlerWrapper, MemAccessHandlerWrapper)> {
    let outb_func = get_outb_handler_func(ctx, outb_func_hdl).map(|f| (*f).clone())?;
    let mem_access_func =
        get_mem_access_handler_func(ctx, mem_access_func_hdl).map(|f| (*f).clone())?;
    Ok((outb_func, mem_access_func))
}

/// Initialise the vCPU, call the equivalent of `execute_until_halt`,
/// and return the result.
///
/// Return an empty `Handle` on success, or a `Handle` that references a
/// descriptive error on failure.
///
/// # Safety
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn hyperv_linux_initialise(
    ctx: *mut Context,
    driver_hdl: Handle,
    outb_func_hdl: Handle,
    mem_access_func_hdl: Handle,
    peb_addr: u64,
    seed: u64,
    page_size: u32,
) -> Handle {
    validate_context!(ctx);
    let driver = match get_driver_mut(&mut *ctx, driver_hdl) {
        Ok(d) => d,
        Err(e) => return (*ctx).register_err(e),
    };
    let (outb_func, mem_access_func) =
        match get_handler_funcs(&*ctx, outb_func_hdl, mem_access_func_hdl) {
            Ok(tup) => tup,
            Err(e) => return (*ctx).register_err(e),
        };
    let init_res = (*driver).initialise(
        peb_addr.into(),
        seed,
        page_size,
        Arc::new(Mutex::new(outb_func)),
        Arc::new(Mutex::new(mem_access_func)),
        // These are set to the defaults in SandboxConfiguration, once we migrate the C# Sandbox to use the Rust Sandbox this API should be deleted so defaulting these for now should be fine
        Duration::from_millis(1000),
        Duration::from_millis(10),
    );
    match init_res {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Execute the virtual CPU stored inside the HyperV Linux driver referenced
/// by `driver_hdl` until a HLT instruction is reached. You likely should
/// call `hyperv_linux_initialise` instead of this function.
///
/// Return an empty `Handle` on success, or a `Handle` that references a
/// descriptive error on failure.
///
/// # Safety
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free
/// - Not modified, except by calling functions in the Hyperlight C API
#[no_mangle]
pub unsafe extern "C" fn hyperv_linux_execute_until_halt(
    ctx: *mut Context,
    driver_hdl: Handle,
    outb_func_hdl: Handle,
    mem_access_func_hdl: Handle,
) -> Handle {
    validate_context!(ctx);
    let driver = match get_driver_mut(&mut *ctx, driver_hdl) {
        Ok(d) => d,
        Err(e) => return (*ctx).register_err(e),
    };
    let (outb_func, mem_access_func) =
        match get_handler_funcs(&*ctx, outb_func_hdl, mem_access_func_hdl) {
            Ok(tup) => tup,
            Err(e) => return (*ctx).register_err(e),
        };
    match (*driver).execute_until_halt(
        Arc::new(Mutex::new(outb_func)),
        Arc::new(Mutex::new(mem_access_func)),
        // These are set to the defaults in SandboxConfiguration, once we migrate the C# Sandbox to use the Rust Sandbox this API should be deleted so defaulting these for now should be fine
        Duration::from_millis(1000),
        Duration::from_millis(10),
    ) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}

/// Dispatch a call from the host to the guest, using the function
/// referenced by `dispatch_func_addr`
///
/// # Safety
/// You must call this function with a `Context*` that has been:
///
/// - Created with `context_new`
/// - Not yet freed with `context_free
/// - Not modified, except by calling functions in the Hyperlight C API
///
#[no_mangle]
pub unsafe extern "C" fn hyperv_linux_dispatch_call_from_host(
    ctx: *mut Context,
    driver_hdl: Handle,
    outb_func_hdl: Handle,
    mem_access_func_hdl: Handle,
    dispatch_func_addr: u64,
) -> Handle {
    validate_context!(ctx);
    let driver = match get_driver_mut(&mut *ctx, driver_hdl) {
        Ok(d) => d,
        Err(e) => return (*ctx).register_err(e),
    };
    let (outb_func, mem_access_func) =
        match get_handler_funcs(&*ctx, outb_func_hdl, mem_access_func_hdl) {
            Ok(tup) => tup,
            Err(e) => return (*ctx).register_err(e),
        };
    match (*driver).dispatch_call_from_host(
        dispatch_func_addr.into(),
        Arc::new(Mutex::new(outb_func)),
        Arc::new(Mutex::new(mem_access_func)),
        // These are set to the defaults in SandboxConfiguration, once we migrate the C# Sandbox to use the Rust Sandbox this API should be deleted so defaulting these for now should be fine
        Duration::from_millis(1000),
        Duration::from_millis(10),
    ) {
        Ok(_) => Handle::new_empty(),
        Err(e) => (*ctx).register_err(e),
    }
}
