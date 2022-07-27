use super::args::{Val, ValType};
use std::fmt::Debug;
use std::ops::Fn;
use std::rc::Rc;
use std::string::ToString;
use std::vec::Vec;

/// HostFunc is the definition of a function implemented
/// by the host but callable by the guest. Internally it
/// is essentially a function closure, but it has additional
/// functionality attached to it.
#[derive(Clone)]
pub struct HostFunc {
    pub func: Rc<Box<dyn Fn(&Val) -> Box<Val>>>,
}

impl HostFunc {
    pub fn new(func: Box<dyn Fn(&Val) -> Box<Val>>) -> Self {
        Self {
            func: Rc::new(func),
        }
    }

    pub fn call(&self, val: &Val) -> Box<Val> {
        (*self.func)(val)
    }
}

/// The definition of a function to be called on the guest.
/// Use the `call` method on this struct's implementation
/// to execute the call.
#[derive(Debug)]
pub struct GuestFunc {
    pub name: String,
    pub args: Vec<ValType>,
}

impl GuestFunc {
    /// Call this function with the specified args.
    /// This is intended to be a low-level function
    /// on top of which there will be a Rust convenience
    /// wrapper.
    pub fn call(&self, args: &Val) -> Result<Val, FuncCallError> {
        println!("ABOUT TO CALL: {:?}({:?})", self.name, args);
        // TODO: implement
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct FuncCallError {
    pub message: String,
}

impl ToString for FuncCallError {
    fn to_string(&self) -> String {
        self.message.to_string()
    }
}
