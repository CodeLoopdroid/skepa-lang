use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use skeplib::ir::{
    self, BasicBlock, BlockId, FieldRef, FunctionId, Instr, IrFunction, IrInterpError,
    IrInterpreter, IrLocal, IrProgram, IrStruct, IrTemp, IrType, IrValue, IrVerifier, PrettyIr,
    StructField, StructId, TempId, Terminator,
};
mod common;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedErrorKind {
    DivisionByZero,
    IndexOutOfBounds,
    TypeMismatch,
}

fn assert_native_and_ir_accept_same_int_source(source: &str, expected: i32) {
    let code = common::native_run_exit_code_ok(source);
    assert_eq!(code, expected);
    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    assert!(
        !program.functions.is_empty(),
        "IR lowering should emit at least one function"
    );
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::Int(i64::from(expected)));
}

fn assert_ir_rejects_source(source: &str, expected: ExpectedErrorKind) {
    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let ir_err = IrInterpreter::new(&program)
        .run_main()
        .expect_err("IR interpreter should fail");
    let ir_kind = match ir_err {
        IrInterpError::DivisionByZero => ExpectedErrorKind::DivisionByZero,
        IrInterpError::IndexOutOfBounds => ExpectedErrorKind::IndexOutOfBounds,
        IrInterpError::TypeMismatch(_) => ExpectedErrorKind::TypeMismatch,
        other => panic!("unexpected IR error kind in comparison test: {other:?}"),
    };
    assert_eq!(ir_kind, expected);
}

#[test]
fn lower_simple_function_to_ir() {
    let source = r#"
fn add_loop(n: Int) -> Int {
  let i = 0;
  let acc = 0;
  while (i < n) {
    acc = acc + i;
    i = i + 1;
  }
  return acc;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    assert_eq!(program.functions.len(), 1);
    let func = &program.functions[0];
    assert_eq!(func.name, "add_loop");
    assert!(func.blocks.len() >= 3);
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("fn add_loop"));
    assert!(printed.contains("while_cond") || printed.contains("Branch"));
}

#[test]
fn compile_source_applies_constant_folding_and_branch_simplification() {
    let source = r#"
fn main() -> Int {
  let x = 1 + 2;
  if (true) {
    return x;
  }
  return 99;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(3));
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("Jump(BlockId("));
    assert!(!printed.contains("Branch(BranchTerminator"));
}

#[test]
fn compile_source_applies_copy_propagation() {
    let source = r#"
fn main() -> Int {
  let x = 1;
  let y = x;
  let z = y;
  return z + 2;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(3));
    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("Copy {"));
}

#[test]
fn compile_source_eliminates_dead_pure_temps() {
    let source = r#"
fn main() -> Int {
  let x = 1 + 2;
  let y = x + 10;
  let z = y + 20;
  return x;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(3));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("value: Int(13)"));
    assert!(!printed.contains("value: Int(33)"));
}

#[test]
fn compile_source_simplifies_cfg_after_constant_branching() {
    let source = r#"
fn main() -> Int {
  if (true) {
    return 7;
  } else {
    return 9;
  }
  return 0;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(7));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("Branch(BranchTerminator"));
}

#[test]
fn compile_source_inlines_trivial_direct_calls() {
    let source = r#"
fn step(x: Int) -> Int {
  return x + 1;
}

fn main() -> Int {
  return step(41);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(42));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("CallDirect"));
}

#[test]
fn compile_source_inlines_trivial_struct_methods() {
    let source = r#"
struct Pair {
  a: Int,
  b: Int
}

impl Pair {
  fn mix(self, x: Int) -> Int {
    return self.a + x + self.b;
  }
}

fn main() -> Int {
  let p = Pair { a: 10, b: 5 };
  return p.mix(7);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(22));

    let printed = PrettyIr::new(&program).to_string();
    assert!(!printed.contains("CallDirect"));
}

