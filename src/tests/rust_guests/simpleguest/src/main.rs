#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use hyperlight_guest::{
    flatbuffer_utils::get_flatbuffer_result_from_int,
    guest_functions::{create_function_definition, register_function},
};

extern crate hyperlight_guest;

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    // create fxn def
    let small_var_def = create_function_definition("smallVar", smallVar as i64, &[]);

    // register fxn def
    register_function(small_var_def);
}

#[no_mangle]
#[allow(improper_ctypes_definitions, non_camel_case_types)]
pub extern "C" fn smallVar() -> Vec<u8> {
    let _buffer: [u8; 1024] = [0; 1024];
    get_flatbuffer_result_from_int(1024)
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn guest_dispatch_function() -> Vec<u8> {
    [0; 0].to_vec()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
