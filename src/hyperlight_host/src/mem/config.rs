use std::cmp::max;

#[derive(Copy, Clone, Debug)]
pub struct SandboxMemoryConfiguration {
    /// The maximum size of the guest error message field.
    pub guest_error_message_size: usize,
    /// The size of the memory buffer that is made available for Guest Function Definitions
    pub host_function_definition_size: usize,
    /// The size of the memory buffer that is made available for serialising Host Exceptions
    pub host_exception_size: usize,
    /// The size of the memory buffer that is made available for input to the Guest Binary
    pub input_data_size: usize,
    /// The size of the memory buffer that is made available for input to the Guest Binary
    pub output_data_size: usize,
}
impl SandboxMemoryConfiguration {
    pub const DEFAULT_INPUT_SIZE: usize = 0x4000;
    // const DEFAULT_OUTPUT_SIZE: usize = 0x4000;
    // const DEFAULT_HOST_FUNCTION_DEFINITION_SIZE: usize = 0x1000;
    // const DEFAULT_HOST_EXCEPTION_SIZE: usize = 0x1000;
    // const DEFAULT_GUEST_ERROR_MESSAGE_SIZE: usize = 0x100;
    const MIN_INPUT_SIZE: usize = 0x2000;
    const MIN_OUTPUT_SIZE: usize = 0x2000;
    const MIN_HOST_FUNCTION_DEFINITION_SIZE: usize = 0x400;
    const MIN_HOST_EXCEPTION_SIZE: usize = 0x400;
    const MIN_GUEST_ERROR_MESSAGE_SIZE: usize = 0x80;

    pub fn new(
        input_data_size: usize,
        output_data_size: usize,
        function_definition_size: usize,
        host_exception_size: usize,
        guest_error_message_size: usize,
    ) -> Self {
        Self {
            input_data_size: max(input_data_size, Self::MIN_INPUT_SIZE),
            output_data_size: max(output_data_size, Self::MIN_OUTPUT_SIZE),
            host_function_definition_size: max(
                function_definition_size,
                Self::MIN_HOST_FUNCTION_DEFINITION_SIZE,
            ),
            host_exception_size: max(host_exception_size, Self::MIN_HOST_EXCEPTION_SIZE),
            guest_error_message_size: max(
                guest_error_message_size,
                Self::MIN_GUEST_ERROR_MESSAGE_SIZE,
            ),
        }
    }
}