#[test]
fn optimize_program_eliminates_overwritten_local_stores() {
    let local = IrLocal {
        id: skeplib::ir::LocalId(0),
        name: "x".into(),
        ty: IrType::Int,
    };
    let temp = IrTemp {
        id: TempId(0),
        ty: IrType::Int,
    };
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: vec![local.clone()],
        temps: vec![temp],
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![
                Instr::StoreLocal {
                    local: local.id,
                    ty: IrType::Int,
                    value: ir::Operand::Const(ir::ConstValue::Int(1)),
                },
                Instr::StoreLocal {
                    local: local.id,
                    ty: IrType::Int,
                    value: ir::Operand::Const(ir::ConstValue::Int(2)),
                },
                Instr::LoadLocal {
                    dst: TempId(0),
                    ty: IrType::Int,
                    local: local.id,
                },
            ],
            terminator: Terminator::Return(Some(ir::Operand::Temp(TempId(0)))),
        }],
    };
    let mut program = IrProgram {
        structs: Vec::new(),
        globals: Vec::new(),
        functions: vec![func],
        module_init: None,
    };

    ir::opt::optimize_program(&mut program);

    let main = &program.functions[0];
    let store_count = main.blocks[0]
        .instrs
        .iter()
        .filter(|instr| matches!(instr, Instr::StoreLocal { .. }))
        .count();
    assert_eq!(store_count, 1);
}

#[test]
fn optimize_program_simplifies_empty_loop_body_blocks() {
    let cond = IrLocal {
        id: skeplib::ir::LocalId(0),
        name: "cond".into(),
        ty: IrType::Bool,
    };
    let mut program = IrProgram {
        structs: Vec::new(),
        globals: Vec::new(),
        functions: vec![IrFunction {
            id: FunctionId(0),
            name: "main".into(),
            params: Vec::new(),
            locals: vec![cond],
            temps: Vec::new(),
            ret_ty: IrType::Int,
            entry: BlockId(0),
            blocks: vec![
                BasicBlock {
                    id: BlockId(0),
                    name: "entry".into(),
                    instrs: Vec::new(),
                    terminator: Terminator::Jump(BlockId(1)),
                },
                BasicBlock {
                    id: BlockId(1),
                    name: "while_cond".into(),
                    instrs: Vec::new(),
                    terminator: Terminator::Branch(ir::BranchTerminator {
                        cond: ir::Operand::Local(skeplib::ir::LocalId(0)),
                        then_block: BlockId(2),
                        else_block: BlockId(3),
                    }),
                },
                BasicBlock {
                    id: BlockId(2),
                    name: "while_body".into(),
                    instrs: Vec::new(),
                    terminator: Terminator::Jump(BlockId(1)),
                },
                BasicBlock {
                    id: BlockId(3),
                    name: "while_exit".into(),
                    instrs: Vec::new(),
                    terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(
                        7,
                    )))),
                },
            ],
        }],
        module_init: None,
    };

    ir::opt::optimize_program(&mut program);

    let main = &program.functions[0];
    assert!(!main.blocks.iter().any(|block| block.name == "while_body"));
}

#[test]
fn optimize_program_simplifies_loop_invariant_const_usage() {
    let cond = IrLocal {
        id: skeplib::ir::LocalId(0),
        name: "cond".into(),
        ty: IrType::Bool,
    };
    let sink = IrLocal {
        id: skeplib::ir::LocalId(1),
        name: "sink".into(),
        ty: IrType::Int,
    };
    let temp = IrTemp {
        id: TempId(0),
        ty: IrType::Int,
    };
    let exit_temp = IrTemp {
        id: TempId(1),
        ty: IrType::Int,
    };
    let mut program = IrProgram {
        structs: Vec::new(),
        globals: Vec::new(),
        functions: vec![IrFunction {
            id: FunctionId(0),
            name: "main".into(),
            params: Vec::new(),
            locals: vec![cond, sink.clone()],
            temps: vec![temp, exit_temp],
            ret_ty: IrType::Int,
            entry: BlockId(0),
            blocks: vec![
                BasicBlock {
                    id: BlockId(0),
                    name: "entry".into(),
                    instrs: Vec::new(),
                    terminator: Terminator::Jump(BlockId(1)),
                },
                BasicBlock {
                    id: BlockId(1),
                    name: "while_cond".into(),
                    instrs: Vec::new(),
                    terminator: Terminator::Branch(ir::BranchTerminator {
                        cond: ir::Operand::Local(skeplib::ir::LocalId(0)),
                        then_block: BlockId(2),
                        else_block: BlockId(3),
                    }),
                },
                BasicBlock {
                    id: BlockId(2),
                    name: "while_body".into(),
                    instrs: vec![
                        Instr::Const {
                            dst: TempId(0),
                            ty: IrType::Int,
                            value: ir::ConstValue::Int(1),
                        },
                        Instr::StoreLocal {
                            local: sink.id,
                            ty: IrType::Int,
                            value: ir::Operand::Temp(TempId(0)),
                        },
                    ],
                    terminator: Terminator::Jump(BlockId(1)),
                },
                BasicBlock {
                    id: BlockId(3),
                    name: "while_exit".into(),
                    instrs: vec![Instr::LoadLocal {
                        dst: TempId(1),
                        ty: IrType::Int,
                        local: sink.id,
                    }],
                    terminator: Terminator::Return(Some(ir::Operand::Temp(TempId(1)))),
                },
            ],
        }],
        module_init: None,
    };

    ir::opt::optimize_program(&mut program);

    let main = &program.functions[0];
    let entry = main
        .blocks
        .iter()
        .find(|block| block.name == "entry")
        .expect("entry block should exist");
    let while_body = main
        .blocks
        .iter()
        .find(|block| block.name == "while_body")
        .expect("while body should still exist");
    assert!(
        !while_body
            .instrs
            .iter()
            .any(|instr| matches!(instr, Instr::Const { .. }))
    );
    assert!(
        entry
            .instrs
            .iter()
            .all(|instr| !matches!(instr, Instr::StoreLocal { .. }))
    );
}

