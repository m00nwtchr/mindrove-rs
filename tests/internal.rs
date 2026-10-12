// This integration test includes private implementation modules so it can test
// them directly; most included helpers are intentionally unused by this focused test.
#![allow(dead_code, unused_imports)]

use std::cell::Cell;
use std::ffi::CString;

pub use mindrove::{Error, Result};

// `internal/mod.rs` re-exports `crate::logging_bridge::ensure_logging_bridge`,
// so the integration test must provide a `logging_bridge` module to satisfy
// the `crate::` path resolution when it re-includes `internal/mod.rs`. The
// bridge is private and unused here; this stub exists purely to satisfy the
// path lookup and intentionally re-exports the symbol under the same name.
#[allow(dead_code)]
mod logging_bridge {
    pub(crate) fn ensure_logging_bridge() {}
}

#[path = "../src/internal/mod.rs"]
mod internal;

use internal::{
    array2_from_native, array3_from_native, check_status, checked_buffer_size,
    checked_element_count, checked_native_capacity, checked_native_len, read_i32_vec, read_string,
    to_cstring, to_json_cstring,
};

#[test]
fn status_helper_accepts_only_success() {
    assert!(check_status(0).is_ok());
    assert!(matches!(check_status(7), Err(Error::BoardNotReady)));
}

#[test]
fn shape_overflow_is_rejected() {
    assert!(matches!(
        checked_element_count(i32::MAX as usize, 2),
        Err(Error::IntegerOverflow)
    ));
}

#[test]
fn checked_capacity_rejects_values_that_do_not_fit_native_int() {
    assert!(matches!(
        checked_native_capacity(i32::MAX as usize + 1),
        Err(Error::IntegerOverflow)
    ));
}

#[test]
fn allocation_size_rejects_multiplication_and_addition_overflow() {
    assert!(matches!(
        checked_buffer_size(usize::MAX, 2, 1),
        Err(Error::IntegerOverflow)
    ));
}

#[test]
fn negative_native_length_is_rejected() {
    assert!(matches!(checked_native_len(-1), Err(Error::InvalidLength)));
}

#[test]
fn oversized_native_length_is_rejected() {
    assert!(matches!(
        checked_native_len((1 << 20) + 1),
        Err(Error::BufferContract(_))
    ));
}

#[test]
fn interior_nul_is_rejected() {
    assert!(matches!(to_cstring("bad\0value"), Err(Error::InteriorNul)));
}

#[test]
fn serialized_parameters_are_owned_c_strings() {
    let value = to_json_cstring(&serde_json::json!({"name": "board"})).unwrap();
    assert_eq!(value, CString::new(r#"{"name":"board"}"#).unwrap());
}

#[test]
fn string_fill_is_single_call_bounded_and_validates_utf8() {
    let calls = Cell::new(0);
    let value = read_string(|buffer, length| {
        calls.set(calls.get() + 1);
        assert!(!buffer.is_null());
        unsafe {
            assert_eq!(*length, 1 << 20);
            std::ptr::copy_nonoverlapping(b"hello".as_ptr().cast(), buffer, 5);
            *length = 5;
        }
        0
    })
    .unwrap();
    assert_eq!(value, "hello");
    assert_eq!(calls.get(), 1);

    let error = read_string(|buffer, length| {
        assert!(!buffer.is_null());
        unsafe {
            assert_eq!(*length, 1 << 20);
            std::ptr::copy_nonoverlapping([0xff_u8, 0xfe].as_ptr().cast(), buffer, 2);
            *length = 2;
        }
        0
    });
    assert!(matches!(error, Err(Error::InvalidUtf8)));
}

#[test]
fn string_fill_rejects_length_over_capacity() {
    let error = read_string(|buffer, length| {
        assert!(!buffer.is_null());
        unsafe {
            assert_eq!(*length, 1 << 20);
            *length = (1 << 20) + 1;
        }
        0
    });
    assert!(matches!(error, Err(Error::BufferContract(_))));
}

#[test]
fn integer_array_fill_is_single_call_and_bounded() {
    let value = read_i32_vec(|buffer, length| {
        assert!(!buffer.is_null());
        unsafe {
            assert_eq!(*length, (1 << 20) / std::mem::size_of::<i32>() as i32);
            *buffer.add(0) = 11;
            *buffer.add(1) = 22;
            *length = 2;
        }
        0
    })
    .unwrap();
    assert_eq!(value, vec![11, 22]);
}

#[test]
fn array_conversions_validate_shape_and_preserve_row_major_order() {
    let array = array2_from_native(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
    assert_eq!(array[[0, 0]], 1.0);
    assert_eq!(array[[1, 1]], 4.0);

    let array = array3_from_native(vec![1.0, 2.0, 3.0, 4.0], 1, 2, 2).unwrap();
    assert_eq!(array[[0, 1, 0]], 3.0);
    assert!(matches!(
        array2_from_native(vec![1.0, 2.0, 3.0], 2, 2),
        Err(Error::InvalidLength)
    ));
}

#[allow(dead_code)]
fn assert_result_type<T>(result: Result<T>) -> Result<T> {
    result
}
