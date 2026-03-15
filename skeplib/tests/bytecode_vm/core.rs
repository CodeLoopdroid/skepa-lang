use super::*;

#[test]
fn compiles_main_to_bytecode_with_locals_and_return() {
    let src = r#"
fn main() -> Int {
  let x = 2;
  let y = x + 3;
  return y;
}
"#;
    let module = compile_ok(src);
    let main = module.functions.get("main").expect("main chunk exists");
    assert!(main.locals_count >= 2);
    assert!(main.code.iter().any(|i| matches!(i, Instr::Add)));
    assert!(matches!(main.code.last(), Some(Instr::Return)));
}

#[test]
fn runs_compiled_main_and_returns_int() {
    let src = r#"
fn main() -> Int {
  let x = 10;
  x = x + 5;
  return x * 2;
}
"#;
    let out = vm_run_ok(src);
    assert_eq!(out, Value::Int(30));
}

#[test]
fn compile_reports_unsupported_constructs() {
    let src = r#"
fn main() -> Int {
  user.name = "x";
  return 0;
}
"#;
    let err = compile_err(src);
    assert_has_diag(&err, "Path assignment not supported");
}

#[test]
fn codegen_rejects_break_outside_loop_with_consistent_message() {
    let src = r#"
fn main() -> Int {
  break;
  return 0;
}
"#;
    let err = compile_err(src);
    assert_has_diag(&err, "`break` used outside a loop");
}

#[test]
fn codegen_rejects_continue_outside_loop_with_consistent_message() {
    let src = r#"
fn main() -> Int {
  continue;
  return 0;
}
"#;
    let err = compile_err(src);
    assert_has_diag(&err, "`continue` used outside a loop");
}

#[test]
fn runs_if_else_branching() {
    let src = r#"
fn main() -> Int {
  let x = 2;
  if (x > 1) {
    return 10;
  } else {
    return 20;
  }
}
"#;
    let out = vm_run_ok(src);
    assert_eq!(out, Value::Int(10));
}

#[test]
fn runs_while_loop_with_assignment_updates() {
    let src = r#"
fn main() -> Int {
  let i = 0;
  let acc = 0;
  while (i < 5) {
    acc = acc + i;
    i = i + 1;
  }
  return acc;
}
"#;
    let out = vm_run_ok(src);
    assert_eq!(out, Value::Int(10));
}

#[test]
fn runs_while_with_break() {
    let src = r#"
fn main() -> Int {
  let i = 0;
  while (true) {
    if (i == 4) {
      break;
    }
    i = i + 1;
  }
  return i;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(4));
}

#[test]
fn runs_while_with_continue() {
    let src = r#"
fn main() -> Int {
  let i = 0;
  let acc = 0;
  while (i < 5) {
    i = i + 1;
    if (i == 3) {
      continue;
    }
    acc = acc + i;
  }
  return acc;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(12));
}

#[test]
fn runs_for_loop_with_break_and_continue() {
    let src = r#"
fn main() -> Int {
  let acc = 0;
  for (let i = 0; i < 8; i = i + 1) {
    if (i == 2) {
      continue;
    }
    if (i == 6) {
      break;
    }
    acc = acc + (i % 3);
  }
  return acc;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(4));
}

#[test]
fn runs_match_statement_with_int_and_wildcard_dispatch() {
    let src = r#"
fn main() -> Int {
  let x = 2;
  match (x) {
    0 => { return 10; }
    2 => { return 20; }
    _ => { return 30; }
  }
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(20));
}