#[test]
fn optimize_program_applies_strength_reduction_identities() {
    let local = IrLocal {
        id: skeplib::ir::LocalId(0),
        name: "x".into(),
        ty: IrType::Int,
    };
    let temps = vec![
        IrTemp {
            id: TempId(0),
            ty: IrType::Int,
        },
        IrTemp {
            id: TempId(1),
            ty: IrType::Int,
        },
    ];
    let mut program = IrProgram {
        structs: Vec::new(),
        globals: Vec::new(),
        functions: vec![IrFunction {
            id: FunctionId(0),
            name: "main".into(),
            params: Vec::new(),
            locals: vec![local.clone()],
            temps,
            ret_ty: IrType::Int,
            entry: BlockId(0),
            blocks: vec![BasicBlock {
                id: BlockId(0),
                name: "entry".into(),
                instrs: vec![
                    Instr::Binary {
                        dst: TempId(0),
                        ty: IrType::Int,
                        op: ir::BinaryOp::Mul,
                        left: ir::Operand::Local(local.id),
                        right: ir::Operand::Const(ir::ConstValue::Int(2)),
                    },
                    Instr::Binary {
                        dst: TempId(1),
                        ty: IrType::Int,
                        op: ir::BinaryOp::Mod,
                        left: ir::Operand::Local(local.id),
                        right: ir::Operand::Const(ir::ConstValue::Int(1)),
                    },
                ],
                terminator: Terminator::Return(Some(ir::Operand::Temp(TempId(0)))),
            }],
        }],
        module_init: None,
    };

    ir::opt::optimize_program(&mut program);

    let main = &program.functions[0];
    assert!(main.blocks[0].instrs.iter().any(|instr| matches!(
        instr,
        Instr::Binary {
            op: ir::BinaryOp::Add,
            ..
        }
    )));
    assert!(!main.blocks[0].instrs.iter().any(|instr| matches!(
        instr,
        Instr::Binary {
            op: ir::BinaryOp::Mod,
            ..
        }
    )));
}

#[test]
fn verifier_rejects_unknown_jump_target() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: Vec::new(),
            terminator: Terminator::Jump(BlockId(99)),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(err, ir::IrVerifyError::UnknownBlockTarget { .. }));
}

#[test]
fn verifier_rejects_unknown_direct_call_target() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![ir::Instr::CallDirect {
                dst: None,
                function: FunctionId(77),
                args: Vec::new(),
                ret_ty: IrType::Int,
            }],
            terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(
        err,
        ir::IrVerifyError::UnknownFunctionTarget { .. }
    ));
}

