use skeplib::ir::{self, IrValue, PrettyIr};

#[test]
fn cfg_simplify_removes_constant_branch_shape() {
    let source = r#"
fn main() -> Int {
  if (1 < 2) {
    return 7;
  }
  return 9;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = ir::IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(7));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("Branch(BranchTerminator"));
}

#[test]
fn cfg_simplify_preserves_loop_control_flow_semantics() {
    let source = r#"
fn main() -> Int {
  let acc = 0;
  for (let i = 0; i < 6; i = i + 1) {
    if (i == 4) {
      break;
    }
    acc = acc + i;
  }
  return acc;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = ir::IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(6));
}
