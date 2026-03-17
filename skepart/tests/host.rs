mod common;

use common::RecordingHost;
use skepart::{NoopHost, RtHost, RtString};

#[test]
fn noop_host_supports_print_and_time_defaults() {
    let mut host = NoopHost;
    host.io_print("hello").expect("print");
    host.io_println("world").expect("println");
    assert!(host.datetime_now_unix().expect("unix") > 0);
    assert!(host.datetime_now_millis().expect("millis") > 0);
}

#[test]
fn recording_host_captures_output_and_overrides_services() {
    let mut host = RecordingHost::seeded();
    host.io_print("a").expect("print");
    host.io_println("b").expect("println");
    assert_eq!(host.output, "ab\n");
    assert_eq!(
        host.io_read_line().expect("read line"),
        RtString::from("typed line")
    );
    assert_eq!(
        host.os_platform().expect("platform"),
        RtString::from("test-os")
    );
    assert_eq!(
        host.fs_read_text("file.txt").expect("read"),
        RtString::from("read:file.txt")
    );
}
