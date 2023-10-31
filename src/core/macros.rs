//
// Conversion helpers
//

// from:  name: &str   to:  *const c_char
//
// example calling gl function:
//
//  GLint foo(name: *const GLchar);
//
// use:
//
//    let c_str = c_string!("thingy");
//    foo(c_str.as_ptr());
//
#[macro_export]
macro_rules! c_string {
    ($a_string:expr) => {
        std::ffi::CString::new($a_string).unwrap()
    };
}

#[macro_export]
macro_rules! size_of_floats {
    ($value:expr) => {
        ($value * mem::size_of::<f32>())
    };
}

#[macro_export]
macro_rules! size_of_uint {
    ($value:expr) => {
        ($value * mem::size_of::<u32>())
    };
}

#[macro_export]
macro_rules! gl_get_uniform_location {
    ($id: expr, $value:expr) => {{
        let c_str = std::ffi::CString::new($value).unwrap();
        gl::GetUniformLocation($id, c_str.as_ptr())
    }};
}
