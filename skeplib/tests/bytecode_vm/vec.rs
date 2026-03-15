use super::*;

#[test]
fn vm_vec_new_and_len_runtime_builtins_work() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "len".to_string(),
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
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(0));
}

#[test]
fn vm_vec_push_get_set_delete_supports_in_place_mutation_and_shift() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::StoreLocal(0),
                    // push 10
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(10)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    // push 20
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(20)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    // push 30
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(30)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    // set index 1 = 99
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::LoadConst(Value::Int(99)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "set".to_string(),
                        argc: 3,
                    },
                    Instr::Pop,
                    // delete index 1, store removed
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "delete".to_string(),
                        argc: 2,
                    },
                    Instr::StoreLocal(1),
                    // Return removed + len + get(0) + get(1)
                    Instr::LoadLocal(1),
                    Instr::LoadLocal(0),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "len".to_string(),
                        argc: 1,
                    },
                    Instr::Add,
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(0)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "get".to_string(),
                        argc: 2,
                    },
                    Instr::Add,
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "get".to_string(),
                        argc: 2,
                    },
                    Instr::Add,
                    Instr::Return,
                ],
                locals_count: 2,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let out = Vm::run_module_main(&module).expect("run");
    // removed=99, len=2, remaining=[10,30]
    assert_eq!(out, Value::Int(141));
}

#[test]
fn vm_vec_aliasing_uses_shared_handle_semantics() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::StoreLocal(0),
                    Instr::LoadLocal(0),
                    Instr::StoreLocal(1),
                    Instr::LoadLocal(1),
                    Instr::LoadConst(Value::Int(7)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    Instr::LoadLocal(0),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "len".to_string(),
                        argc: 1,
                    },
                    Instr::Return,
                ],
                locals_count: 2,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(1));
}

#[test]
fn vm_vec_delete_reports_index_out_of_bounds() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::LoadConst(Value::Int(0)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "delete".to_string(),
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
    let err = Vm::run_module_main(&module).expect_err("oob");
    assert_eq!(err.kind, VmErrorKind::IndexOutOfBounds);
}

#[test]
fn vm_vec_runtime_type_and_arity_errors_are_reported() {
    let arity_module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
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
    assert!(err.message.contains("vec.new expects 0 arguments"));

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
                        package: "vec".to_string(),
                        name: "len".to_string(),
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
    assert!(err.message.contains("vec.len argument 1 expects Vec"));
}

#[test]
fn vm_vec_get_set_delete_negative_index_errors() {
    let mk_module = |name: &str, argc: usize, extra: Vec<Instr>| BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: {
                    let mut code = vec![Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    }];
                    code.extend(extra);
                    code.push(Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: name.to_string(),
                        argc,
                    });
                    code.push(Instr::Return);
                    code
                },
                locals_count: 0,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };

    let get_m = mk_module("get", 2, vec![Instr::LoadConst(Value::Int(-1))]);
    let e1 = Vm::run_module_main(&get_m).expect_err("negative get");
    assert_eq!(e1.kind, VmErrorKind::IndexOutOfBounds);

    let set_m = mk_module(
        "set",
        3,
        vec![
            Instr::LoadConst(Value::Int(-1)),
            Instr::LoadConst(Value::Int(1)),
        ],
    );
    let e2 = Vm::run_module_main(&set_m).expect_err("negative set");
    assert_eq!(e2.kind, VmErrorKind::IndexOutOfBounds);

    let del_m = mk_module("delete", 2, vec![Instr::LoadConst(Value::Int(-1))]);
    let e3 = Vm::run_module_main(&del_m).expect_err("negative delete");
    assert_eq!(e3.kind, VmErrorKind::IndexOutOfBounds);
}

