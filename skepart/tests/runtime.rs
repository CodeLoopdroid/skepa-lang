use skepart::{RtArray, RtErrorKind, RtString, RtValue, RtVec};

#[test]
fn arrays_use_copy_on_write_value_storage() {
    let mut first = RtArray::repeat(RtValue::Int(1), 3);
    let second = first.clone();

    first
        .set(1, RtValue::Int(9))
        .expect("array write should succeed");

    assert_eq!(first.get(1), Ok(RtValue::Int(9)));
    assert_eq!(second.get(1), Ok(RtValue::Int(1)));
}

#[test]
fn vecs_use_shared_handle_semantics() {
    let first = RtVec::new();
    let second = first.clone();

    first.borrow_mut().push(RtValue::Int(7));

    assert_eq!(second.borrow().as_slice(), &[RtValue::Int(7)]);
}

#[test]
fn strings_track_character_length() {
    let value = RtString::from("naive");
    assert_eq!(value.len_chars(), 5);
}

#[test]
fn arrays_report_bounds_errors() {
    let array = RtArray::repeat(RtValue::Int(0), 2);
    let err = array.get(3).expect_err("index should be rejected");
    assert_eq!(err.kind, RtErrorKind::IndexOutOfBounds);
}