#[test]
fn verifier_rejects_duplicate_block_ids() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![
            BasicBlock {
                id: BlockId(0),
                name: "entry".into(),
                instrs: Vec::new(),
                terminator: Terminator::Jump(BlockId(0)),
            },
            BasicBlock {
                id: BlockId(0),
                name: "duplicate".into(),
                instrs: Vec::new(),
                terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
            },
        ],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(err, ir::IrVerifyError::DuplicateBlockId { .. }));
}

#[test]
fn verifier_rejects_unknown_struct_field_ref() {
    let strukt = IrStruct {
        id: StructId(0),
        name: "Pair".into(),
        fields: vec![StructField {
            name: "a".into(),
            ty: IrType::Int,
        }],
    };
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: vec![IrLocal {
            id: ir::LocalId(0),
            name: "pair".into(),
            ty: IrType::Named("Pair".into()),
        }],
        temps: vec![IrTemp {
            id: TempId(0),
            ty: IrType::Int,
        }],
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![Instr::StructGet {
                dst: TempId(0),
                ty: IrType::Int,
                base: ir::Operand::Local(ir::LocalId(0)),
                field: FieldRef {
                    index: 1,
                    name: "b".into(),
                },
            }],
            terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: vec![strukt],
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(
        err,
        ir::IrVerifyError::UnknownField { ref field, .. } if field == "b"
    ));
}

#[test]
fn verifier_rejects_unknown_temp_operand() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![Instr::Copy {
                dst: TempId(1),
                ty: IrType::Int,
                src: ir::Operand::Temp(TempId(99)),
            }],
            terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(err, ir::IrVerifyError::UnknownTemp { .. }));
}

#[test]
fn verifier_rejects_unknown_local_operand() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: vec![IrTemp {
            id: TempId(0),
            ty: IrType::Int,
        }],
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![Instr::StoreLocal {
                local: ir::LocalId(77),
                ty: IrType::Int,
                value: ir::Operand::Const(ir::ConstValue::Int(1)),
            }],
            terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(err, ir::IrVerifyError::UnknownLocal { .. }));
}

#[test]
fn verifier_rejects_unknown_global_operand() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: vec![IrTemp {
            id: TempId(0),
            ty: IrType::Int,
        }],
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![Instr::LoadGlobal {
                dst: TempId(0),
                ty: IrType::Int,
                global: ir::GlobalId(42),
            }],
            terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(err, ir::IrVerifyError::UnknownGlobal));
}

#[test]
fn verifier_rejects_unknown_module_init_target() {
    let program = IrProgram {
        functions: vec![IrFunction {
            id: FunctionId(0),
            name: "main".into(),
            params: Vec::new(),
            locals: Vec::new(),
            temps: Vec::new(),
            ret_ty: IrType::Int,
            entry: BlockId(0),
            blocks: vec![BasicBlock {
                id: BlockId(0),
                name: "entry".into(),
                instrs: Vec::new(),
                terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
            }],
        }],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: Some(ir::IrModuleInit {
            function: FunctionId(99),
        }),
    };

    let err = IrVerifier::verify_program(&program).expect_err("verifier should fail");
    assert!(matches!(err, ir::IrVerifyError::UnknownModuleInitFunction));
}

#[test]
fn interpreter_rejects_non_bool_branch_condition() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![
            BasicBlock {
                id: BlockId(0),
                name: "entry".into(),
                instrs: Vec::new(),
                terminator: ir::Terminator::Branch(ir::BranchTerminator {
                    cond: ir::Operand::Const(ir::ConstValue::Int(1)),
                    then_block: BlockId(1),
                    else_block: BlockId(2),
                }),
            },
            BasicBlock {
                id: BlockId(1),
                name: "then".into(),
                instrs: Vec::new(),
                terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(1)))),
            },
            BasicBlock {
                id: BlockId(2),
                name: "else".into(),
                instrs: Vec::new(),
                terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
            },
        ],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrInterpreter::new(&program)
        .run_main()
        .expect_err("interpreter should reject non-bool branch conditions");
    assert!(matches!(
        err,
        IrInterpError::TypeMismatch("branch condition must be bool")
    ));
}

