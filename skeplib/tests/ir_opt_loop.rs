use skeplib::ir::{self, IrValue, PrettyIr};

#[test]
fn loop_passes_simplify_nested_loops_without_changing_result() {
    let source = r#"
fn main() -> Int {
  let outer = 0;
  let total = 0;
  while (outer < 8) {
    let inner = 0;
    while (inner < 4) {
      total = total + outer + inner;
      inner = inner + 1;
    }
    outer = outer + 1;
  }
  return total;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = ir::IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(160));
}

#[test]
fn optimizer_reaches_a_stable_fixed_point_for_loop_heavy_programs() {
    let source = r#"
fn main() -> Int {
  let i = 0;
  let acc = 0;
  while (i < 5) {
    acc = acc + (1 * 2);
    i = i + 1;
  }
  return acc;
}
"#;

    let mut program =
        ir::lowering::compile_source_unoptimized(source).expect("IR lowering should succeed");
    ir::opt::optimize_program(&mut program);
    let first = PrettyIr::new(&program).to_string();
    ir::opt::optimize_program(&mut program);
    let second = PrettyIr::new(&program).to_string();
    assert_eq!(first, second);
}