#[test]
fn vm_vec_get_set_out_of_bounds_and_empty_errors() {
    let get_oob = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::LoadConst(Value::Int(0)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "get".to_string(),
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
    let e1 = Vm::run_module_main(&get_oob).expect_err("get oob");
    assert_eq!(e1.kind, VmErrorKind::IndexOutOfBounds);

    let set_oob = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::LoadConst(Value::Int(0)),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "set".to_string(),
                        argc: 3,
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
    let e2 = Vm::run_module_main(&set_oob).expect_err("set oob");
    assert_eq!(e2.kind, VmErrorKind::IndexOutOfBounds);
}

#[test]
fn vm_vec_delete_first_last_and_singleton_cases() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "new".to_string(),
                        argc: 0,
                    },
                    Instr::StoreLocal(0),
                    // [5,6,7]
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(5)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(6)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(7)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "push".to_string(),
                        argc: 2,
                    },
                    Instr::Pop,
                    // delete first -> 5
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(0)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "delete".to_string(),
                        argc: 2,
                    },
                    Instr::StoreLocal(1),
                    // delete last (now index 1) -> 7
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(1)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "delete".to_string(),
                        argc: 2,
                    },
                    Instr::StoreLocal(2),
                    // delete singleton (remaining [6]) -> 6
                    Instr::LoadLocal(0),
                    Instr::LoadConst(Value::Int(0)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "delete".to_string(),
                        argc: 2,
                    },
                    Instr::StoreLocal(3),
                    // return d1 + d2 + d3 + len
                    Instr::LoadLocal(1),
                    Instr::LoadLocal(2),
                    Instr::Add,
                    Instr::LoadLocal(3),
                    Instr::Add,
                    Instr::LoadLocal(0),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "len".to_string(),
                        argc: 1,
                    },
                    Instr::Add,
                    Instr::Return,
                ],
                locals_count: 4,
                param_count: 0,
            },
        )]
        .into_iter()
        .collect(),
    };
    let out = Vm::run_module_main(&module).expect("run");
    assert_eq!(out, Value::Int(18));
}

#[test]
fn vm_vec_each_function_reports_arity_and_type_errors() {
    let cases = vec![
        (
            "len",
            BytecodeModule {
                method_names: Vec::new(),
                struct_shapes: Vec::new(),
                functions: vec![(
                    "main".to_string(),
                    FunctionChunk {
                        name: "main".to_string(),
                        code: vec![
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "len".to_string(),
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
            },
            VmErrorKind::ArityMismatch,
        ),
        (
            "push",
            BytecodeModule {
                method_names: Vec::new(),
                struct_shapes: Vec::new(),
                functions: vec![(
                    "main".to_string(),
                    FunctionChunk {
                        name: "main".to_string(),
                        code: vec![
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "new".to_string(),
                                argc: 0,
                            },
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "push".to_string(),
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
            },
            VmErrorKind::ArityMismatch,
        ),
        (
            "get",
            BytecodeModule {
                method_names: Vec::new(),
                struct_shapes: Vec::new(),
                functions: vec![(
                    "main".to_string(),
                    FunctionChunk {
                        name: "main".to_string(),
                        code: vec![
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "new".to_string(),
                                argc: 0,
                            },
                            Instr::LoadConst(Value::String("x".to_string().into())),
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "get".to_string(),
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
            },
            VmErrorKind::TypeMismatch,
        ),
        (
            "set",
            BytecodeModule {
                method_names: Vec::new(),
                struct_shapes: Vec::new(),
                functions: vec![(
                    "main".to_string(),
                    FunctionChunk {
                        name: "main".to_string(),
                        code: vec![
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "new".to_string(),
                                argc: 0,
                            },
                            Instr::LoadConst(Value::Bool(true)),
                            Instr::LoadConst(Value::Int(1)),
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "set".to_string(),
                                argc: 3,
                            },
                            Instr::Return,
                        ],
                        locals_count: 0,
                        param_count: 0,
                    },
                )]
                .into_iter()
                .collect(),
            },
            VmErrorKind::TypeMismatch,
        ),
        (
            "delete",
            BytecodeModule {
                method_names: Vec::new(),
                struct_shapes: Vec::new(),
                functions: vec![(
                    "main".to_string(),
                    FunctionChunk {
                        name: "main".to_string(),
                        code: vec![
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "new".to_string(),
                                argc: 0,
                            },
                            Instr::LoadConst(Value::Bool(true)),
                            Instr::CallBuiltin {
                                package: "vec".to_string(),
                                name: "delete".to_string(),
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
            },
            VmErrorKind::TypeMismatch,
        ),
    ];

    for (label, module, kind) in cases {
        let err = Vm::run_module_main(&module).expect_err(label);
        assert_eq!(err.kind, kind, "{label}: {}", err.message);
    }
}

#[test]
fn vm_vec_invalid_handle_reports_type_mismatch() {
    let module = BytecodeModule {
        method_names: Vec::new(),
        struct_shapes: Vec::new(),
        functions: vec![(
            "main".to_string(),
            FunctionChunk {
                name: "main".to_string(),
                code: vec![
                    Instr::LoadConst(Value::VecHandle(999_999)),
                    Instr::CallBuiltin {
                        package: "vec".to_string(),
                        name: "len".to_string(),
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
    let err = Vm::run_module_main(&module).expect_err("invalid handle");
    assert_eq!(err.kind, VmErrorKind::TypeMismatch);
    assert!(err.message.contains("invalid vec handle"));
}
