use gl::types::*;
use log::error;

static mut GLOBAL_GL_CALL_COUNTER: u64 = 0u64;

/// Returns the corresponding error string for the given OpenGL error code
///
///* `error_code` - OpenGL error code
fn code_to_string(error_code: GLenum) -> &'static str {
    match error_code {
        gl::NO_ERROR => "no error",
        gl::INVALID_ENUM => "invalid enumerant",
        gl::INVALID_VALUE => "invalid value",
        gl::INVALID_OPERATION => "invalid operation",
        gl::STACK_OVERFLOW => "stack overflow",
        gl::STACK_UNDERFLOW => "stack underflow",
        //gl::TABLE_TOO_LARGE => "table too large",
        gl::OUT_OF_MEMORY => "out of memory",
        gl::INVALID_FRAMEBUFFER_OPERATION => "invalid framebuffer operation",
        _ => "unknown error code",
    }
}

/// Internal function for increasing the number of OpenGL calls.
/// Note: This function is not thread safe
#[inline]
fn increase_gl_call() {
    unsafe {
        GLOBAL_GL_CALL_COUNTER += 1;
    }
}

/// Returns the total number of OpenGL calls
pub fn get_number_of_gl_calls() -> u64 {
    unsafe { GLOBAL_GL_CALL_COUNTER }
}

/// Checks if the previous OpenGL function calls caused any errors
/// and write into the log if this is the case
///
///* `filename` - The source filename where the error was caused
///* `line` - The line in the source filename where the error was caused
///* `column` - The column in the source filename where the error was caused
pub fn check(filename: &str, line: u32, column: u32) {
    let error_code = unsafe { gl::GetError() };

    if error_code != gl::NO_ERROR {
        let error_msg = code_to_string(error_code);
        error!(
            "{} ({}:{}): Found OpenGL error '{}'",
            filename, line, column, error_msg
        );
    }
}

/// Internal use only function which performs the additional steps for an  OpenGL function call.
/// Returns the passed return value t.
///
///* `t` - The return value of the previously executed OpenGL function call
///* `filename` - The source filename where the function has been executed
///* `line` - The line in the source code file
///* `column` - Then column in the source code file
#[inline]
pub fn gl_call_helper<T>(t: T, filename: &str, line: u32, column: u32) -> T {
    check(filename, line, column);
    increase_gl_call();

    t
}

/// Encapsulates an OpenGL function call and performs internal checks and OpenGL call counting
macro_rules! gl_call {
    ($function_call:expr) => {
        $crate::gl_call::gl_call_helper(unsafe { $function_call }, file!(), line!(), column!())
    };
}
