use skepart::{RtErrorKind, RtFunctionRef, RtValue};

#[test]
fn value_accessors_return_expected_values() {
    assert_eq!(RtValue::Int(4).expect_int(), Ok(4));
    assert_eq!(RtValue::Float(2.5).expect_float(), Ok(2.5));
    assert_eq!(RtValue::Bool(true).expect_bool(), Ok(true));
    assert_eq!(
        RtValue::Function(RtFunctionRef(3)).expect_function(),
        Ok(RtFunctionRef(3))
    );
}

#[test]
fn value_accessors_report_wrong_type() {
    assert_eq!(
        RtValue::Bool(true)
            .expect_int()
            .expect_err("wrong type")
            .kind,
        RtErrorKind::TypeMismatch
    );
    assert_eq!(
        RtValue::Int(1)
            .expect_string()
            .expect_err("wrong type")
            .kind,
        RtErrorKind::TypeMismatch
    );
    assert_eq!(
        RtValue::Unit.expect_vec().expect_err("wrong type").kind,
        RtErrorKind::TypeMismatch
    );
}