#[test]
fn interpreter_rejects_indirect_call_on_non_closure() {
    let func = IrFunction {
        id: FunctionId(0),
        name: "main".into(),
        params: Vec::new(),
        locals: Vec::new(),
        temps: Vec::new(),
        ret_ty: IrType::Int,
        entry: BlockId(0),
        blocks: vec![BasicBlock {
            id: BlockId(0),
            name: "entry".into(),
            instrs: vec![Instr::CallIndirect {
                dst: None,
                ret_ty: IrType::Int,
                callee: ir::Operand::Const(ir::ConstValue::Int(7)),
                args: Vec::new(),
            }],
            terminator: Terminator::Return(Some(ir::Operand::Const(ir::ConstValue::Int(0)))),
        }],
    };
    let program = IrProgram {
        functions: vec![func],
        globals: Vec::new(),
        structs: Vec::new(),
        module_init: None,
    };

    let err = IrInterpreter::new(&program)
        .run_main()
        .expect_err("interpreter should reject non-closure indirect calls");
    assert!(matches!(
        err,
        IrInterpError::TypeMismatch("indirect call on non-closure")
    ));
}

#[test]
fn lower_globals_and_direct_calls_to_ir() {
    let source = r#"
let seed: Int = 41;

fn inc(x: Int) -> Int {
  return x + 1;
}

fn main() -> Int {
  let x = inc(seed);
  let y = str.len("abc");
  return x + y;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    assert_eq!(program.globals.len(), 1);
    assert!(program.module_init.is_some());
    assert!(program.functions.iter().any(|f| f.name == "__globals_init"));
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(45));

    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("CallBuiltin"));
    assert!(printed.contains("StoreGlobal"));
}

#[test]
fn lower_static_array_ops_to_ir() {
    let source = r#"
fn main() -> Int {
  let arr: [Int; 4] = [0; 4];
  arr[1] = 7;
  arr[2] = arr[1] + 3;
  return arr[2];
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("MakeArrayRepeat"));
    assert!(printed.contains("ArraySet"));
    assert!(printed.contains("ArrayGet"));
}

#[test]
fn lower_struct_literal_and_field_ops_to_ir() {
    let source = r#"
struct Pair {
  a: Int,
  b: Int
}

fn main() -> Int {
  let p = Pair { a: 2, b: 3 };
  p.a = 7;
  return p.a + p.b;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    assert_eq!(program.structs.len(), 1);
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("MakeStruct"));
    assert!(printed.contains("StructSet"));
    assert!(printed.contains("StructGet"));
}

#[test]
fn lower_short_circuit_bool_ops_to_ir() {
    let source = r#"
fn main() -> Bool {
  let a = true;
  let b = false;
  return (a && b) || a;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("sc_rhs"));
    assert!(printed.contains("sc_short"));
    assert!(printed.contains("Branch"));
}

#[test]
fn lower_named_function_values_and_indirect_calls_to_ir() {
    let source = r#"
fn inc(x: Int) -> Int {
  return x + 1;
}

fn main() -> Int {
  let f = inc;
  return f(4);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("MakeClosure"));
    assert!(printed.contains("CallIndirect"));
}

#[test]
fn lower_non_capturing_function_literals_to_ir() {
    let source = r#"
fn main() -> Int {
  let f = fn(x: Int) -> Int {
    return x + 2;
  };
  return f(5);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    assert!(
        program
            .functions
            .iter()
            .any(|func| func.name.starts_with("__fn_lit_"))
    );
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("MakeClosure"));
    assert!(printed.contains("CallIndirect"));
}

#[test]
fn lower_vec_ops_to_ir() {
    let source = r#"
fn main() -> Int {
  let xs: Vec[Int] = vec.new();
  vec.push(xs, 10);
  vec.push(xs, 20);
  vec.set(xs, 1, 30);
  let first = vec.get(xs, 0);
  let removed = vec.delete(xs, 1);
  return first + removed + vec.len(xs);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("VecNew"));
    assert!(printed.contains("VecPush"));
    assert!(printed.contains("VecSet"));
    assert!(printed.contains("VecGet"));
    assert!(printed.contains("VecDelete"));
    assert!(printed.contains("VecLen"));
}

