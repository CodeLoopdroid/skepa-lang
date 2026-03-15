use super::*;

#[test]
fn vm_runs_with_custom_builtin_registry_extension() {
    let src = r#"
fn main() -> Int {
  return math.inc(41);
}
"#;
    let module = compile_source(src).expect("compile");
    let mut reg = BuiltinRegistry::with_defaults();
    reg.register("math", "inc", custom_math_inc);
    let mut host = TestHost::default();
    let out = Vm::run_module_main_with_registry(&module, &mut host, &reg).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn vm_runs_os_cwd_builtin() {
    let src = r#"
import os;
fn main() -> String {
  return os.cwd();
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    match out {
        Value::String(s) => assert!(!s.is_empty()),
        other => panic!("expected String from os.cwd, got {:?}", other),
    }
}
#[test]
fn vm_runs_os_platform_builtin() {
    let src = r#"
import os;
fn main() -> String {
  return os.platform();
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    match out {
        Value::String(s) => assert!(matches!(&*s, "windows" | "linux" | "macos")),
        other => panic!("expected String from os.platform, got {:?}", other),
    }
}

#[test]
fn vm_os_cwd_rejects_wrong_arity() {
    let src = r#"
import os;
fn main() -> String {
  return os.cwd(1);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("os.cwd expects 0 arguments"));
}

#[test]
fn vm_os_platform_rejects_wrong_arity() {
    let src = r#"
import os;
fn main() -> String {
  return os.platform(1);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("os.platform expects 0 arguments"));
}

#[test]
fn vm_runs_os_sleep_builtin() {
    let src = r#"
import os;
fn main() -> Int {
  os.sleep(1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn vm_os_sleep_rejects_negative_ms() {
    let src = r#"
import os;
fn main() -> Int {
  os.sleep(-1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected os.sleep error");
    assert_eq!(err.kind, VmErrorKind::HostError);
    assert!(
        err.message
            .contains("os.sleep expects non-negative milliseconds")
    );
}

#[test]
fn vm_os_sleep_rejects_wrong_arity() {
    let src = r#"
import os;
fn main() -> Int {
  os.sleep();
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("os.sleep expects 1 argument"));
}

#[test]
fn vm_os_sleep_rejects_wrong_type() {
    let src = r#"
import os;
fn main() -> Int {
  os.sleep(true);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected type error");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("os.sleep expects Int argument"));
}

#[test]
fn vm_runs_os_exec_shell_and_returns_exit_code() {
    let cmd = if cfg!(target_os = "windows") {
        "exit /b 0"
    } else {
        "exit 0"
    };
    let src = format!(
        r#"
import os;
fn main() -> Int {{
  return os.execShell("{cmd}");
}}
"#
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn vm_os_exec_shell_returns_non_zero_exit_code() {
    let cmd = if cfg!(target_os = "windows") {
        "exit /b 7"
    } else {
        "exit 7"
    };
    let src = format!(
        r#"
import os;
fn main() -> Int {{
  return os.execShell("{cmd}");
}}
"#
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(7));
}

#[test]
fn vm_runs_os_exec_shell_out_and_captures_stdout() {
    let cmd = if cfg!(target_os = "windows") {
        "echo hello"
    } else {
        "printf hello"
    };
    let src = format!(
        r#"
import os;
fn main() -> String {{
  return os.execShellOut("{cmd}");
}}
"#
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    match out {
        Value::String(s) => assert!(s.contains("hello")),
        other => panic!("expected String from os.execShellOut, got {:?}", other),
    }
}

#[test]
fn vm_os_exec_shell_out_rejects_wrong_arity() {
    let src = r#"
import os;
fn main() -> String {
  return os.execShellOut();
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("os.execShellOut expects 1 argument"));
}

#[test]
fn vm_os_exec_shell_out_rejects_wrong_type() {
    let src = r#"
import os;
fn main() -> String {
  return os.execShellOut(false);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected type error");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(
        err.message
            .contains("os.execShellOut expects String argument")
    );
}

#[test]
fn vm_os_exec_shell_rejects_wrong_type() {
    let src = r#"
import os;
fn main() -> Int {
  return os.execShell(1);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected type error");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("os.execShell expects String argument"));
}

#[test]
fn vm_runs_fs_exists_for_missing_path() {
    let src = r#"
import fs;
fn main() -> Bool {
  return fs.exists("nope");
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Bool(false));
}

#[test]
fn vm_runs_fs_join_builtin() {
    let src = r#"
import fs;
import str;
fn main() -> Int {
  let p = fs.join("alpha", "beta");
  if (str.contains(p, "alpha") && str.contains(p, "beta")) {
    return 0;
  }
  return 1;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn vm_fs_exists_rejects_wrong_type() {
    let src = r#"
import fs;
fn main() -> Bool {
  return fs.exists(1);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected type error");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("fs.exists expects String argument"));
}

#[test]
fn vm_fs_join_rejects_wrong_arity() {
    let src = r#"
import fs;
fn main() -> String {
  return fs.join("a");
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("fs.join expects 2 arguments"));
}

#[test]
fn vm_runs_fs_mkdir_all_and_remove_dir_all() {
    let root = make_temp_dir("fs_mkdir_remove_dir");
    let nested = root.join("a").join("b").join("c");
    let nested_s = sk_string_escape(&nested.display().to_string());
    let root_s = sk_string_escape(&root.display().to_string());
    let src = format!(
        r#"
import fs;
fn main() -> Int {{
  fs.mkdirAll("{0}");
  if (!fs.exists("{0}")) {{
    return 1;
  }}
  fs.removeDirAll("{1}");
  if (fs.exists("{1}")) {{
    return 2;
  }}
  return 0;
}}
"#,
        nested_s, root_s
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn vm_runs_fs_remove_file() {
    let root = make_temp_dir("fs_remove_file");
    let file = root.join("x.txt");
    let file_s = sk_string_escape(&file.display().to_string());
    fs::write(&file, "x").expect("seed file");
    let src = format!(
        r#"
import fs;
fn main() -> Int {{
  fs.removeFile("{0}");
  if (fs.exists("{0}")) {{
    return 1;
  }}
  return 0;
}}
"#,
        file_s
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vm_fs_remove_file_missing_path_errors() {
    let src = r#"
import fs;
fn main() -> Int {
  fs.removeFile("definitely_missing_file_123456.tmp");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected removeFile error");
    assert_eq!(err.kind, VmErrorKind::HostError);
    assert!(err.message.contains("fs.removeFile failed"));
}

#[test]
fn vm_fs_remove_dir_all_missing_path_errors() {
    let src = r#"
import fs;
fn main() -> Int {
  fs.removeDirAll("definitely_missing_dir_123456");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected removeDirAll error");
    assert_eq!(err.kind, VmErrorKind::HostError);
    assert!(err.message.contains("fs.removeDirAll failed"));
}

#[test]
fn vm_fs_mkdir_all_rejects_wrong_type() {
    let src = r#"
import fs;
fn main() -> Int {
  fs.mkdirAll(1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected type error");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("fs.mkdirAll expects String argument"));
}

#[test]
fn vm_fs_remove_dir_all_rejects_wrong_arity() {
    let src = r#"
import fs;
fn main() -> Int {
  fs.removeDirAll();
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("fs.removeDirAll expects 1 argument"));
}

#[test]
fn vm_runs_fs_write_and_read_text() {
    let root = make_temp_dir("fs_write_read");
    let file = root.join("x.txt");
    let file_s = sk_string_escape(&file.display().to_string());
    let src = format!(
        r#"
import fs;
fn main() -> String {{
  fs.writeText("{0}", "hello");
  return fs.readText("{0}");
}}
"#,
        file_s
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::String("hello".to_string().into()));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vm_fs_write_text_overwrites_existing_file() {
    let root = make_temp_dir("fs_write_overwrite");
    let file = root.join("x.txt");
    let file_s = sk_string_escape(&file.display().to_string());
    fs::write(&file, "old").expect("seed file");
    let src = format!(
        r#"
import fs;
fn main() -> String {{
  fs.writeText("{0}", "new");
  return fs.readText("{0}");
}}
"#,
        file_s
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::String("new".to_string().into()));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vm_fs_append_text_appends_and_can_create_file() {
    let root = make_temp_dir("fs_append");
    let file = root.join("x.txt");
    let file_s = sk_string_escape(&file.display().to_string());
    let src = format!(
        r#"
import fs;
fn main() -> String {{
  fs.appendText("{0}", "a");
  fs.appendText("{0}", "b");
  return fs.readText("{0}");
}}
"#,
        file_s
    );
    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::String("ab".to_string().into()));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vm_fs_read_text_missing_file_errors() {
    let root = make_temp_dir("fs_read_missing");
    let file = root.join("missing.txt");
    let file_s = sk_string_escape(&file.display().to_string());
    let src = format!(
        r#"
import fs;
fn main() -> String {{
  return fs.readText("{0}");
}}
"#,
        file_s
    );
    let module = compile_source(&src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected read error");
    assert_eq!(err.kind, VmErrorKind::HostError);
    assert!(err.message.contains("fs.readText failed"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vm_fs_write_and_append_error_when_parent_missing() {
    let root = make_temp_dir("fs_parent_missing");
    let missing_parent = root.join("missing_dir");
    let write_path = missing_parent.join("a.txt");
    let write_path_s = sk_string_escape(&write_path.display().to_string());
    let write_src = format!(
        r#"
import fs;
fn main() -> Int {{
  fs.writeText("{0}", "x");
  return 0;
}}
"#,
        write_path_s
    );
    let write_mod = compile_source(&write_src).expect("compile");
    let write_err = Vm::run_module_main(&write_mod).expect_err("expected write error");
    assert_eq!(write_err.kind, VmErrorKind::HostError);
    assert!(write_err.message.contains("fs.writeText failed"));

    let append_src = format!(
        r#"
import fs;
fn main() -> Int {{
  fs.appendText("{0}", "x");
  return 0;
}}
"#,
        write_path_s
    );
    let append_mod = compile_source(&append_src).expect("compile");
    let append_err = Vm::run_module_main(&append_mod).expect_err("expected append error");
    assert_eq!(append_err.kind, VmErrorKind::HostError);
    assert!(append_err.message.contains("fs.appendText failed"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn vm_fs_read_text_rejects_wrong_type() {
    let src = r#"
import fs;
fn main() -> String {
  return fs.readText(1);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected type error");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("fs.readText expects String argument"));
}

#[test]
fn vm_fs_write_text_rejects_wrong_arity() {
    let src = r#"
import fs;
fn main() -> Int {
  fs.writeText("a");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("expected arity error");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("fs.writeText expects 2 arguments"));
}