#[test]
fn runs_match_statement_with_string_or_pattern() {
    let src = r#"
fn main() -> Int {
  let s = "Y";
  match (s) {
    "y" | "Y" => { return 1; }
    _ => { return 0; }
  }
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn match_target_expression_is_evaluated_once() {
    let src = r#"
let n: Int = 0;

fn next() -> Int {
  n = n + 1;
  return n;
}

fn main() -> Int {
  match (next()) {
    1 => { return n; }
    _ => { return 99; }
  }
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn runs_match_statement_with_float_literal_patterns() {
    let src = r#"
fn main() -> Int {
  let x: Float = 2.5;
  match (x) {
    1.0 => { return 10; }
    2.5 | 3.5 => { return 20; }
    _ => { return 30; }
  }
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(20));
}

#[test]
fn runs_nested_match_stress_case() {
    let src = r#"
fn bucket(n: Int) -> Int {
  match (n % 3) {
    0 => {
      match (n % 2) {
        0 => { return 10; }
        _ => { return 11; }
      }
    }
    1 => {
      match (n) {
        1 | 4 | 7 => { return 20; }
        _ => { return 21; }
      }
    }
    _ => {
      match (n > 5) {
        true => { return 30; }
        _ => { return 31; }
      }
    }
  }
}

fn main() -> Int {
  let i = 0;
  let acc = 0;
  while (i < 9) {
    acc = acc + bucket(i);
    i = i + 1;
  }
  return acc;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(183));
}

#[test]
fn runs_infinite_for_loop_with_break() {
    let src = r#"
fn main() -> Int {
  let i = 0;
  for (;;) {
    if (i == 5) {
      break;
    }
    i = i + 1;
  }
  return i;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(5));
}

#[test]
fn runs_nested_for_loops_with_inner_continue() {
    let src = r#"
fn main() -> Int {
  let acc = 0;
  for (let i = 0; i < 3; i = i + 1) {
    for (let j = 0; j < 4; j = j + 1) {
      if (j == 1) {
        continue;
      }
      acc = acc + 1;
    }
  }
  return acc;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(9));
}

#[test]
fn runs_for_continue_inside_nested_if_branch() {
    let src = r#"
fn main() -> Int {
  let acc = 0;
  for (let i = 0; i < 6; i = i + 1) {
    if (i < 4) {
      if ((i % 2) == 0) {
        continue;
      }
    }
    acc = acc + i;
  }
  return acc;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(13));
}

#[test]
fn runs_static_array_literal_index_and_assignment() {
    let src = r#"
fn main() -> Int {
  let a: [Int; 3] = [1, 2, 3];
  let x = a[1];
  a[2] = x + 4;
  return a[0] + a[1] + a[2];
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(9));
}

#[test]
fn runs_static_array_repeat_literal() {
    let src = r#"
fn main() -> Int {
  let a: [Int; 4] = [3; 4];
  return a[0] + a[1] + a[2] + a[3];
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(12));
}

#[test]
fn runs_str_builtins() {
    let src = r#"
import str;
fn main() -> Int {
  let s = "  hello  ";
  let t = str.trim(s);
  if (str.contains(t, "ell") && str.startsWith(t, "he") && str.endsWith(t, "lo")) {
    return str.len(t);
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(5));
}

#[test]
fn runs_str_case_conversion_builtins() {
    let src = r#"
import str;
fn main() -> Int {
  let a = str.toLower("SkEpA");
  let b = str.toUpper("laNg");
  if (a == "skepa" && b == "LANG") {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn runs_str_indexof_slice_and_isempty() {
    let src = r#"
import str;
fn main() -> Int {
  let s = "skepa";
  let idx = str.indexOf(s, "ep");
  let miss = str.indexOf(s, "zz");
  let cut = str.slice(s, 1, 4);
  if (idx == 2 && miss == -1 && cut == "kep" && !str.isEmpty(cut) && str.isEmpty("")) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_reports_str_slice_out_of_bounds() {
    let src = r#"
import str;
fn main() -> Int {
  let _s = str.slice("abc", 1, 9);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let err = Vm::run_module_main(&module).expect_err("slice bounds");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
    assert!(err.message.contains("str.slice bounds out of range"));
}

#[test]
fn runs_nested_static_array_3d_read_write() {
    let src = r#"
fn main() -> Int {
  let t: [[[Int; 2]; 2]; 2] = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
  t[1][0][1] = 42;
  return t[1][0][1];
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_nested_static_array_4d_read_write() {
    let src = r#"
fn main() -> Int {
  let q: [[[[Int; 2]; 1]; 1]; 1] = [[[[1, 2]]]];
  q[0][0][0][1] = 9;
  return q[0][0][0][1];
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(9));
}

#[test]
fn vm_reports_array_index_out_of_bounds() {
    let src = r#"
fn main() -> Int {
  let a: [Int; 2] = [1, 2];
  return a[5];
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let err = Vm::run_module_main(&module).expect_err("oob");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
}

#[test]
fn vm_reports_nested_array_index_out_of_bounds() {
    let src = r#"
fn main() -> Int {
  let t: [[[Int; 2]; 2]; 2] = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
  return t[1][3][0];
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let err = Vm::run_module_main(&module).expect_err("oob");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
}

#[test]
fn vm_reports_type_mismatch_for_len_on_non_collection_value() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::ArrayLen,
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
}

#[test]
fn vm_reports_struct_get_type_mismatch_on_non_struct() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::StructGet("id".to_string()),
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
}

#[test]
fn vm_reports_unknown_method_on_struct_receiver() {
    let src = r#"
struct User { id: Int }
fn main() -> Int {
  let u = User { id: 1 };
  return u.nope(2);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("unknown method");
    assert_eq!(err.kind, VmErrorKind::UnknownFunction);
    assert!(
        err.message
            .contains("Unknown method `nope` on struct `User`")
    );
}

#[test]
fn vm_reports_unknown_struct_field_with_clear_message() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Struct {
                        shape: std::rc::Rc::new(StructShape {
                            name: "User".to_string(),
                            field_names: vec!["id".to_string()].into(),
                        }),
                        fields: vec![Value::Int(1)].into(),
                    }),
                    Instr::StructGet("name".to_string()),
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("unknown field");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(
        err.message
            .contains("Unknown struct field `name` on `User`")
    );
}

#[test]
fn vm_reports_struct_set_path_with_non_struct_intermediate() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Struct {
                        shape: std::rc::Rc::new(StructShape {
                            name: "User".to_string(),
                            field_names: vec!["id".to_string()].into(),
                        }),
                        fields: vec![Value::Int(1)].into(),
                    }),
                    Instr::LoadConst(Value::Int(42)),
                    Instr::StructSetPath(vec!["id".to_string(), "x".to_string()]),
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("invalid nested set path");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("StructSetPath failed"));
}

#[test]
fn vm_reports_struct_set_path_with_empty_path() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Struct {
                        shape: std::rc::Rc::new(StructShape {
                            name: "User".to_string(),
                            field_names: vec!["id".to_string()].into(),
                        }),
                        fields: vec![Value::Int(1)].into(),
                    }),
                    Instr::LoadConst(Value::Int(42)),
                    Instr::StructSetPath(vec![]),
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("empty set path");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("requires non-empty field path"));
}

#[test]
fn for_bytecode_has_expected_jump_shape() {
    let src = r#"
fn main() -> Int {
  let acc = 0;
  for (let i = 0; i < 3; i = i + 1) {
    acc = acc + i;
  }
  return acc;
}
"#;
    let module = compile_source(src).expect("compile");
    let main = module.functions.get("main").expect("main");
    let code = &main.code;

    let jf_idx = code
        .iter()
        .position(|i| matches!(i, Instr::JumpIfFalse(_)))
        .expect("for should emit JumpIfFalse");
    let body_jump_idx = jf_idx + 1;
    assert!(matches!(code[body_jump_idx], Instr::Jump(_)));

    let backward_jumps: Vec<_> = code
        .iter()
        .enumerate()
        .filter_map(|(idx, instr)| match instr {
            Instr::Jump(target) if *target < idx => Some((*target, idx)),
            _ => None,
        })
        .collect();
    assert!(
        backward_jumps.len() >= 2,
        "expected two backward jumps (to cond and step), got {backward_jumps:?}"
    );
}

#[test]
fn for_bytecode_patches_break_jumps() {
    let src = r#"
fn main() -> Int {
  for (;;) {
    break;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let main = module.functions.get("main").expect("main");
    assert!(!main.code.iter().any(|i| {
        matches!(i, Instr::Jump(t) if *t == usize::MAX)
            || matches!(i, Instr::JumpIfFalse(t) if *t == usize::MAX)
    }));
}

#[test]
fn runs_bool_logic_and_not_for_conditions() {
    let src = r#"
fn main() -> Int {
  let t = true;
  if (!false && t) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn short_circuit_and_skips_rhs_evaluation() {
    let src = r#"
fn main() -> Int {
  if (false && ((1 / 0) == 0)) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn short_circuit_or_skips_rhs_evaluation() {
    let src = r#"
fn main() -> Int {
  if (true || ((1 / 0) == 0)) {
    return 1;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn runs_user_defined_function_calls_with_args() {
    let src = r#"
fn add(a: Int, b: Int) -> Int {
  return a + b;
}

fn twice(x: Int) -> Int {
  return add(x, x);
}

fn main() -> Int {
  return twice(7);
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(14));
}

#[test]
fn runs_struct_literal_field_access_assignment_and_method_call() {
    let src = r#"
struct User { id: Int, name: String }
impl User {
  fn bump(self, delta: Int) -> Int {
    return self.id + delta;
  }
}
fn main() -> Int {
  let u = User { id: 7, name: "sam" };
  let before = u.id;
  u.id = before + 5;
  return u.bump(3);
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(15));
}

#[test]
fn runs_nested_struct_field_assignment() {
    let src = r#"
struct Profile { age: Int }
struct User { profile: Profile, name: String }

fn main() -> Int {
  let u = User { profile: Profile { age: 20 }, name: "a" };
  u.profile.age = 42;
  return u.profile.age;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_method_call_on_call_expression_receiver() {
    let src = r#"
struct User { id: Int }
impl User {
  fn bump(self, d: Int) -> Int { return self.id + d; }
}
fn makeUser(x: Int) -> User {
  return User { id: x };
}
fn main() -> Int {
  return makeUser(9).bump(4);
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(13));
}

#[test]
fn runs_method_call_on_index_expression_receiver() {
    let src = r#"
struct User { id: Int }
impl User {
  fn bump(self, d: Int) -> Int { return self.id + d; }
}
fn main() -> Int {
  let users: [User; 2] = [User { id: 2 }, User { id: 5 }];
  return users[1].bump(7);
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let out = Vm::run_module_main(&module).expect("vm run");
    assert_eq!(out, Value::Int(12));
}

#[test]
fn runs_io_println_and_readline_through_builtin_registry() {
    let src = r#"
import io;

fn main() -> Int {
  let name = io.readLine();
  io.println("hi " + name);
  return 0;
}
"#;
    let module = compile_source(src).expect("compile should succeed");
    let mut host = TestHost {
        output: String::new(),
        input: VecDeque::from([String::from("sam")]),
        rng_state: 0,
        ..Default::default()
    };
    let out = Vm::run_module_main_with_host(&module, &mut host).expect("vm run");
    assert_eq!(out, Value::Int(0));
    assert_eq!(host.output, "hi sam\n");
}

#[test]
fn compile_rejects_non_direct_builtin_path_depth() {
    let src = r#"
fn main() -> Int {
  a.b.c();
  return 0;
}
"#;
    let err = compile_source(src).expect_err("should reject deep path call");
    assert!(
        err.as_slice()
            .iter()
            .any(|d| d.message.contains("package.function"))
    );
}

#[test]
fn bytecode_module_roundtrip_bytes() {
    let src = r#"
fn main() -> Int {
  return 42;
}
"#;
    let module = compile_source(src).expect("compile");
    let bytes = module.to_bytes();
    let decoded = skeplib::bytecode::BytecodeModule::from_bytes(&bytes).expect("decode");
    let out = Vm::run_module_main(&decoded).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn bytecode_decode_rejects_bad_magic() {
    let bad = vec![0, 1, 2, 3, 1, 0, 0, 0];
    let err = skeplib::bytecode::BytecodeModule::from_bytes(&bad).expect_err("bad header");
    assert!(err.contains("magic"));
}

#[test]
fn bytecode_decode_rejects_unknown_version() {
    let mut bytes = b"SKBC".to_vec();
    bytes.extend_from_slice(&99u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes()); // zero functions
    let err = skeplib::bytecode::BytecodeModule::from_bytes(&bytes).expect_err("bad version");
    assert!(err.contains("Unsupported bytecode version"));
}

#[test]
fn vm_reports_stack_overflow_on_unbounded_recursion() {
    let src = r#"
fn f(x: Int) -> Int {
  return f(x + 1);
}

fn main() -> Int {
  return f(0);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("should overflow");
    assert_eq!(err.kind, VmErrorKind::StackOverflow);
}

#[test]
fn vm_stack_overflow_respects_configured_limit() {
    let src = r#"
fn f(x: Int) -> Int {
  return f(x + 1);
}
fn main() -> Int { return f(0); }
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main_with_config(
        &module,
        VmConfig {
            max_call_depth: 8,
            trace: false,
        },
    )
    .expect_err("overflow");
    assert_eq!(err.kind, VmErrorKind::StackOverflow);
    assert!(err.message.contains("8"));
}

#[test]
fn vm_reports_division_by_zero_kind() {
    let src = r#"
fn main() -> Int {
  return 10 / 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("should fail");
    assert_eq!(err.kind, VmErrorKind::DivisionByZero);
}

#[test]
fn vm_reports_division_by_zero_inside_for_loop() {
    let src = r#"
fn main() -> Int {
  let i = 0;
  for (; i < 1; i = i + 1) {
    let x = 1 / 0;
    return x;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("division by zero");
    assert_eq!(err.kind, VmErrorKind::DivisionByZero);
}

#[test]
fn vm_reports_type_mismatch_for_loop_like_bad_condition() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::JumpIfFalse(4),
                    Instr::Jump(0),
                    Instr::LoadConst(Value::Int(0)),
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
}

#[test]
fn runs_int_modulo() {
    let src = r#"
fn main() -> Int {
  return 17 % 5;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(2));
}

#[test]
fn runs_unary_plus_numeric_values() {
    let src = r#"
fn main() -> Int {
  let a = +5;
  let b: Float = +2.5;
  if (b == 2.5) {
    return a;
  }
  return 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(5));
}

#[test]
fn vm_reports_modulo_by_zero_kind() {
    let src = r#"
fn main() -> Int {
  return 10 % 0;
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("should fail");
    assert_eq!(err.kind, VmErrorKind::DivisionByZero);
}

#[test]
fn vm_reports_unknown_builtin_kind() {
    let src = r#"
fn main() -> Int {
  return pkg.work(1);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("unknown builtin");
    assert_eq!(err.kind, VmErrorKind::UnknownBuiltin);
}

#[test]
fn vm_reports_function_arity_mismatch_kind() {
    let src = r#"
fn f(x: Int) -> Int {
  return x;
}

fn main() -> Int {
  return f();
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("arity mismatch");
    assert_eq!(err.kind, VmErrorKind::ArityMismatch);
}

#[test]
fn runs_function_value_call_from_local_binding() {
    let src = r#"
fn add(a: Int, b: Int) -> Int {
  return a + b;
}

fn main() -> Int {
  let f: Fn(Int, Int) -> Int = add;
  return f(20, 22);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_function_value_call_through_parameter() {
    let src = r#"
fn apply(f: Fn(Int, Int) -> Int, x: Int, y: Int) -> Int {
  return f(x, y);
}

fn add(a: Int, b: Int) -> Int {
  return a + b;
}

fn main() -> Int {
  return apply(add, 3, 7);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(10));
}

#[test]
fn vm_reports_callvalue_type_mismatch_for_non_function_callee() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(7)),
                    Instr::CallValue { argc: 0 },
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
}

#[test]
fn vm_reports_type_mismatch_for_function_value_equality() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Function("f".to_string().into())),
                    Instr::LoadConst(Value::Function("f".to_string().into())),
                    Instr::Eq,
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
}

#[test]
fn vm_reports_type_mismatch_for_function_value_inequality() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Function("f".to_string().into())),
                    Instr::LoadConst(Value::Function("g".to_string().into())),
                    Instr::Neq,
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
}

#[test]
fn runs_function_value_call_via_grouped_callee_expr() {
    let src = r#"
fn add(a: Int, b: Int) -> Int {
  return a + b;
}

fn main() -> Int {
  return (add)(8, 9);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(17));
}

#[test]
fn runs_function_value_call_via_array_index_callee_expr() {
    let src = r#"
fn add(a: Int, b: Int) -> Int { return a + b; }
fn mul(a: Int, b: Int) -> Int { return a * b; }

fn main() -> Int {
  let ops: [Fn(Int, Int) -> Int; 2] = [add, mul];
  return ops[1](6, 7);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_non_capturing_function_literal() {
    let src = r#"
fn main() -> Int {
  let f: Fn(Int) -> Int = fn(x: Int) -> Int {
    return x + 2;
  };
  return f(40);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn compile_rejects_capturing_function_literal() {
    let src = r#"
fn main() -> Int {
  let y = 5;
  let f: Fn(Int) -> Int = fn(x: Int) -> Int {
    return x + y;
  };
  return f(1);
}
"#;
    let err = compile_err(src);
    assert_has_diag(&err, "Unknown local `y`");
}

#[test]
fn runs_immediate_function_literal_call() {
    let src = r#"
fn main() -> Int {
  return (fn(x: Int) -> Int { return x + 1; })(41);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_function_literal_passed_as_argument() {
    let src = r#"
fn apply(f: Fn(Int) -> Int, x: Int) -> Int {
  return f(x);
}

fn main() -> Int {
  return apply(fn(x: Int) -> Int { return x + 2; }, 40);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_function_returning_function_literal_and_chained_call() {
    let src = r#"
fn makeInc() -> Fn(Int) -> Int {
  return fn(x: Int) -> Int { return x + 1; };
}

fn main() -> Int {
  return makeInc()(41);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_function_type_in_struct_field_and_call_via_grouping() {
    let src = r#"
struct Op {
  apply: Fn(Int, Int) -> Int
}

fn add(a: Int, b: Int) -> Int { return a + b; }

fn main() -> Int {
  let op: Op = Op { apply: add };
  return (op.apply)(20, 22);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn runs_array_of_functions_returned_from_function() {
    let src = r#"
fn add(a: Int, b: Int) -> Int { return a + b; }
fn mul(a: Int, b: Int) -> Int { return a * b; }

fn makeOps() -> [Fn(Int, Int) -> Int; 2] {
  return [add, mul];
}

fn main() -> Int {
  let ops = makeOps();
  return ops[0](2, 3) + ops[1](2, 3);
}
"#;
    let module = compile_source(src).expect("compile");
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(11));
}

#[test]
fn vm_reports_unknown_method_for_method_style_call_on_function_field() {
    let src = r#"
struct Op {
  apply: Fn(Int, Int) -> Int
}

fn add(a: Int, b: Int) -> Int { return a + b; }

fn main() -> Int {
  let op: Op = Op { apply: add };
  return op.apply(1, 2);
}
"#;
    let module = compile_source(src).expect("compile");
    let err = Vm::run_module_main(&module).expect_err("unknown method");
    assert_eq!(err.kind, VmErrorKind::UnknownFunction);
    assert!(
        err.message
            .contains("Unknown method `apply` on struct `Op`")
    );
}

#[test]
fn bytecode_roundtrip_preserves_function_value_and_callvalue_instr() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![
            (
                "inc".to_string(),
                FunctionChunk {
                    name: "inc".to_string(),
                    code: vec![
                        Instr::LoadLocal(0),
                        Instr::LoadConst(Value::Int(1)),
                        Instr::Add,
                        Instr::Return,
                    ],
                    locals_count: 1,
                    param_count: 1,
                },
            ),
            (
                "main".to_string(),
                FunctionChunk {
                    name: "main".to_string(),
                    code: vec![
                        Instr::LoadConst(Value::Function("inc".to_string().into())),
                        Instr::LoadConst(Value::Int(41)),
                        Instr::CallValue { argc: 1 },
                        Instr::Return,
                    ],
                    locals_count: 0,
                    param_count: 0,
                },
            ),
        ]
        .into_iter()
        .collect(),
    };
    let bytes = module.to_bytes();
    let decoded = BytecodeModule::from_bytes(&bytes).expect("decode");
    let out = Vm::run_module_main(&decoded).expect("run");
    assert_eq!(out, Value::Int(42));
}

#[test]
fn vm_supports_string_concat_and_equality() {
    let src = r#"
fn main() -> Int {
  if ("ab" + "cd" == "abcd") {
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
fn bytecode_decode_rejects_truncated_payload() {
    let src = r#"
fn main() -> Int { return 1; }
"#;
    let module = compile_source(src).expect("compile");
    let mut bytes = module.to_bytes();
    bytes.truncate(bytes.len().saturating_sub(3));
    let err = BytecodeModule::from_bytes(&bytes).expect_err("truncate should fail");
    assert!(err.contains("Unexpected EOF"));
}

#[test]
fn vm_reports_stack_underflow_for_invalid_program() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![Instr::Pop, Instr::Return],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("stack underflow");
    assert_eq!(err.kind, VmErrorKind::StackUnderflow);
}

#[test]
fn vm_reports_type_mismatch_for_bad_jump_condition() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::JumpIfFalse(4),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::Return,
                    Instr::LoadConst(Value::Int(0)),
                    Instr::Return,
                ],
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let err = Vm::run_module_main(&module).expect_err("type mismatch");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("main@"));
}
