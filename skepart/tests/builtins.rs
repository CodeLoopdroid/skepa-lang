mod common;

use common::RecordingHost;
use skepart::{builtins, NoopHost, RtErrorKind, RtString, RtValue};

#[test]
fn builtins_dispatch_valid_core_families() {
    let mut host = RecordingHost::seeded();
    assert_eq!(
        builtins::call_with_host(&mut host, "datetime", "nowUnix", &[]).expect("datetime"),
        RtValue::Int(100)
    );
    assert_eq!(
        builtins::call("str", "len", &[RtValue::String(RtString::from("abc"))]).expect("str.len"),
        RtValue::Int(3)
    );
    assert_eq!(
        builtins::call("vec", "new", &[])
            .expect("vec.new")
            .type_name(),
        "Vec"
    );
}

#[test]
fn builtins_report_unknown_family_arity_and_type_errors() {
    let mut host = NoopHost;
    assert_eq!(
        builtins::call("missing", "fn", &[])
            .expect_err("bad family")
            .kind,
        RtErrorKind::UnsupportedBuiltin
    );
    assert_eq!(
        builtins::call("str", "len", &[])
            .expect_err("bad arity")
            .kind,
        RtErrorKind::UnsupportedBuiltin
    );
    assert_eq!(
        builtins::call("str", "len", &[RtValue::Int(1)])
            .expect_err("bad type")
            .kind,
        RtErrorKind::TypeMismatch
    );
    assert_eq!(
        builtins::call_with_host(
            &mut host,
            "random",
            "int",
            &[RtValue::Int(1), RtValue::Int(2)]
        )
        .expect_err("unsupported host")
        .kind,
        RtErrorKind::UnsupportedBuiltin
    );
}

#[test]
fn builtins_map_host_backed_results_consistently() {
    let mut host = RecordingHost::seeded();
    assert_eq!(
        builtins::call_with_host(
            &mut host,
            "fs",
            "join",
            &[
                RtValue::String(RtString::from("tmp")),
                RtValue::String(RtString::from("x.txt")),
            ],
        )
        .expect("fs.join"),
        RtValue::String(RtString::from("tmp/x.txt"))
    );
    assert_eq!(
        builtins::call_with_host(&mut host, "os", "platform", &[]).expect("os.platform"),
        RtValue::String(RtString::from("test-os"))
    );
    assert_eq!(
        builtins::call_with_host(&mut host, "random", "float", &[]).expect("random.float"),
        RtValue::Float(0.25)
    );
}
