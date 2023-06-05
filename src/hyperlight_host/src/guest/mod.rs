///! Represents a function call.
pub mod function_call;
///! Represents the result of a function call.
pub mod function_call_result;
///! Represents an error that occured int the guest.
pub mod guest_error;
///! Represents a function call from host to guest.
pub mod guest_function_call;
///! Represents guest log data
pub mod guest_log_data;
///! Represents a function call from guest to host.
pub mod host_function_call;
///! Represents the definition of a function that the host exposes to the guest.
pub mod host_function_definition;
///! Represents the functions that the host exposes to the guest.
pub mod host_function_details;
///! An enumeration and supporting logic to determine the desired
///! level of a log message issued from the guest.
pub mod log_level;