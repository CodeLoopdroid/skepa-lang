use skeplib::ir::{self, IrValue, PrettyIr};

#[test]
fn dce_eliminates_dead_pure_temps() {
    let source = r#"
fn main() -> Int {
  let x = 1;
  let y = x + 41;
  let dead = y + 99;
  return y;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = ir::IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(42));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("Int(141)"));
}

#[test]
fn dce_keeps_effectful_runtime_operations() {
    let source = r#"
import io;

fn main() -> Int {
  io.print("ok");
  return 1;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("CallBuiltin"));
}
