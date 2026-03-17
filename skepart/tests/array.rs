use skepart::{RtArray, RtErrorKind, RtString, RtValue};

#[test]
fn arrays_use_copy_on_write_and_preserve_nested_values() {
    let mut first = RtArray::repeat(
        RtValue::Array(RtArray::new(vec![RtValue::Int(1), RtValue::Int(2)])),
        2,
    );
    let second = first.clone();
    first.set(0, RtValue::Int(9)).expect("set should work");

    assert_eq!(first.get(0), Ok(RtValue::Int(9)));
    match second.get(0).expect("nested array should remain") {
        RtValue::Array(inner) => assert_eq!(inner.get(1), Ok(RtValue::Int(2))),
        other => panic!("expected nested array, got {other:?}"),
    }
}

#[test]
fn arrays_report_get_and_set_bounds() {
    let mut array = RtArray::repeat(RtValue::Int(0), 2);
    assert_eq!(
        array.get(3).expect_err("oob").kind,
        RtErrorKind::IndexOutOfBounds
    );
    assert_eq!(
        array.set(2, RtValue::Int(4)).expect_err("oob set").kind,
        RtErrorKind::IndexOutOfBounds
    );
}

#[test]
fn arrays_store_mixed_runtime_values() {
    let array = RtArray::new(vec![
        RtValue::Int(1),
        RtValue::Bool(true),
        RtValue::String(RtString::from("hi")),
    ]);
    assert_eq!(array.get(0), Ok(RtValue::Int(1)));
    assert_eq!(array.get(1), Ok(RtValue::Bool(true)));
    assert_eq!(array.get(2), Ok(RtValue::String(RtString::from("hi"))));
}
