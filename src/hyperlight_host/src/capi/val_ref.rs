use super::context::Context;
use super::handle::Handle;
use super::hdl::Hdl;
use crate::func::args::Val;
use crate::func::SerializationType;
use crate::validate_context;
use crate::validate_context_or_panic;
use anyhow::bail;
use anyhow::Result;
use std::boxed::Box;

mod impls {
    use super::super::handle::Handle;
    use crate::capi::{arrays::raw_vec::RawVec, context::Context};
    use crate::func::args::Val;
    use crate::func::SerializationType;

    pub unsafe fn val_ref_new(
        arr_ptr: *const i8,
        arr_len: usize,
        ser_type: SerializationType,
    ) -> Box<Val> {
        let raw_vec = RawVec::copy_from_ptr(arr_ptr as *mut i8, arr_len);
        Box::new(Val::new(raw_vec.into(), ser_type))
    }

    pub fn val_ref_get(ctx: &Context, val_hdl: Handle) -> Box<Val> {
        match super::get_val(ctx, val_hdl) {
            Ok(val) => Box::new((*val).clone()),
            Err(_) => Box::new(Val::new(Vec::new(), SerializationType::Raw)),
        }
    }
}

/// Get a read-only reference to a value from `ctx` that is
/// referenced by `val_hdl`.
pub fn get_val(ctx: &Context, val_hdl: Handle) -> Result<&Val> {
    match Hdl::try_from(val_hdl) {
        Ok(Hdl::Val(_)) => {}
        _ => bail!("Handle is not a Val"),
    }
    Context::get(val_hdl, &ctx.vals, |v| matches!(v, Hdl::Val(_)))
}

/// Convert a `(i8*, usize)` (indicating a raw C array) and
/// `SerializationType` into a `Val` and return a `Handle`
/// referencing that `Val`.
///
/// # Safety
///
/// `arr_ptr` must point to the start of a memory block you own
/// that can hold `arr_len` `i8` values. This memory must exist
/// for the entire duration of this function's execution.
///
/// The memory is borrowed by this function and will be copied
/// internally. It is the caller's responsibility to delete
/// both the returned `Val*` and the memory referenced by `arr_ptr`
/// when they are done with it.
#[no_mangle]
pub unsafe extern "C" fn val_ref_new(
    arr_ptr: *const i8,
    arr_len: usize,
    ser_type: SerializationType,
) -> *mut Val {
    Box::into_raw(impls::val_ref_new(arr_ptr, arr_len, ser_type))
}

/// Deep-compare the values referenced by `val1_hdl` and `val2_hdl`.
/// Return `true` if they both are valid references and are equal,
/// and `false` otherwise.
///
/// # Safety
///
/// `val1_hdl` and `val2_hdl` must be valid references created with
/// `val_ref_new` and not modified or deleted in any way while this
/// function is executing.
#[no_mangle]
pub unsafe extern "C" fn val_refs_compare(val1: *const Val, val2: *const Val) -> bool {
    *val1 == *val2
}

/// Free the memory associated with the given `Val`
///
/// # Safety
///
/// `v` must be a reference to memory the caller owns that was created
/// with `val_ref_new`. After calling this function, the given reference
/// is no longer valid and must not be used for any purpose.
#[no_mangle]
pub unsafe extern "C" fn val_ref_free(v: *mut Val) {
    drop(Box::from_raw(v));
}

/// Return the `Val` associated with `val_hdl`, if one exists, and
/// an empty `Val` otherwise.
///
/// # Safety
///
/// `ctx` must be a valid `Context` created by `context_new`, owned by the
/// caller, and not deleted or modified in any way while this function is
/// executing.
///
/// The return `Val` is a reference to new memory that you own. Make sure
/// you call `val_ref_free` exactly once when you're done with it.
#[no_mangle]
pub unsafe extern "C" fn val_ref_get(ctx: *const Context, val_hdl: Handle) -> *mut Val {
    validate_context_or_panic!(ctx);

    let ctx_ref = &*ctx;
    let bx = impls::val_ref_get(ctx_ref, val_hdl);
    Box::into_raw(bx)
}

/// Copy `val`, register the copy with `ctx`, and return the `Handle` associated
/// with the newly registered `Val`.
///
/// # Safety
///
/// `ctx` and `val` must be valid references created by `context_new` and
/// `val_ref_new` respectively. They must be owned by the caller
/// and not modified or deleted in any way while this function is executing.
///
/// `val` is copied internally, so it's the caller's responsibility
/// to delete `val` with `val_ref_free` after they are done with it,
/// and no earlier than when this function returns.
#[no_mangle]
pub unsafe extern "C" fn val_ref_register(ctx: *mut Context, val: *const Val) -> Handle {
    validate_context!(ctx);

    let ctx_ref = &mut *ctx;
    let val_ref = &*val;
    Context::register(val_ref.clone(), &mut ctx_ref.vals, Hdl::Val)
}
