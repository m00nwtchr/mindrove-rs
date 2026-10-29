use std::ffi::{c_char, c_int};

pub(crate) fn prepare(params: *const c_char) -> c_int {
    unsafe { mindrove_sys::prepare(params) }
}

pub(crate) fn predict(
    data: *mut f64,
    data_len: c_int,
    output: *mut f64,
    output_len: *mut c_int,
    params: *const c_char,
) -> c_int {
    unsafe { mindrove_sys::predict(data, data_len, output, output_len, params) }
}

pub(crate) fn release(params: *const c_char) -> c_int {
    unsafe { mindrove_sys::release(params) }
}

pub(crate) fn release_all() -> c_int {
    unsafe { mindrove_sys::release_all() }
}

pub(crate) fn version(buffer: *mut c_char, len: *mut c_int, max_len: c_int) -> c_int {
    unsafe { mindrove_sys::get_version_ml_module(buffer, len, max_len) }
}
