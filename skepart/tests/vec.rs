use skepart::{RtErrorKind, RtFunctionRef, RtString, RtStruct, RtValue, RtVec};

#[test]
fn vecs_share_aliasing_and_support_boundary_mutations() {
    let first = RtVec::new();
    let second = first.clone();
    first.push(RtValue::Int(1));
    first.push(RtValue::Int(2));
    first.push(RtValue::Int(3));
    second.set(0, RtValue::Int(9)).expect("set should work");
    assert_eq!(first.get(0), Ok(RtValue::Int(9)));
    assert_eq!(second.delete(2), Ok(RtValue::Int(3)));
    assert_eq!(first.len(), 2);
}

#[test]
fn vecs_report_empty_and_oob_errors() {
    let vec = RtVec::new();
    assert_eq!(
        vec.get(0).expect_err("empty get").kind,
        RtErrorKind::IndexOutOfBounds
    );
    assert_eq!(
        vec.delete(0).expect_err("empty delete").kind,
        RtErrorKind::IndexOutOfBounds
    );
}

#[test]
fn vecs_hold_structs_functions_and_strings() {
    let vec = RtVec::new();
    vec.push(RtValue::Struct(RtStruct::named(
        "Pair",
        vec![RtValue::Int(1)],
    )));
    vec.push(RtValue::Function(RtFunctionRef(7)));
    vec.push(RtValue::String(RtString::from("done")));
    assert!(matches!(vec.get(0), Ok(RtValue::Struct(_))));
    assert_eq!(vec.get(1), Ok(RtValue::Function(RtFunctionRef(7))));
    assert_eq!(vec.get(2), Ok(RtValue::String(RtString::from("done"))));
}
