use crate::codegen::CodegenError;
use crate::codegen::llvm::types::llvm_ty;
use crate::codegen::llvm::value::{ValueNames, operand_load};
use crate::ir::Instr;
use crate::ir::{BuiltinCall, IrFunction, IrProgram, IrType, TempId};
use std::collections::HashMap;

pub fn ensure_supported(instr: &Instr) -> Result<(), CodegenError> {
    match instr {
        Instr::MakeArray { .. }
        | Instr::MakeArrayRepeat { .. }
        | Instr::ArrayGet { .. }
        | Instr::ArraySet { .. }
        | Instr::VecNew { .. }
        | Instr::VecLen { .. }
        | Instr::VecPush { .. }
        | Instr::VecGet { .. }
        | Instr::VecSet { .. }
        | Instr::VecDelete { .. }
        | Instr::MakeStruct { .. }
        | Instr::StructGet { .. }
        | Instr::StructSet { .. }
        | Instr::MakeClosure { .. } => Err(CodegenError::Unsupported(
            "runtime-backed values are not lowered until later LLVM milestones",
        )),
        Instr::CallBuiltin { builtin, .. } if !is_supported_builtin(builtin) => Err(
            CodegenError::Unsupported("only str.* builtins are lowered in current LLVM milestone"),
        ),
        _ => Ok(()),
    }
}

pub fn emit_runtime_decls(program: &IrProgram, out: &mut Vec<String>) {
    if uses_strings(program) {
        out.push("declare ptr @skp_rt_string_from_utf8(ptr, i64)".into());
    }
    if uses_builtin(program, "str", "len") {
        out.push("declare i64 @skp_rt_builtin_str_len(ptr)".into());
    }
    if uses_builtin(program, "str", "contains") {
        out.push("declare i1 @skp_rt_builtin_str_contains(ptr, ptr)".into());
    }
    if uses_builtin(program, "str", "indexOf") {
        out.push("declare i64 @skp_rt_builtin_str_index_of(ptr, ptr)".into());
    }
    if uses_builtin(program, "str", "slice") {
        out.push("declare ptr @skp_rt_builtin_str_slice(ptr, i64, i64)".into());
    }
}

pub struct BuiltinCallInstr<'a> {
    pub dst: Option<TempId>,
    pub ret_ty: &'a IrType,
    pub builtin: &'a BuiltinCall,
    pub args: &'a [crate::ir::Operand],
}

pub fn emit_builtin_call(
    func: &IrFunction,
    names: &ValueNames,
    call: BuiltinCallInstr<'_>,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let helper = match (call.builtin.package.as_str(), call.builtin.name.as_str()) {
        ("str", "len") => "skp_rt_builtin_str_len",
        ("str", "contains") => "skp_rt_builtin_str_contains",
        ("str", "indexOf") => "skp_rt_builtin_str_index_of",
        ("str", "slice") => "skp_rt_builtin_str_slice",
        _ => {
            return Err(CodegenError::Unsupported(
                "only str.* builtins are lowered in current LLVM milestone",
            ));
        }
    };

    let expected = match call.builtin.name.as_str() {
        "len" => vec![IrType::String],
        "contains" => vec![IrType::String, IrType::String],
        "indexOf" => vec![IrType::String, IrType::String],
        "slice" => vec![IrType::String, IrType::Int, IrType::Int],
        _ => unreachable!(),
    };
    if call.args.len() != expected.len() {
        return Err(CodegenError::InvalidIr(format!(
            "builtin arity mismatch for {}.{}",
            call.builtin.package, call.builtin.name
        )));
    }

    let mut lowered_args = Vec::with_capacity(call.args.len());
    for (arg, ty) in call.args.iter().zip(expected.iter()) {
        let value = operand_load(names, arg, func, lines, counter, ty, string_literals)?;
        lowered_args.push(format!("{} {value}", llvm_ty(ty)?));
    }
    let joined_args = lowered_args.join(", ");
    let ret_llvm_ty = llvm_ty(call.ret_ty)?;

    if call.ret_ty.is_void() {
        lines.push(format!("  call {ret_llvm_ty} @{helper}({joined_args})"));
        return Ok(());
    }

    let Some(dst) = call.dst else {
        return Err(CodegenError::InvalidIr(
            "non-void builtin call must write to a destination temp".into(),
        ));
    };
    let dest = names.temp(dst)?;
    lines.push(format!(
        "  {dest} = call {ret_llvm_ty} @{helper}({joined_args})"
    ));
    Ok(())
}

fn uses_strings(program: &IrProgram) -> bool {
    program
        .globals
        .iter()
        .any(|g| matches!(g.ty, IrType::String))
        || program.functions.iter().any(function_uses_strings)
}

fn function_uses_strings(func: &IrFunction) -> bool {
    matches!(func.ret_ty, IrType::String)
        || func
            .params
            .iter()
            .any(|param| matches!(param.ty, IrType::String))
        || func
            .locals
            .iter()
            .any(|local| matches!(local.ty, IrType::String))
        || func
            .temps
            .iter()
            .any(|temp| matches!(temp.ty, IrType::String))
        || func.blocks.iter().any(|block| {
            block.instrs.iter().any(instr_uses_strings)
                || matches!(
                    &block.terminator,
                    crate::ir::Terminator::Return(Some(crate::ir::Operand::Const(
                        crate::ir::ConstValue::String(_),
                    )))
                )
        })
}

fn instr_uses_strings(instr: &Instr) -> bool {
    match instr {
        Instr::Const { ty, value, .. } => {
            matches!(ty, IrType::String) || matches!(value, crate::ir::ConstValue::String(_))
        }
        Instr::Copy { ty, src, .. }
        | Instr::StoreGlobal { ty, value: src, .. }
        | Instr::StoreLocal { ty, value: src, .. } => {
            matches!(ty, IrType::String)
                || matches!(
                    src,
                    crate::ir::Operand::Const(crate::ir::ConstValue::String(_))
                )
        }
        Instr::LoadGlobal { ty, .. } | Instr::LoadLocal { ty, .. } => matches!(ty, IrType::String),
        Instr::CallDirect { ret_ty, args, .. } | Instr::CallBuiltin { ret_ty, args, .. } => {
            matches!(ret_ty, IrType::String)
                || args.iter().any(|arg| {
                    matches!(
                        arg,
                        crate::ir::Operand::Const(crate::ir::ConstValue::String(_))
                    )
                })
        }
        _ => false,
    }
}

fn uses_builtin(program: &IrProgram, package: &str, name: &str) -> bool {
    program.functions.iter().any(|func| {
        func.blocks.iter().any(|block| {
            block.instrs.iter().any(|instr| match instr {
                Instr::CallBuiltin { builtin, .. } => {
                    builtin.package == package && builtin.name == name
                }
                _ => false,
            })
        })
    })
}

fn is_supported_builtin(builtin: &BuiltinCall) -> bool {
    builtin.package == "str"
        && matches!(
            builtin.name.as_str(),
            "len" | "contains" | "indexOf" | "slice"
        )
}
