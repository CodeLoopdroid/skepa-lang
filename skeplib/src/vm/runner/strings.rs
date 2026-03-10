use crate::bytecode::{Instr, Value};
use crate::vm::{VmError, VmErrorKind};

use super::{err_at, invalid_local_slot, state};

pub(super) fn handle_hot_string_instr(
    frame: &mut state::CallFrame<'_>,
    instr: &Instr,
    function_name: &str,
    ip: usize,
) -> Result<bool, VmError> {
    match instr {
        Instr::StrLen => {
            let Some(value) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on StrLen",
                    function_name,
                    ip,
                ));
            };
            frame
                .stack
                .push(super::super::builtins::str::direct_str_len(value)?);
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrLenLocal(slot) => {
            let Some(value) = frame.locals.get(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            match value {
                Value::String(s) => frame
                    .stack
                    .push(Value::Int(super::super::builtins::str::str_len_ref(s))),
                _ => {
                    return Err(err_at(
                        VmErrorKind::TypeMismatch,
                        "StrLenLocal expects String local",
                        function_name,
                        ip,
                    ));
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrIndexOfConst(needle) => {
            let Some(value) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on StrIndexOfConst",
                    function_name,
                    ip,
                ));
            };
            frame
                .stack
                .push(super::super::builtins::str::direct_str_index_of_const(
                    value, needle,
                )?);
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrIndexOfLocalConst { slot, needle } => {
            let Some(value) = frame.locals.get(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            match value {
                Value::String(s) => frame.stack.push(Value::Int(
                    super::super::builtins::str::str_index_of_const_or_neg1(s, needle),
                )),
                _ => {
                    return Err(err_at(
                        VmErrorKind::TypeMismatch,
                        "StrIndexOfLocalConst expects String local",
                        function_name,
                        ip,
                    ));
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrSliceConst { start, end } => {
            let Some(value) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on StrSliceConst",
                    function_name,
                    ip,
                ));
            };
            frame
                .stack
                .push(super::super::builtins::str::direct_str_slice_const(
                    value, *start, *end,
                )?);
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrSliceLocalConst { slot, start, end } => {
            let Some(value) = frame.locals.get(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            match value {
                Value::String(s) => {
                    let len = super::super::builtins::str::str_len_ref(s);
                    if *start < 0 || *end < 0 || *start > *end || *end > len {
                        return Err(err_at(
                            VmErrorKind::IndexOutOfBounds,
                            format!(
                                "str.slice bounds out of range: start={}, end={}, len={len}",
                                start, end
                            ),
                            function_name,
                            ip,
                        ));
                    }
                    frame.stack.push(Value::String(
                        super::super::builtins::str::str_slice_const_ref(s, *start, *end).into(),
                    ));
                }
                _ => {
                    return Err(err_at(
                        VmErrorKind::TypeMismatch,
                        "StrSliceLocalConst expects String local",
                        function_name,
                        ip,
                    ));
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrContainsConst(needle) => {
            let Some(value) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on StrContainsConst",
                    function_name,
                    ip,
                ));
            };
            frame
                .stack
                .push(super::super::builtins::str::direct_str_contains_const(
                    value, needle,
                )?);
            frame.ip += 1;
            Ok(true)
        }
        Instr::StrContainsLocalConst { slot, needle } => {
            let Some(value) = frame.locals.get(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            match value {
                Value::String(s) => frame.stack.push(Value::Bool(
                    super::super::builtins::str::str_contains_const_ref(s, needle),
                )),
                _ => {
                    return Err(err_at(
                        VmErrorKind::TypeMismatch,
                        "StrContainsLocalConst expects String local",
                        function_name,
                        ip,
                    ));
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        _ => Ok(false),
    }
}
