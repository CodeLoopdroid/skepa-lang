use super::*;

#[test]
fn disassemble_outputs_named_instructions() {
    let src = r#"
fn main() -> Int {
  let x = 1;
  return x + 2;
}
"#;
    let module = compile_source(src).expect("compile");
    let txt = module.disassemble();
    assert!(txt.contains("fn main"));
    assert!(txt.contains("LoadConst Int(1)"));
    assert!(txt.contains("Add"));
    assert!(txt.contains("Return"));
}

#[test]
fn disassemble_includes_short_circuit_and_modulo_instruction_flow() {
    let src = r#"
fn main() -> Int {
  if (true || false) {
    return 8 % 3;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let txt = module.disassemble();
    assert!(txt.contains("JumpIfTrue"));
    assert!(txt.contains("ModInt") || txt.contains("IntStackConstOp op=Mod"));
}
#[test]
fn runs_float_arithmetic() {
    let src = r#"
fn main() -> Float {
  let x = 8.0;
  x = x / 2.0;
  return x + 0.25;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Float(4.25));
}

#[test]
fn supports_float_comparison_in_conditionals() {
    let src = r#"
fn main() -> Int {
  if (2.5 > 2.0) {
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
fn modulo_handles_negative_operands_with_rust_semantics() {
    let src = r#"
fn main() -> Int {
  let a = -7 % 3;
  let b = 7 % -3;
  if (a == -1 && b == 1) {
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
fn float_negative_zero_compares_equal_and_not_less_than_zero() {
    let src = r#"
fn main() -> Int {
  let z: Float = -0.0;
  if (z == 0.0 && !(z < 0.0) && !(z > 0.0)) {
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
fn bytecode_roundtrip_preserves_float_constant() {
    let src = r#"
fn main() -> Float { return 3.5; }
"#;
    let module = compile_source(src).expect("compile");
    let bytes = module.to_bytes();
    let decoded = BytecodeModule::from_bytes(&bytes).expect("decode");
    let out = Vm::run_module_main(&decoded).expect("run");
    assert_eq!(out, Value::Float(3.5));
}

#[test]
fn bytecode_roundtrip_preserves_array_constants_and_ops() {
    let src = r#"
fn main() -> Int {
  let a: [Int; 3] = [2, 4, 6];
  return a[0] + a[2];
}
"#;
    let module = compile_source(src).expect("compile");
    let bytes = module.to_bytes();
    let decoded = BytecodeModule::from_bytes(&bytes).expect("decode");
    let out = Vm::run_module_main(&decoded).expect("run");
    assert_eq!(out, Value::Int(8));
}

#[test]
fn bytecode_roundtrip_preserves_struct_values_and_method_dispatch() {
    let src = r#"
struct User { id: Int, name: String }
impl User {
  fn bump(self, d: Int) -> Int {
    return self.id + d;
  }
}
fn main() -> Int {
  let u = User { id: 10, name: "sam" };
  return u.bump(5);
}
"#;
    let module = compile_source(src).expect("compile");
    let bytes = module.to_bytes();
    let decoded = BytecodeModule::from_bytes(&bytes).expect("decode");
    let out = Vm::run_module_main(&decoded).expect("run");
    assert_eq!(out, Value::Int(15));
}

#[test]
fn runs_io_format_and_printf_with_escapes_and_percent() {
    let src = r#"
import io;
fn main() -> Int {
  let msg = io.format("v=%d f=%f ok=%b s=%s %%\n", 5, 1.5, true, "yo");
  io.printf("%s\t%s\\", msg, "done");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let mut host = TestHost::default();
    let out = Vm::run_module_main_with_host(&module, &mut host).expect("run");
    assert_eq!(out, Value::Int(0));
    assert_eq!(host.output, "v=5 f=1.5 ok=true s=yo %\n\tdone\\");
}

#[test]
fn vm_reports_io_format_runtime_type_mismatch() {
    let src = r#"
import io;
fn main() -> Int {
  let _x = io.format("n=%d", "bad");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("must be Int for `%d`"));
}

#[test]
fn vm_reports_io_printf_runtime_arity_mismatch() {
    let src = r#"
import io;
fn main() -> Int {
  io.printf("%d %d", 1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("expects 2 value argument(s), got 1"));
}

#[test]
fn runs_typed_io_print_builtins_with_newlines() {
    let src = r#"
import io;
fn main() -> Int {
  io.printInt(7);
  io.printFloat(2.5);
  io.printBool(false);
  io.printString("ok");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let mut host = TestHost::default();
    let out = Vm::run_module_main_with_host(&module, &mut host).expect("run");
    assert_eq!(out, Value::Int(0));
    assert_eq!(host.output, "7\n2.5\nfalse\nok\n");
}

#[test]
fn vm_reports_typed_io_print_runtime_mismatch() {
    let src = r#"
import io;
fn main() -> Int {
  let fmt = "x";
  io.printInt(fmt);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("io.printInt expects Int argument"));
}

#[test]
fn vm_io_print_and_println_have_expected_newline_behavior() {
    let src = r#"
import io;
fn main() -> Int {
  io.print("a");
  io.print("b");
  io.println("c");
  io.print("d");
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let mut host = TestHost::default();
    let out = Vm::run_module_main_with_host(&module, &mut host).expect("run");
    assert_eq!(out, Value::Int(0));
    assert_eq!(host.output, "abc\nd");
}

#[test]
fn vm_reports_io_print_runtime_arity_mismatch() {
    let src = r#"
import io;
fn main() -> Int {
  io.print();
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("io.print expects 1 argument"));
}

#[test]
fn vm_reports_io_printf_runtime_invalid_specifier_for_dynamic_format() {
    let src = r#"
import io;
fn main() -> Int {
  let fmt = "bad=%q";
  io.printf(fmt, 1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("invalid specifier");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("unsupported format specifier `%q`"));
}

#[test]
fn vm_reports_io_format_runtime_trailing_percent_for_dynamic_format() {
    let src = r#"
import io;
fn main() -> Int {
  let fmt = "oops %";
  let _s = io.format(fmt, 1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("trailing percent");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("format string ends with `%`"));
}

#[test]
fn runs_arr_package_generic_ops_and_array_add() {
    let src = r#"
import arr;
fn main() -> Int {
  let a: [Int; 4] = [1, 2, 3, 2];
  let b: [Int; 2] = [9, 8];
  let c = a + b;
  if (arr.len(c) == 6 && !arr.isEmpty(c) && arr.contains(c, 8) && arr.indexOf(c, 2) == 1) {
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
fn runs_arr_contains_and_indexof_for_nested_arrays() {
    let src = r#"
import arr;
fn main() -> Int {
  let rows: [[Int; 2]; 3] = [[1, 2], [3, 4], [5, 6]];
  if (arr.contains(rows, [3, 4]) && arr.indexOf(rows, [5, 6]) == 2 && arr.indexOf(rows, [9, 9]) == -1) {
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
fn arr_is_empty_handles_zero_sized_arrays() {
    let src = r#"
import arr;
fn main() -> Int {
  let z: [Int; 0] = [1; 0];
  if (arr.isEmpty(z) && arr.len(z) == 0) {
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
fn vm_reports_arr_builtin_runtime_arity_mismatch_from_manual_bytecode() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Array(vec![Value::Int(1)].into())),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "len".to_string(),
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
    let err = Vm::run_module_main(&module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("arr.len expects 1 argument"));
}

#[test]
fn vm_reports_arr_count_runtime_errors_from_manual_bytecode() {
    let arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Array(vec![Value::Int(1)].into())),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "count".to_string(),
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
    let err = Vm::run_module_main(&arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("arr.count expects 2 arguments"));

    let type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "count".to_string(),
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
    assert!(
        err.message
            .contains("arr.count expects Array as first argument")
    );
}

#[test]
fn vm_reports_arr_first_last_runtime_errors_from_manual_bytecode() {
    let first_arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Array(vec![Value::Int(1)].into())),
                    Instr::LoadConst(Value::Int(2)),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "first".to_string(),
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
    let err = Vm::run_module_main(&first_arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("arr.first expects 1 argument"));

    let first_type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::String("x".to_string().into())),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "first".to_string(),
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
    let err = Vm::run_module_main(&first_type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("arr.first expects Array argument"));

    let last_arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Array(vec![Value::Int(1)].into())),
                    Instr::LoadConst(Value::Int(2)),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "last".to_string(),
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
    let err = Vm::run_module_main(&last_arity_module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
    assert!(err.message.contains("arr.last expects 1 argument"));

    let last_type_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::String("x".to_string().into())),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "last".to_string(),
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
    let err = Vm::run_module_main(&last_type_module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("arr.last expects Array argument"));
}

#[test]
fn regression_arr_concat_large_arrays() {
    let n = 1500usize;
    let mut src = String::from("import arr;\nfn main() -> Int {\n  let a: [Int; ");
    src.push_str(&n.to_string());
    src.push_str("] = [1; ");
    src.push_str(&n.to_string());
    src.push_str("];\n  let b: [Int; ");
    src.push_str(&n.to_string());
    src.push_str("] = [2; ");
    src.push_str(&n.to_string());
    src.push_str("];\n  let c = a + b;\n");
    src.push_str("  if (arr.len(c) != ");
    src.push_str(&(2 * n).to_string());
    src.push_str(") { return 1; }\n  return arr.first(c) + arr.last(c);\n}\n");

    let module = compile_source(&src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(3));
}

#[test]
fn runs_arr_count_first_last() {
    let src = r#"
import arr;
fn main() -> Int {
  let a: [Int; 5] = [2, 9, 2, 3, 2];
  if (arr.count(a, 2) == 3 && arr.first(a) == 2 && arr.last(a) == 2) {
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
fn vm_reports_arr_first_last_on_empty_array() {
    let src = r#"
import arr;
fn main() -> Int {
  let z: [Int; 0] = [1; 0];
  let _a = arr.first(z);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("empty");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
    assert!(err.message.contains("arr.first on empty array"));
}

#[test]
fn runs_str_lastindexof_and_replace() {
    let src = r#"
import str;
fn main() -> Int {
  let s = "a-b-a-b";
  let i = str.lastIndexOf(s, "a");
  let r = str.replace(s, "-", "_");
  if (i == 4 && r == "a_b_a_b" && str.lastIndexOf(s, "z") == -1) {
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
fn runs_str_repeat() {
    let src = r#"
import str;
fn main() -> Int {
  let s = str.repeat("ab", 3);
  if (s == "ababab") {
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
fn vm_reports_str_repeat_negative_count() {
    let src = r#"
import str;
fn main() -> Int {
  let _s = str.repeat("x", -1);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("negative repeat");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
    assert!(err.message.contains("str.repeat count must be >= 0"));
}

#[test]
fn vm_reports_str_repeat_output_too_large() {
    let src = r#"
import str;
fn main() -> Int {
  let _s = str.repeat("x", 1000001);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("repeat too large");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
    assert!(err.message.contains("str.repeat output too large"));
}

#[test]
fn runs_arr_join_and_unicode_last_indexof() {
    let src = r#"
import arr;
import str;
fn main() -> Int {
  let a: [String; 3] = ["hi", "sk", "lang"];
  let j = arr.join(a, "::");
  let s = "naïve-naïve";
  let idx = str.lastIndexOf(s, "ï");
  if (j == "hi::sk::lang" && idx == 8) {
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
fn vm_reports_arr_join_runtime_type_mismatch_for_non_string_elements() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Array(vec![Value::Int(1), Value::Int(2)].into())),
                    Instr::LoadConst(Value::String(",".to_string().into())),
                    Instr::CallBuiltin {
                        package: "arr".to_string(),
                        name: "join".to_string(),
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
    let err = Vm::run_module_main(&module).expect_err("join type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("arr.join expects Array[String]"));
}
