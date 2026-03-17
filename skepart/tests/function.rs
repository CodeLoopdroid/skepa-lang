mod common;

use common::RecordingHost;
use skepart::{RtErrorKind, RtFunctionRef, RtFunctionRegistry, RtValue};

fn add_one(_host: &mut dyn skepart::RtHost, args: &[RtValue]) -> skepart::RtResult<RtValue> {
    Ok(RtValue::Int(args[0].expect_int()? + 1))
}

fn sum_two(_host: &mut dyn skepart::RtHost, args: &[RtValue]) -> skepart::RtResult<RtValue> {
    Ok(RtValue::Int(args[0].expect_int()? + args[1].expect_int()?))
}

#[test]
fn function_registry_calls_registered_functions() {
    let mut registry = RtFunctionRegistry::new();
    let f = registry.register(add_one);
    let mut host = RecordingHost::seeded();
    assert_eq!(
        registry
            .call(&mut host, f, &[RtValue::Int(4)])
            .expect("call"),
        RtValue::Int(5)
    );
}

#[test]
fn function_registry_reports_missing_function_id() {
    let registry = RtFunctionRegistry::new();
    let mut host = RecordingHost::seeded();
    assert_eq!(
        registry
            .call(&mut host, RtFunctionRef(99), &[RtValue::Int(1)])
            .expect_err("missing id")
            .kind,
        RtErrorKind::UnsupportedBuiltin
    );
}

#[test]
fn function_registry_preserves_argument_type_checks_inside_function() {
    let mut registry = RtFunctionRegistry::new();
    let f = registry.register(sum_two);
    let mut host = RecordingHost::seeded();
    assert_eq!(
        registry
            .call(&mut host, f, &[RtValue::Int(1), RtValue::Bool(true)])
            .expect_err("type mismatch")
            .kind,
        RtErrorKind::TypeMismatch
    );
}
