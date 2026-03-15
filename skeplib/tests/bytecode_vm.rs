mod common;
mod bytecode_vm {
    use super::common::{assert_has_diag, compile_err, compile_ok, vm_run_ok};
    use skeplib::bytecode::{
        BytecodeModule, FunctionChunk, Instr, StructShape, Value, compile_source,
    };
    use skeplib::vm::{BuiltinHost, BuiltinRegistry, TestHost, Vm, VmConfig, VmErrorKind};
    use std::collections::VecDeque;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("skepa_vm_{label}_{nanos}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn sk_string_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('\"', "\\\"")
    }

    fn custom_math_inc(
        _host: &mut dyn BuiltinHost,
        args: Vec<Value>,
    ) -> Result<Value, skeplib::vm::VmError> {
        if args.len() != 1 {
            return Err(skeplib::vm::VmError {
                kind: VmErrorKind::ArityMismatch,
                message: "math.inc expects 1 arg".to_string(),
            });
        }
        match args[0] {
            Value::Int(v) => Ok(Value::Int(v + 1)),
            _ => Err(skeplib::vm::VmError {
                kind: VmErrorKind::TypeMismatch,
                message: "math.inc expects Int".to_string(),
            }),
        }
    }

    mod core;
    mod datetime_random_globals;
    mod os_fs;
    mod packages;
    mod vec;
}
