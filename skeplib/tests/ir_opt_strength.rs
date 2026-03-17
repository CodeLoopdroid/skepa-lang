use skeplib::ir::{self, IrValue, PrettyIr};

#[test]
fn strength_reduce_rewrites_arithmetic_identities() {
    let source = r#"
fn main() -> Int {
  let x = 9 * 2;
  let y = x + 0;
  let z = y / 1;
  return z - 0;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = ir::IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(18));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("Mul"));
    assert!(!printed.contains("Div"));
}

#[test]
fn strength_reduce_preserves_array_update_recurrence_semantics() {
    let source = r#"
fn main() -> Int {
  let arr: [Int; 4] = [0; 4];
  let i = 0;
  while (i < 8) {
    let idx = i % 4;
    arr[idx] = arr[idx] + ((i * 2) / 1);
    i = i + 1;
  }
  return arr[0] + arr[1] + arr[2] + arr[3];
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = ir::IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(56));
}
