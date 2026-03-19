use crate::codegen::CodegenError;
use crate::codegen::llvm::types::llvm_ty;
use crate::codegen::llvm::value::{ValueNames, llvm_symbol, operand_load, raw_string_ptr};
use crate::ir::Instr;
use crate::ir::{BuiltinCall, IrFunction, IrProgram, IrType, TempId};
use std::collections::HashMap;

const RUNTIME_DECLS: &[(&str, &str)] = &[
    (
        "skp_rt_string_from_utf8",
        "declare ptr @skp_rt_string_from_utf8(ptr, i64)",
    ),
    ("skp_rt_string_eq", "declare i1 @skp_rt_string_eq(ptr, ptr)"),
    (
        "skp_rt_builtin_str_len",
        "declare i64 @skp_rt_builtin_str_len(ptr)",
    ),
    (
        "skp_rt_builtin_str_contains",
        "declare i1 @skp_rt_builtin_str_contains(ptr, ptr)",
    ),
    (
        "skp_rt_builtin_str_index_of",
        "declare i64 @skp_rt_builtin_str_index_of(ptr, ptr)",
    ),
    (
        "skp_rt_builtin_str_slice",
        "declare ptr @skp_rt_builtin_str_slice(ptr, i64, i64)",
    ),
    (
        "skp_rt_call_builtin",
        "declare ptr @skp_rt_call_builtin(ptr, ptr, i64, ptr)",
    ),
    (
        "skp_rt_call_function",
        "declare ptr @skp_rt_call_function(i32, i64, ptr)",
    ),
    (
        "skp_rt_abort_if_error",
        "declare void @skp_rt_abort_if_error()",
    ),
    (
        "skp_rt_value_from_int",
        "declare ptr @skp_rt_value_from_int(i64)",
    ),
    (
        "skp_rt_value_from_bool",
        "declare ptr @skp_rt_value_from_bool(i1)",
    ),
    (
        "skp_rt_value_from_float",
        "declare ptr @skp_rt_value_from_float(double)",
    ),
    (
        "skp_rt_value_from_unit",
        "declare ptr @skp_rt_value_from_unit()",
    ),
    (
        "skp_rt_value_from_string",
        "declare ptr @skp_rt_value_from_string(ptr)",
    ),
    (
        "skp_rt_value_from_array",
        "declare ptr @skp_rt_value_from_array(ptr)",
    ),
    (
        "skp_rt_value_from_vec",
        "declare ptr @skp_rt_value_from_vec(ptr)",
    ),
    (
        "skp_rt_value_from_struct",
        "declare ptr @skp_rt_value_from_struct(ptr)",
    ),
    (
        "skp_rt_value_from_function",
        "declare ptr @skp_rt_value_from_function(i32)",
    ),
    ("skp_rt_value_free", "declare void @skp_rt_value_free(ptr)"),
    (
        "skp_rt_value_to_int",
        "declare i64 @skp_rt_value_to_int(ptr)",
    ),
    (
        "skp_rt_value_to_bool",
        "declare i1 @skp_rt_value_to_bool(ptr)",
    ),
    (
        "skp_rt_value_to_float",
        "declare double @skp_rt_value_to_float(ptr)",
    ),
    (
        "skp_rt_value_to_string",
        "declare ptr @skp_rt_value_to_string(ptr)",
    ),
    (
        "skp_rt_value_to_array",
        "declare ptr @skp_rt_value_to_array(ptr)",
    ),
    (
        "skp_rt_value_to_vec",
        "declare ptr @skp_rt_value_to_vec(ptr)",
    ),
    (
        "skp_rt_value_to_struct",
        "declare ptr @skp_rt_value_to_struct(ptr)",
    ),
    (
        "skp_rt_value_to_function",
        "declare i32 @skp_rt_value_to_function(ptr)",
    ),
    ("skp_rt_array_new", "declare ptr @skp_rt_array_new(i64)"),
    (
        "skp_rt_array_repeat",
        "declare ptr @skp_rt_array_repeat(ptr, i64)",
    ),
    (
        "skp_rt_array_get",
        "declare ptr @skp_rt_array_get(ptr, i64)",
    ),
    (
        "skp_rt_array_set",
        "declare void @skp_rt_array_set(ptr, i64, ptr)",
    ),
    ("skp_rt_vec_new", "declare ptr @skp_rt_vec_new()"),
    ("skp_rt_vec_len", "declare i64 @skp_rt_vec_len(ptr)"),
    ("skp_rt_vec_push", "declare void @skp_rt_vec_push(ptr, ptr)"),
    ("skp_rt_vec_get", "declare ptr @skp_rt_vec_get(ptr, i64)"),
    (
        "skp_rt_vec_set",
        "declare void @skp_rt_vec_set(ptr, i64, ptr)",
    ),
    (
        "skp_rt_vec_delete",
        "declare ptr @skp_rt_vec_delete(ptr, i64)",
    ),
    (
        "skp_rt_struct_new",
        "declare ptr @skp_rt_struct_new(i64, i64)",
    ),
    (
        "skp_rt_struct_get",
        "declare ptr @skp_rt_struct_get(ptr, i64)",
    ),
    (
        "skp_rt_struct_set",
        "declare void @skp_rt_struct_set(ptr, i64, ptr)",
    ),
];

