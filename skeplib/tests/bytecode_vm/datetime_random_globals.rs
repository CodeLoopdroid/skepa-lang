use super::*;

#[test]
fn runs_datetime_now_builtins() {
    let src = r#"
import datetime;
fn main() -> Int {
  let s = datetime.nowUnix();
  let ms = datetime.nowMillis();
  if (s < 0) {
    return 1;
  }
  if (ms < s * 1000) {
    return 2;
  }
  if (ms > (s + 2) * 1000) {
    return 3;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn vm_reports_datetime_runtime_arity_mismatch_from_manual_bytecode() {
    let unix_arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "nowUnix".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&unix_arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("datetime.nowUnix expects 0 arguments"));

    let millis_arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "nowMillis".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&millis_arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(
        err.message
            .contains("datetime.nowMillis expects 0 arguments")
    );
}

#[test]
fn runs_datetime_from_unix_and_millis() {
    let src = r#"
import datetime;
fn main() -> Int {
  let a = datetime.fromUnix(0);
  let b = datetime.fromMillis(1234);
  let c = datetime.fromUnix(-1);
  let d = datetime.fromMillis(-1);
  if (a == "1970-01-01T00:00:00Z"
      && b == "1970-01-01T00:00:01.234Z"
      && c == "1969-12-31T23:59:59Z"
      && d == "1969-12-31T23:59:59.999Z") {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_reports_datetime_from_runtime_errors_from_manual_bytecode() {
    let from_unix_arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "fromUnix".to_string(),
                        argc: 0,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&from_unix_arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("datetime.fromUnix expects 1 argument"));

    let from_millis_type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::String("bad".to_string().into())),
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "fromMillis".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&from_millis_type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(
        err.message
            .contains("datetime.fromMillis expects Int argument")
    );
}

#[test]
fn runs_datetime_parse_unix() {
    let src = r#"
import datetime;
fn main() -> Int {
  let z = datetime.parseUnix("1970-01-01T00:00:00Z");
  let p = datetime.parseUnix("1970-01-01T00:00:01Z");
  let n = datetime.parseUnix("1969-12-31T23:59:59Z");
  if (z == 0 && p == 1 && n == -1) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_reports_datetime_parse_unix_invalid_format() {
    let src = r#"
import datetime;
fn main() -> Int {
  let _x = datetime.parseUnix("2026-02-17 12:34:56");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("invalid datetime format");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("datetime.parseUnix expects format"));
}

#[test]
fn vm_reports_datetime_parse_unix_runtime_errors_from_manual_bytecode() {
    let arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "parseUnix".to_string(),
                        argc: 0,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(
        err.message
            .contains("datetime.parseUnix expects 1 argument")
    );

    let type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "parseUnix".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(
        err.message
            .contains("datetime.parseUnix expects String argument")
    );
}

#[test]
fn runs_datetime_component_extractors() {
    let src = r#"
import datetime;
fn main() -> Int {
  let ts = 1704112496; // 2024-01-01T12:34:56Z
  if (datetime.year(ts) == 2024
      && datetime.month(ts) == 1
      && datetime.day(ts) == 1
      && datetime.hour(ts) == 12
      && datetime.minute(ts) == 34
      && datetime.second(ts) == 56) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_reports_datetime_component_runtime_errors_from_manual_bytecode() {
    let arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "year".to_string(),
                        argc: 0,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("datetime.year expects 1 argument"));

    let type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::String("bad".to_string().into())),
                    Instr::CallBuiltin {
                        package: "datetime".to_string(),
                        name: "second".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("datetime.second expects Int argument"));
}

#[test]
fn runs_datetime_roundtrip_from_unix_and_parse_unix() {
    let src = r#"
import datetime;
fn main() -> Int {
  let a = 0;
  let b = -1;
  let c = 1704112496;
  if (datetime.parseUnix(datetime.fromUnix(a)) == a
      && datetime.parseUnix(datetime.fromUnix(b)) == b
      && datetime.parseUnix(datetime.fromUnix(c)) == c) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn runs_datetime_parse_unix_leap_year_and_rejects_invalid_date() {
    let ok_src = r#"
import datetime;
fn main() -> Int {
  let ts = datetime.parseUnix("2024-02-29T00:00:00Z");
  if (datetime.month(ts) == 2 && datetime.day(ts) == 29) {
    return 1;
  }
  return 0;
}
"#;
    let ok_module = compile_source(ok_src).expect("compile");
    let out = Vm::run_module_main(&ok_module).expect("run");
    assert_eq!(out, Value::Int(1));

    let bad_src = r#"
import datetime;
fn main() -> Int {
  let _x = datetime.parseUnix("2023-02-29T00:00:00Z");
  return 0;
}
"#;
    let bad_module = compile_source(bad_src).expect("compile");
    let err = Vm::run_module_main(&bad_module).expect_err("invalid date");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("datetime.parseUnix day out of range"));
}

#[test]
fn vm_reports_datetime_parse_unix_invalid_time_ranges() {
    let bad_hour = r#"
import datetime;
fn main() -> Int {
  let _x = datetime.parseUnix("2026-01-01T24:00:00Z");
  return 0;
}
"#;
    let m1 = compile_source(bad_hour).expect("compile");
    let e1 = Vm::run_module_main(&m1).expect_err("hour out of range");
    assert_eq!(e1.kind, VmErrorKind::TypeMismatch);
    assert!(e1.message.contains("datetime.parseUnix time out of range"));

    let bad_month = r#"
import datetime;
fn main() -> Int {
  let _x = datetime.parseUnix("2026-13-01T00:00:00Z");
  return 0;
}
"#;
    let m2 = compile_source(bad_month).expect("compile");
    let e2 = Vm::run_module_main(&m2).expect_err("month out of range");
    assert_eq!(e2.kind, VmErrorKind::TypeMismatch);
    assert!(e2.message.contains("datetime.parseUnix month out of range"));
}

#[test]
fn runs_random_seed_and_updates_host_state() {
    let src = r#"
import random;
fn main() -> Int {
  random.seed(12345);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let mut host = TestHost::default();
    let out = Vm::run_module_main_with_host(&module, &mut host).expect("run");
    assert_eq!(out, Value::Int(0));
    assert_eq!(host.rng_state, 12345u64);
}

#[test]
fn vm_reports_random_seed_runtime_errors_from_manual_bytecode() {
    let arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "random".to_string(),
                        name: "seed".to_string(),
                        argc: 0,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("random.seed expects 1 argument"));

    let type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::String("bad".to_string().into())),
                    Instr::CallBuiltin {
                        package: "random".to_string(),
                        name: "seed".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("random.seed expects Int argument"));
}

#[test]
fn runs_random_int_and_float_with_seed_determinism_and_ranges() {
    let src = r#"
import random;
fn main() -> Int {
  random.seed(42);
  let i1 = random.int(10, 20);
  let f1 = random.float();
  random.seed(42);
  let i2 = random.int(10, 20);
  let f2 = random.float();
  if (i1 == i2 && f1 == f2 && i1 >= 10 && i1 <= 20 && f1 >= 0.0 && f1 < 1.0) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_reports_random_int_float_runtime_arity_errors_from_manual_bytecode() {
    let int_arity = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "random".to_string(),
                        name: "int".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&int_arity).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("random.int expects 2 arguments"));

    let float_arity = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "random".to_string(),
                        name: "float".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&float_arity).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("random.float expects 0 arguments"));
}

#[test]
fn vm_reports_random_int_runtime_type_and_bounds_errors() {
    let src = r#"
import random;
fn main() -> Int {
  let _x = random.int(10, 5);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("invalid bounds");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("random.int expects min <= max"));

    let type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::String("x".to_string().into())),
                    Instr::LoadConst(Value::Int(2)),
                    Instr::CallBuiltin {
                        package: "random".to_string(),
                        name: "int".to_string(),
                        argc: 2,
                    },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("random.int argument 1 expects Int"));
}

#[test]
fn runs_random_int_single_point_range_is_constant() {
    let src = r#"
import random;
fn main() -> Int {
  random.seed(999);
  let a = random.int(5, 5);
  let b = random.int(5, 5);
  if (a == 5 && b == 5) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn runs_datetime_component_extractors_for_negative_timestamp() {
    let src = r#"
import datetime;
fn main() -> Int {
  let ts = -1;
  if (datetime.year(ts) == 1969
      && datetime.month(ts) == 12
      && datetime.day(ts) == 31
      && datetime.hour(ts) == 23
      && datetime.minute(ts) == 59
      && datetime.second(ts) == 59) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_reports_datetime_parse_unix_invalid_non_digit_fields() {
    let src = r#"
import datetime;
fn main() -> Int {
  let _x = datetime.parseUnix("202A-01-01T00:00:00Z");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("invalid year");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("datetime.parseUnix invalid year"));
}

#[test]
fn runs_global_variables_across_functions() {
    let src = r#"
let counter: Int = 0;

fn inc() -> Int {
  counter = counter + 1;
  return counter;
}

fn main() -> Int {
  let a = inc();
  let b = inc();
  return a * 10 + b;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(12));
}

#[test]
fn runs_global_initializer_before_main() {
    let src = r#"
let g: Int = seed();

fn seed() -> Int {
  return 7;
}

fn main() -> Int {
  return g;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(7));
}

#[test]
fn lowering_uses_fully_qualified_name_for_from_import_call() {
    let src = r#"
from utils.math import add as plus;
fn main() -> Int {
  return plus(1, 2);
}
"#;
    let module = compile_source(src).expect("compile");
    let main = module.functions.get("main").expect("main fn");
    assert!(main.code.iter().any(|i| {
        matches!(
            i,
            Instr::Call { name, argc } if name == "utils.math.add" && *argc == 2
        )
    }));
}

#[test]
fn lowering_uses_fully_qualified_name_for_namespace_call() {
    let src = r#"
import utils.math;
fn main() -> Int {
  return utils.math.add(1, 2);
}
"#;
    let module = compile_source(src).expect("compile");
    let main = module.functions.get("main").expect("main fn");
    assert!(main.code.iter().any(|i| {
        matches!(
            i,
            Instr::Call { name, argc } if name == "utils.math.add" && *argc == 2
        )
    }));
}
