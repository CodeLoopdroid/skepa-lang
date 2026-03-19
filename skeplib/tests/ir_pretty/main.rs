use skeplib::ir::{self, PrettyIr};

#[test]
fn pretty_ir_includes_structs_globals_and_module_init() {
    let source = r#"
struct Pair {
  a: Int,
  b: Int
}

let seed = 7;

fn main() -> Int {
  let p = Pair { a: seed, b: 2 };
  return p.a + p.b;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let printed = PrettyIr::new(&program).to_string();

    assert!(printed.contains("structs {"));
    assert!(printed.contains("Pair(a: Int, b: Int)"));
    assert!(printed.contains("globals {"));
    assert!(printed.contains("seed"));
    assert!(printed.contains("module_init FunctionId("));
    assert!(printed.contains("fn __globals_init -> Void {"));
    assert!(printed.contains("fn main -> Int {"));
}