pub fn ensure_supported(instr: &Instr) -> Result<(), CodegenError> {
    let _ = instr;
    Ok(())
}

pub fn emit_runtime_decls(program: &IrProgram, out: &mut Vec<String>) -> Result<(), CodegenError> {
    for (_, decl) in runtime_declarations() {
        out.push((*decl).into());
    }
    emit_indirect_call_dispatch(program, out)?;
    Ok(())
}

fn runtime_declarations() -> &'static [(&'static str, &'static str)] {
    RUNTIME_DECLS
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
        _ => return emit_builtin_call_generic(func, names, call, lines, counter, string_literals),
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
        emit_abort_if_error(lines);
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
    emit_abort_if_error(lines);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_builtin_call_generic(
    func: &IrFunction,
    names: &ValueNames,
    call: BuiltinCallInstr<'_>,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let package_ptr = raw_string_ptr(&call.builtin.package, lines, counter, string_literals)?;
    let name_ptr = raw_string_ptr(&call.builtin.name, lines, counter, string_literals)?;
    let boxed_args = emit_boxed_arg_array(func, names, call.args, lines, counter, string_literals)?;
    let raw = format!("%v{counter}");
    *counter += 1;
    lines.push(format!(
        "  {raw} = call ptr @skp_rt_call_builtin(ptr {package_ptr}, ptr {name_ptr}, i64 {}, ptr {})",
        call.args.len(),
        boxed_args.array
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_values(&boxed_args.values, lines);
    if call.ret_ty.is_void() {
        emit_free_boxed_value(&raw, lines);
        return Ok(());
    }
    let Some(dst) = call.dst else {
        return Err(CodegenError::InvalidIr(
            "non-void builtin call must write to a destination temp".into(),
        ));
    };
    emit_unbox_value(names, dst, call.ret_ty, &raw, lines)?;
    emit_free_boxed_value(&raw, lines);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_indirect_call(
    func: &IrFunction,
    names: &ValueNames,
    dst: Option<TempId>,
    ret_ty: &IrType,
    callee: &crate::ir::Operand,
    args: &[crate::ir::Operand],
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let callee_ty = infer_operand_type(func, callee);
    let (callee_params, callee_ret) = match &callee_ty {
        IrType::Fn { params, ret } => (params.clone(), ret.as_ref().clone()),
        other => {
            return Err(CodegenError::InvalidIr(format!(
                "indirect callee must have function type, got {:?}",
                other
            )));
        }
    };
    if callee_ret != *ret_ty {
        return Err(CodegenError::InvalidIr(format!(
            "indirect call return type mismatch: callee returns {:?}, call expects {:?}",
            callee_ret, ret_ty
        )));
    }
    if callee_params.len() != args.len() {
        return Err(CodegenError::InvalidIr(format!(
            "indirect call arity mismatch: callee expects {}, got {}",
            callee_params.len(),
            args.len()
        )));
    }
    let callee = operand_load(
        names,
        callee,
        func,
        lines,
        counter,
        &callee_ty,
        string_literals,
    )?;
    let boxed_args = emit_boxed_arg_array(func, names, args, lines, counter, string_literals)?;
    let raw = format!("%v{counter}");
    *counter += 1;
    lines.push(format!(
        "  {raw} = call ptr @__skp_rt_call_function_dispatch(i32 {callee}, i64 {}, ptr {})",
        args.len(),
        boxed_args.array
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_values(&boxed_args.values, lines);
    if ret_ty.is_void() {
        emit_free_boxed_value(&raw, lines);
        return Ok(());
    }
    let Some(dst) = dst else {
        return Err(CodegenError::InvalidIr(
            "non-void indirect call must write to a destination temp".into(),
        ));
    };
    emit_unbox_value(names, dst, ret_ty, &raw, lines)?;
    emit_free_boxed_value(&raw, lines);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_make_array(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    elem_ty: &IrType,
    items: &[crate::ir::Operand],
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let dest = names.temp(dst)?;
    lines.push(format!(
        "  {dest} = call ptr @skp_rt_array_new(i64 {})",
        items.len()
    ));
    for (index, item) in items.iter().enumerate() {
        let boxed =
            emit_boxed_operand(func, names, item, elem_ty, lines, counter, string_literals)?;
        lines.push(format!(
            "  call void @skp_rt_array_set(ptr {dest}, i64 {index}, ptr {boxed})"
        ));
        emit_abort_if_error(lines);
        emit_free_boxed_value(&boxed, lines);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_make_array_repeat(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    elem_ty: &IrType,
    value: &crate::ir::Operand,
    size: usize,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let dest = names.temp(dst)?;
    let boxed = emit_boxed_operand(func, names, value, elem_ty, lines, counter, string_literals)?;
    lines.push(format!(
        "  {dest} = call ptr @skp_rt_array_repeat(ptr {boxed}, i64 {size})"
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_value(&boxed, lines);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_array_get(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    elem_ty: &IrType,
    array: &crate::ir::Operand,
    index: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let array = operand_load(
        names,
        array,
        func,
        lines,
        counter,
        &IrType::Array {
            elem: Box::new(elem_ty.clone()),
            size: 0,
        },
        string_literals,
    )?;
    let index = operand_load(
        names,
        index,
        func,
        lines,
        counter,
        &IrType::Int,
        string_literals,
    )?;
    let raw = format!("%v{counter}");
    *counter += 1;
    lines.push(format!(
        "  {raw} = call ptr @skp_rt_array_get(ptr {array}, i64 {index})"
    ));
    emit_abort_if_error(lines);
    emit_unbox_value(names, dst, elem_ty, &raw, lines)
}

#[allow(clippy::too_many_arguments)]
pub fn emit_array_set(
    func: &IrFunction,
    names: &ValueNames,
    elem_ty: &IrType,
    array: &crate::ir::Operand,
    index: &crate::ir::Operand,
    value: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let array = operand_load(
        names,
        array,
        func,
        lines,
        counter,
        &IrType::Array {
            elem: Box::new(elem_ty.clone()),
            size: 0,
        },
        string_literals,
    )?;
    let index = operand_load(
        names,
        index,
        func,
        lines,
        counter,
        &IrType::Int,
        string_literals,
    )?;
    let boxed = emit_boxed_operand(func, names, value, elem_ty, lines, counter, string_literals)?;
    lines.push(format!(
        "  call void @skp_rt_array_set(ptr {array}, i64 {index}, ptr {boxed})"
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_value(&boxed, lines);
    Ok(())
}

pub fn emit_vec_new(
    names: &ValueNames,
    dst: TempId,
    lines: &mut Vec<String>,
) -> Result<(), CodegenError> {
    let dest = names.temp(dst)?;
    lines.push(format!("  {dest} = call ptr @skp_rt_vec_new()"));
    Ok(())
}

pub fn emit_vec_len(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    vec: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let vec = operand_load(
        names,
        vec,
        func,
        lines,
        counter,
        &IrType::Vec {
            elem: Box::new(IrType::Unknown),
        },
        string_literals,
    )?;
    let dest = names.temp(dst)?;
    lines.push(format!("  {dest} = call i64 @skp_rt_vec_len(ptr {vec})"));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_vec_push(
    func: &IrFunction,
    names: &ValueNames,
    elem_ty: &IrType,
    vec: &crate::ir::Operand,
    value: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let elem_ty = if matches!(elem_ty, IrType::Unknown) {
        infer_operand_type(func, value)
    } else {
        elem_ty.clone()
    };
    let vec = operand_load(
        names,
        vec,
        func,
        lines,
        counter,
        &IrType::Vec {
            elem: Box::new(elem_ty.clone()),
        },
        string_literals,
    )?;
    let boxed = emit_boxed_operand(
        func,
        names,
        value,
        &elem_ty,
        lines,
        counter,
        string_literals,
    )?;
    lines.push(format!(
        "  call void @skp_rt_vec_push(ptr {vec}, ptr {boxed})"
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_value(&boxed, lines);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_vec_get(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    elem_ty: &IrType,
    vec: &crate::ir::Operand,
    index: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let vec = operand_load(
        names,
        vec,
        func,
        lines,
        counter,
        &IrType::Vec {
            elem: Box::new(elem_ty.clone()),
        },
        string_literals,
    )?;
    let index = operand_load(
        names,
        index,
        func,
        lines,
        counter,
        &IrType::Int,
        string_literals,
    )?;
    let raw = format!("%v{counter}");
    *counter += 1;
    lines.push(format!(
        "  {raw} = call ptr @skp_rt_vec_get(ptr {vec}, i64 {index})"
    ));
    emit_abort_if_error(lines);
    emit_unbox_value(names, dst, elem_ty, &raw, lines)
}

#[allow(clippy::too_many_arguments)]
pub fn emit_vec_set(
    func: &IrFunction,
    names: &ValueNames,
    elem_ty: &IrType,
    vec: &crate::ir::Operand,
    index: &crate::ir::Operand,
    value: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let vec = operand_load(
        names,
        vec,
        func,
        lines,
        counter,
        &IrType::Vec {
            elem: Box::new(elem_ty.clone()),
        },
        string_literals,
    )?;
    let index = operand_load(
        names,
        index,
        func,
        lines,
        counter,
        &IrType::Int,
        string_literals,
    )?;
    let boxed = emit_boxed_operand(func, names, value, elem_ty, lines, counter, string_literals)?;
    lines.push(format!(
        "  call void @skp_rt_vec_set(ptr {vec}, i64 {index}, ptr {boxed})"
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_value(&boxed, lines);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_vec_delete(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    elem_ty: &IrType,
    vec: &crate::ir::Operand,
    index: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let vec = operand_load(
        names,
        vec,
        func,
        lines,
        counter,
        &IrType::Vec {
            elem: Box::new(elem_ty.clone()),
        },
        string_literals,
    )?;
    let index = operand_load(
        names,
        index,
        func,
        lines,
        counter,
        &IrType::Int,
        string_literals,
    )?;
    let raw = format!("%v{counter}");
    *counter += 1;
    lines.push(format!(
        "  {raw} = call ptr @skp_rt_vec_delete(ptr {vec}, i64 {index})"
    ));
    emit_abort_if_error(lines);
    emit_unbox_value(names, dst, elem_ty, &raw, lines)
}

#[allow(clippy::too_many_arguments)]
pub fn emit_make_struct(
    program: &IrProgram,
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    struct_id: crate::ir::StructId,
    fields: &[crate::ir::Operand],
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let struct_info = program
        .structs
        .iter()
        .find(|candidate| candidate.id == struct_id)
        .ok_or_else(|| CodegenError::InvalidIr(format!("unknown struct {:?}", struct_id)))?;
    let dest = names.temp(dst)?;
    lines.push(format!(
        "  {dest} = call ptr @skp_rt_struct_new(i64 {}, i64 {})",
        struct_id.0,
        fields.len()
    ));
    for (index, (field, field_info)) in fields.iter().zip(&struct_info.fields).enumerate() {
        let boxed = emit_boxed_operand(
            func,
            names,
            field,
            &field_info.ty,
            lines,
            counter,
            string_literals,
        )?;
        lines.push(format!(
            "  call void @skp_rt_struct_set(ptr {dest}, i64 {index}, ptr {boxed})"
        ));
        emit_abort_if_error(lines);
        emit_free_boxed_value(&boxed, lines);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn emit_struct_get(
    func: &IrFunction,
    names: &ValueNames,
    dst: TempId,
    ty: &IrType,
    base: &crate::ir::Operand,
    field: &crate::ir::FieldRef,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let base = operand_load(
        names,
        base,
        func,
        lines,
        counter,
        &IrType::Named(String::new()),
        string_literals,
    )?;
    let raw = format!("%v{counter}");
    *counter += 1;
    lines.push(format!(
        "  {raw} = call ptr @skp_rt_struct_get(ptr {base}, i64 {})",
        field.index
    ));
    emit_abort_if_error(lines);
    emit_unbox_value(names, dst, ty, &raw, lines)
}

#[allow(clippy::too_many_arguments)]
pub fn emit_struct_set(
    func: &IrFunction,
    names: &ValueNames,
    ty: &IrType,
    base: &crate::ir::Operand,
    field: &crate::ir::FieldRef,
    value: &crate::ir::Operand,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<(), CodegenError> {
    let base = operand_load(
        names,
        base,
        func,
        lines,
        counter,
        &IrType::Named(String::new()),
        string_literals,
    )?;
    let boxed = emit_boxed_operand(func, names, value, ty, lines, counter, string_literals)?;
    lines.push(format!(
        "  call void @skp_rt_struct_set(ptr {base}, i64 {}, ptr {boxed})",
        field.index
    ));
    emit_abort_if_error(lines);
    emit_free_boxed_value(&boxed, lines);
    Ok(())
}

struct BoxedArgArray {
    array: String,
    values: Vec<String>,
}

fn emit_boxed_arg_array(
    func: &IrFunction,
    names: &ValueNames,
    args: &[crate::ir::Operand],
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<BoxedArgArray, CodegenError> {
    let array = format!("%v{counter}");
    *counter += 1;
    let len = args.len().max(1);
    lines.push(format!("  {array} = alloca ptr, i64 {len}, align 8"));
    let mut values = Vec::with_capacity(args.len());
    for (index, arg) in args.iter().enumerate() {
        let arg_ty = infer_operand_type(func, arg);
        let boxed = emit_boxed_operand(func, names, arg, &arg_ty, lines, counter, string_literals)?;
        values.push(boxed.clone());
        let slot = format!("%v{counter}");
        *counter += 1;
        lines.push(format!(
            "  {slot} = getelementptr inbounds ptr, ptr {array}, i64 {index}"
        ));
        lines.push(format!("  store ptr {boxed}, ptr {slot}, align 8"));
    }
    Ok(BoxedArgArray { array, values })
}

fn emit_boxed_operand(
    func: &IrFunction,
    names: &ValueNames,
    operand: &crate::ir::Operand,
    ty: &IrType,
    lines: &mut Vec<String>,
    counter: &mut usize,
    string_literals: &HashMap<String, String>,
) -> Result<String, CodegenError> {
    let value = operand_load(names, operand, func, lines, counter, ty, string_literals)?;
    let boxed = format!("%v{counter}");
    *counter += 1;
    let helper = match ty {
        IrType::Int => "skp_rt_value_from_int",
        IrType::Float => "skp_rt_value_from_float",
        IrType::Bool => "skp_rt_value_from_bool",
        IrType::String => "skp_rt_value_from_string",
        IrType::Array { .. } => "skp_rt_value_from_array",
        IrType::Vec { .. } => "skp_rt_value_from_vec",
        IrType::Named(_) => "skp_rt_value_from_struct",
        IrType::Fn { .. } => "skp_rt_value_from_function",
        _ => {
            return Err(CodegenError::Unsupported(
                "boxing is only implemented for Int/Float/Bool/String/Array/Vec/Struct/Function",
            ));
        }
    };
    lines.push(format!(
        "  {boxed} = call ptr @{helper}({} {value})",
        llvm_ty(ty)?
    ));
    Ok(boxed)
}

fn emit_unbox_value(
    names: &ValueNames,
    dst: TempId,
    ty: &IrType,
    raw: &str,
    lines: &mut Vec<String>,
) -> Result<(), CodegenError> {
    let dest = names.temp(dst)?;
    match ty {
        IrType::Int => lines.push(format!(
            "  {dest} = call i64 @skp_rt_value_to_int(ptr {raw})"
        )),
        IrType::Float => lines.push(format!(
            "  {dest} = call double @skp_rt_value_to_float(ptr {raw})"
        )),
        IrType::Bool => lines.push(format!(
            "  {dest} = call i1 @skp_rt_value_to_bool(ptr {raw})"
        )),
        IrType::String => lines.push(format!(
            "  {dest} = call ptr @skp_rt_value_to_string(ptr {raw})"
        )),
        IrType::Array { .. } => lines.push(format!(
            "  {dest} = call ptr @skp_rt_value_to_array(ptr {raw})"
        )),
        IrType::Vec { .. } => lines.push(format!(
            "  {dest} = call ptr @skp_rt_value_to_vec(ptr {raw})"
        )),
        IrType::Named(_) => lines.push(format!(
            "  {dest} = call ptr @skp_rt_value_to_struct(ptr {raw})"
        )),
        IrType::Fn { .. } => lines.push(format!(
            "  {dest} = call i32 @skp_rt_value_to_function(ptr {raw})"
        )),
        _ => {
            return Err(CodegenError::Unsupported(
                "unboxing is only implemented for Int/Float/Bool/String/Array/Vec/Struct/Function",
            ));
        }
    }
    emit_abort_if_error(lines);
    Ok(())
}

fn emit_abort_if_error(lines: &mut Vec<String>) {
    lines.push("  call void @skp_rt_abort_if_error()".into());
}

fn emit_free_boxed_value(value: &str, lines: &mut Vec<String>) {
    lines.push(format!("  call void @skp_rt_value_free(ptr {value})"));
}

fn emit_free_boxed_values(values: &[String], lines: &mut Vec<String>) {
    for value in values {
        emit_free_boxed_value(value, lines);
    }
}

fn infer_operand_type(func: &IrFunction, operand: &crate::ir::Operand) -> IrType {
    match operand {
        crate::ir::Operand::Const(crate::ir::ConstValue::Int(_)) => IrType::Int,
        crate::ir::Operand::Const(crate::ir::ConstValue::Float(_)) => IrType::Float,
        crate::ir::Operand::Const(crate::ir::ConstValue::Bool(_)) => IrType::Bool,
        crate::ir::Operand::Const(crate::ir::ConstValue::String(_)) => IrType::String,
        crate::ir::Operand::Const(crate::ir::ConstValue::Unit) => IrType::Void,
        crate::ir::Operand::Temp(id) => func
            .temps
            .iter()
            .find(|temp| temp.id == *id)
            .map(|temp| temp.ty.clone())
            .unwrap_or(IrType::Unknown),
        crate::ir::Operand::Local(id) => func
            .locals
            .iter()
            .find(|local| local.id == *id)
            .map(|local| local.ty.clone())
            .unwrap_or(IrType::Unknown),
        _ => IrType::Unknown,
    }
}

fn emit_indirect_call_dispatch(
    program: &IrProgram,
    out: &mut Vec<String>,
) -> Result<(), CodegenError> {
    for func in &program.functions {
        out.extend(emit_indirect_wrapper(func)?);
        out.push(String::new());
    }

    out.push("define internal ptr @__skp_rt_call_function_dispatch(i32 %function, i64 %argc, ptr %argv) {".into());
    out.push("entry:".into());
    if program.functions.is_empty() {
        out.push("  %unit = call ptr @skp_rt_value_from_unit()".into());
        out.push("  ret ptr %unit".into());
        out.push("}".into());
        return Ok(());
    }
    let cases = program
        .functions
        .iter()
        .map(|func| format!("    i32 {}, label %case{}", func.id.0, func.id.0))
        .collect::<Vec<_>>()
        .join("\n");
    out.push(format!(
        "  switch i32 %function, label %default [\n{cases}\n  ]"
    ));
    for func in &program.functions {
        out.push(format!("case{}:", func.id.0));
        out.push(format!(
            "  %call{} = call ptr @__skp_rt_fnwrap_{}(i64 %argc, ptr %argv)",
            func.id.0, func.id.0
        ));
        out.push(format!("  ret ptr %call{}", func.id.0));
    }
    out.push("default:".into());
    out.push("  %unit = call ptr @skp_rt_value_from_unit()".into());
    out.push("  ret ptr %unit".into());
    out.push("}".into());
    Ok(())
}

fn emit_indirect_wrapper(func: &IrFunction) -> Result<Vec<String>, CodegenError> {
    let mut lines = vec![format!(
        "define internal ptr @__skp_rt_fnwrap_{}(i64 %argc, ptr %argv) {{",
        func.id.0
    )];
    lines.push("entry:".into());
    let argc_ok = format!("%argc_ok_{}", func.id.0);
    lines.push(format!(
        "  {argc_ok} = icmp eq i64 %argc, {}",
        func.params.len()
    ));
    lines.push(format!(
        "  br i1 {argc_ok}, label %argc_ok, label %argc_bad"
    ));
    lines.push("argc_bad:".into());
    lines.push(format!(
        "  %argc_err = call ptr @skp_rt_call_function(i32 {}, i64 %argc, ptr %argv)",
        func.id.0
    ));
    lines.push("  ret ptr %argc_err".into());
    lines.push("argc_ok:".into());
    for (index, param) in func.params.iter().enumerate() {
        lines.push(format!(
            "  %argslot{index} = getelementptr inbounds ptr, ptr %argv, i64 {index}"
        ));
        lines.push(format!(
            "  %argraw{index} = load ptr, ptr %argslot{index}, align 8"
        ));
        match &param.ty {
            IrType::Int => lines.push(format!(
                "  %arg{index} = call i64 @skp_rt_value_to_int(ptr %argraw{index})"
            )),
            IrType::Float => lines.push(format!(
                "  %arg{index} = call double @skp_rt_value_to_float(ptr %argraw{index})"
            )),
            IrType::Bool => lines.push(format!(
                "  %arg{index} = call i1 @skp_rt_value_to_bool(ptr %argraw{index})"
            )),
            IrType::String => lines.push(format!(
                "  %arg{index} = call ptr @skp_rt_value_to_string(ptr %argraw{index})"
            )),
            IrType::Array { .. } => lines.push(format!(
                "  %arg{index} = call ptr @skp_rt_value_to_array(ptr %argraw{index})"
            )),
            IrType::Vec { .. } => lines.push(format!(
                "  %arg{index} = call ptr @skp_rt_value_to_vec(ptr %argraw{index})"
            )),
            IrType::Named(_) => lines.push(format!(
                "  %arg{index} = call ptr @skp_rt_value_to_struct(ptr %argraw{index})"
            )),
            IrType::Fn { .. } => lines.push(format!(
                "  %arg{index} = call i32 @skp_rt_value_to_function(ptr %argraw{index})"
            )),
            _ => {
                return Err(CodegenError::Unsupported(
                    "indirect-call trampoline only supports Int/Float/Bool/String/Named/Array/Vec/Fn/Void signatures",
                ));
            }
        }
        lines.push("  call void @skp_rt_abort_if_error()".into());
    }
    let joined_args = func
        .params
        .iter()
        .enumerate()
        .map(|(index, param)| Ok(format!("{} %arg{index}", llvm_ty(&param.ty)?)))
        .collect::<Result<Vec<_>, CodegenError>>()?
        .join(", ");
    if func.ret_ty.is_void() {
        lines.push(format!(
            "  call void {}({joined_args})",
            llvm_symbol(&func.name)
        ));
        lines.push("  %unit = call ptr @skp_rt_value_from_unit()".into());
        lines.push("  ret ptr %unit".into());
    } else {
        lines.push(format!(
            "  %ret = call {} {}({joined_args})",
            llvm_ty(&func.ret_ty)?,
            llvm_symbol(&func.name)
        ));
        let boxer = match &func.ret_ty {
            IrType::Int => "skp_rt_value_from_int",
            IrType::Float => "skp_rt_value_from_float",
            IrType::Bool => "skp_rt_value_from_bool",
            IrType::String => "skp_rt_value_from_string",
            IrType::Array { .. } => "skp_rt_value_from_array",
            IrType::Vec { .. } => "skp_rt_value_from_vec",
            IrType::Named(_) => "skp_rt_value_from_struct",
            IrType::Fn { .. } => "skp_rt_value_from_function",
            _ => {
                return Err(CodegenError::Unsupported(
                    "indirect-call trampoline only supports Int/Float/Bool/String/Named/Array/Vec/Fn/Void signatures",
                ));
            }
        };
        lines.push(format!(
            "  %boxed = call ptr @{boxer}({} %ret)",
            llvm_ty(&func.ret_ty)?
        ));
        lines.push("  ret ptr %boxed".into());
    }
    lines.push("}".into());
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::runtime_declarations;
    use std::collections::HashSet;

    #[test]
    fn runtime_declarations_are_unique_and_cover_core_abi_surface() {
        let decls = runtime_declarations();
        let names = decls.iter().map(|(name, _)| *name).collect::<Vec<_>>();
        let unique = names.iter().copied().collect::<HashSet<_>>();
        assert_eq!(names.len(), unique.len(), "duplicate runtime decl names");
        assert!(names.contains(&"skp_rt_call_builtin"));
        assert!(names.contains(&"skp_rt_call_function"));
        assert!(names.contains(&"skp_rt_value_free"));
        assert!(names.contains(&"skp_rt_abort_if_error"));
    }
}