#[test]
fn lower_struct_method_calls_to_ir() {
    let source = r#"
struct Pair {
  a: Int,
  b: Int
}

impl Pair {
  fn mix(self, x: Int) -> Int {
    return self.a + self.b + x;
  }
}

fn main() -> Int {
  let p = Pair { a: 2, b: 3 };
  return p.mix(4);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    assert!(
        program
            .functions
            .iter()
            .any(|func| func.name == "Pair::mix")
    );
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run optimized source");
    assert_eq!(value, IrValue::Int(9));
    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("fn Pair::mix"));
    let main_fn = program
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main should be lowered");
    assert!(!main_fn.blocks.iter().any(|block| {
        block
            .instrs
            .iter()
            .any(|instr| matches!(instr, ir::Instr::CallBuiltin { .. }))
    }));
}

#[test]
fn lower_project_entry_to_ir() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for temp name")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("skepa_ir_project_{unique}"));
    fs::create_dir_all(&root).expect("temp project dir should be created");

    let entry = root.join("main.sk");
    fs::write(
        root.join("util.sk"),
        r#"
export { inc };

fn inc(x: Int) -> Int {
  return x + 1;
}
"#,
    )
    .expect("util module should be written");
    fs::write(
        &entry,
        r#"
from util import inc;

fn main() -> Int {
  return inc(41);
}
"#,
    )
    .expect("entry module should be written");

    let program =
        ir::lowering::compile_project_entry(&entry).expect("project IR lowering should succeed");
    assert!(
        program
            .functions
            .iter()
            .any(|func| func.name == "util::inc")
    );
    assert!(program.functions.iter().any(|func| func.name == "main"));

    let printed = PrettyIr::new(&program).to_string();
    assert!(printed.contains("fn util::inc"));
    assert!(printed.contains("fn main"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn interpret_project_entry_ir_for_cross_module_call_flow() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for temp name")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("skepa_ir_exec_project_{unique}"));
    fs::create_dir_all(&root).expect("temp project dir should be created");

    let entry = root.join("main.sk");
    fs::write(
        root.join("math.sk"),
        r#"
export { bump, seed_value };

let seed: Int = 9;

fn bump(x: Int) -> Int {
  return x + 2;
}

fn seed_value() -> Int {
  return seed;
}
"#,
    )
    .expect("math module should be written");
    fs::write(
        &entry,
        r#"
from math import bump, seed_value;

fn main() -> Int {
  return bump(seed_value()) + 1;
}
"#,
    )
    .expect("entry module should be written");

    let program =
        ir::lowering::compile_project_entry(&entry).expect("project IR lowering should succeed");
    let ir_value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run project");
    assert_eq!(ir_value, IrValue::Int(12));
    assert_eq!(common::native_run_project_exit_code_ok(&entry), 12);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn interpret_project_entry_ir_for_cross_module_struct_method_flow() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for temp name")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("skepa_ir_struct_project_{unique}"));
    fs::create_dir_all(&root).expect("temp project dir should be created");

    let entry = root.join("main.sk");
    fs::write(
        root.join("pair.sk"),
        r#"
export { Pair, make };

struct Pair {
  a: Int,
  b: Int
}

impl Pair {
  fn mix(self, x: Int) -> Int {
    return self.a + self.b + x;
  }
}

fn make() -> Pair {
  return Pair { a: 4, b: 6 };
}
"#,
    )
    .expect("pair module should be written");
    fs::write(
        &entry,
        r#"
from pair import Pair, make;

fn main() -> Int {
  let p = make();
  p.a = 7;
  return p.mix(5);
}
"#,
    )
    .expect("entry module should be written");

    let program =
        ir::lowering::compile_project_entry(&entry).expect("project IR lowering should succeed");
    let ir_value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run project");
    assert_eq!(ir_value, IrValue::Int(18));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn ir_accepts_same_project_with_globals_and_string_builtins() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for temp name")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("skepa_ir_string_project_{unique}"));
    fs::create_dir_all(&root).expect("temp project dir should be created");

    let entry = root.join("main.sk");
    fs::write(
        root.join("words.sk"),
        r#"
export { get_base, get_needle, bonus };

let base: String = "skepa-language-benchmark";
let needle: String = "bench";
let extra: Int = 1;

fn get_base() -> String {
  return base;
}

fn get_needle() -> String {
  return needle;
}

fn bonus() -> Int {
  return extra;
}
"#,
    )
    .expect("words module should be written");
    fs::write(
        &entry,
        r#"
import str;
from words import get_base, get_needle, bonus;

fn main() -> Int {
  let base = get_base();
  let needle = get_needle();
  let total = str.len(base) + str.indexOf(base, needle);
  let cut = str.slice(base, 6, 14);
  if (str.contains(cut, "language")) {
    return total + bonus();
  }
  return total;
}
"#,
    )
    .expect("entry module should be written");

    let program =
        ir::lowering::compile_project_entry(&entry).expect("project IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run project");
    assert_eq!(value, IrValue::Int(40));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn ir_rejects_division_by_zero_source() {
    let source = r#"
fn main() -> Int {
  return 8 / 0;
}
"#;

    assert_ir_rejects_source(source, ExpectedErrorKind::DivisionByZero);
}

#[test]
fn ir_rejects_array_oob_source() {
    let source = r#"
fn main() -> Int {
  let arr: [Int; 2] = [1; 2];
  return arr[3];
}
"#;

    assert_ir_rejects_source(source, ExpectedErrorKind::IndexOutOfBounds);
}

#[test]
fn ir_rejects_string_slice_oob_source() {
    let source = r#"
import str;

fn main() -> String {
  return str.slice("abc", 0, 99);
}
"#;

    assert_ir_rejects_source(source, ExpectedErrorKind::IndexOutOfBounds);
}

#[test]
fn native_and_ir_accept_same_core_control_flow_source() {
    let source = r#"
fn main() -> Int {
  let i = 0;
  let acc = 0;
  while (i < 6) {
    acc = acc + i;
    i = i + 1;
  }
  return acc;
}
"#;

    assert_native_and_ir_accept_same_int_source(source, 15);
}

#[test]
fn native_and_ir_accept_same_for_loop_source() {
    let source = r#"
fn main() -> Int {
  let acc = 0;
  for (let i = 0; i < 8; i = i + 1) {
    if (i == 2) {
      continue;
    }
    if (i == 6) {
      break;
    }
    acc = acc + i;
  }
  return acc;
}
"#;

    assert_native_and_ir_accept_same_int_source(source, 13);
}

#[test]
fn ir_accepts_same_struct_and_method_source() {
    let source = r#"
struct Pair {
  a: Int,
  b: Int
}

impl Pair {
  fn mix(self, x: Int) -> Int {
    return self.a + self.b + x;
  }
}

fn main() -> Int {
  let p = Pair { a: 10, b: 5 };
  p.a = 7;
  return p.mix(4);
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::Int(16));
}

#[test]
fn ir_accepts_same_array_and_vec_source() {
    let source = r#"
fn main() -> Int {
  let arr: [Int; 3] = [1; 3];
  arr[1] = 5;
  let xs: Vec[Int] = vec.new();
  vec.push(xs, arr[0]);
  vec.push(xs, arr[1]);
  return vec.get(xs, 0) + vec.get(xs, 1) + arr[2];
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::Int(7));
}

#[test]
fn ir_accepts_same_string_builtin_source() {
    let source = r#"
fn main() -> Int {
  let s = "skepa-language-benchmark";
  let total = 0;
  total = total + str.len(s);
  total = total + str.indexOf(s, "bench");
  let cut = str.slice(s, 6, 14);
  if (str.contains(cut, "language")) {
    total = total + 1;
  }
  return total;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::Int(40));
}

#[test]
fn ir_accepts_same_float_source() {
    let source = r#"
fn main() -> Float {
  let x = 1.5;
  let y = 2.0;
  return (x + y) * 2.0;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::Float(7.0));
}

#[test]
fn ir_accepts_same_bool_short_circuit_source() {
    let source = r#"
fn main() -> Bool {
  let a = true;
  let b = false;
  return (a && b) || !b;
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::Bool(true));
}

#[test]
fn ir_accepts_same_string_builtin_output_source() {
    let source = r#"
fn main() -> String {
  let s = "alpha-beta";
  let cut = str.slice(s, 0, 5);
  if (str.contains(s, "beta")) {
    return cut + "-ok";
  }
  return "bad";
}
"#;

    let program = ir::lowering::compile_source(source).expect("IR lowering should succeed");
    let value = IrInterpreter::new(&program)
        .run_main()
        .expect("IR interpreter should run source");
    assert_eq!(value, IrValue::String("alpha-ok".into()));
}
