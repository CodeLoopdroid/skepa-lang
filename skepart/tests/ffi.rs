use std::ffi::c_void;

use skepart::{RtFunctionRef, RtValue};

unsafe extern "C" {
    fn skp_rt_string_from_utf8(data: *const u8, len: i64) -> *mut c_void;
    fn skp_rt_builtin_str_len(value: *mut c_void) -> i64;
    fn skp_rt_value_from_int(value: i64) -> *mut c_void;
    fn skp_rt_value_from_unit() -> *mut c_void;
    fn skp_rt_value_to_int(value: *mut c_void) -> i64;
    fn skp_rt_value_from_function(value: i32) -> *mut c_void;
    fn skp_rt_value_to_function(value: *mut c_void) -> i32;
    fn skp_rt_array_repeat(value: *mut c_void, size: i64) -> *mut c_void;
    fn skp_rt_array_get(array: *mut c_void, index: i64) -> *mut c_void;
    fn skp_rt_vec_new() -> *mut c_void;
    fn skp_rt_vec_push(vec: *mut c_void, value: *mut c_void);
    fn skp_rt_vec_get(vec: *mut c_void, index: i64) -> *mut c_void;
}

#[test]
fn ffi_string_and_value_roundtrip_surfaces_work() {
    let bytes = "🙂ok".as_bytes();
    let string_ptr = unsafe { skp_rt_string_from_utf8(bytes.as_ptr(), bytes.len() as i64) };
    assert_eq!(unsafe { skp_rt_builtin_str_len(string_ptr) }, 3);

    let int_ptr = unsafe { skp_rt_value_from_int(42) };
    assert_eq!(unsafe { skp_rt_value_to_int(int_ptr) }, 42);

    let unit_ptr = unsafe { skp_rt_value_from_unit() };
    let unit = unsafe { (*(unit_ptr as *mut RtValue)).clone() };
    assert!(matches!(unit, RtValue::Unit));
}

#[test]
fn ffi_function_and_container_surfaces_work() {
    let fn_ptr = unsafe { skp_rt_value_from_function(7) };
    assert_eq!(unsafe { skp_rt_value_to_function(fn_ptr) }, 7);
    assert_eq!(
        unsafe { (*(fn_ptr as *mut RtValue)).expect_function().expect("fn") },
        RtFunctionRef(7)
    );

    let repeated = unsafe { skp_rt_array_repeat(skp_rt_value_from_int(9), 2) };
    let second = unsafe { skp_rt_array_get(repeated, 1) };
    assert_eq!(
        unsafe { (*(second as *mut RtValue)).expect_int().expect("int") },
        9
    );

    let vec_ptr = unsafe { skp_rt_vec_new() };
    unsafe { skp_rt_vec_push(vec_ptr, skp_rt_value_from_int(5)) };
    let got = unsafe { skp_rt_vec_get(vec_ptr, 0) };
    assert_eq!(
        unsafe { (*(got as *mut RtValue)).expect_int().expect("int") },
        5
    );
}
