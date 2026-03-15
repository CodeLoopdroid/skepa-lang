use crate::bytecode::{Instr, IntLocalConstOp, Value};
use crate::vm::{VmError, VmErrorKind};

use super::{arith, control_flow, invalid_local_slot, state, strings};

#[inline(always)]
pub(super) fn handle_hot_instr(
    frame: &mut state::CallFrame<'_>,
    instr: &Instr,
    function_name: &str,
    ip: usize,
) -> Result<bool, VmError> {
    match instr {
        Instr::LoadConst(v) => {
            frame.stack.push(v.clone());
            frame.ip += 1;
            Ok(true)
        }
        Instr::LoadLocal(slot) => {
            let Some(v) = frame.read_local_cloned(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            frame.stack.push(v);
            frame.ip += 1;
            Ok(true)
        }
        Instr::StoreLocal(slot) => {
            let Some(v) = frame.stack.pop() else {
                return Err(super::err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on StoreLocal",
                    function_name,
                    ip,
                ));
            };
            if !frame.write_local_fast(*slot, v) {
                return Err(invalid_local_slot(function_name, ip, *slot));
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::AddLocalToLocal { dst, src } => match frame.add_int_local_to_local(*dst, *src) {
            Some(Ok(())) => {
                frame.ip += 1;
                Ok(true)
            }
            Some(Err(())) => Ok(false),
            None => {
                if *dst >= frame.locals.len() {
                    Err(invalid_local_slot(function_name, ip, *dst))
                } else {
                    Err(invalid_local_slot(function_name, ip, *src))
                }
            }
        },
        Instr::AddConstToLocal { slot, rhs } => match frame.add_const_to_int_local(*slot, *rhs) {
            Some(Ok(())) => {
                frame.ip += 1;
                Ok(true)
            }
            Some(Err(())) => Ok(false),
            None => Err(invalid_local_slot(function_name, ip, *slot)),
        },
        Instr::IntLocalLocalOp { lhs, rhs, op } => {
            let Some(left) = frame.locals.get(*lhs) else {
                return Err(invalid_local_slot(function_name, ip, *lhs));
            };
            let Some(right) = frame.locals.get(*rhs) else {
                return Err(invalid_local_slot(function_name, ip, *rhs));
            };
            match (left, right) {
                (Value::Int(lhs), Value::Int(rhs)) => {
                    let result = match op {
                        IntLocalConstOp::Add => Value::Int(*lhs + *rhs),
                        IntLocalConstOp::Sub => Value::Int(*lhs - *rhs),
                        IntLocalConstOp::Mul => Value::Int(*lhs * *rhs),
                        IntLocalConstOp::Div => {
                            if *rhs == 0 {
                                return Err(super::err_at(
                                    VmErrorKind::DivisionByZero,
                                    "division by zero",
                                    function_name,
                                    ip,
                                ));
                            }
                            Value::Int(*lhs / *rhs)
                        }
                        IntLocalConstOp::Mod => {
                            if *rhs == 0 {
                                return Err(super::err_at(
                                    VmErrorKind::DivisionByZero,
                                    "modulo by zero",
                                    function_name,
                                    ip,
                                ));
                            }
                            Value::Int(*lhs % *rhs)
                        }
                    };
                    frame.stack.push(result);
                    frame.ip += 1;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        Instr::IntLocalConstOp { slot, op, rhs } => {
            let Some(value) = frame.locals.get(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            match value {
                Value::Int(lhs) => {
                    let result = match op {
                        IntLocalConstOp::Add => Value::Int(*lhs + *rhs),
                        IntLocalConstOp::Sub => Value::Int(*lhs - *rhs),
                        IntLocalConstOp::Mul => Value::Int(*lhs * *rhs),
                        IntLocalConstOp::Div => {
                            if *rhs == 0 {
                                return Err(super::err_at(
                                    VmErrorKind::DivisionByZero,
                                    "division by zero",
                                    function_name,
                                    ip,
                                ));
                            }
                            Value::Int(*lhs / *rhs)
                        }
                        IntLocalConstOp::Mod => {
                            if *rhs == 0 {
                                return Err(super::err_at(
                                    VmErrorKind::DivisionByZero,
                                    "modulo by zero",
                                    function_name,
                                    ip,
                                ));
                            }
                            Value::Int(*lhs % *rhs)
                        }
                    };
                    frame.stack.push(result);
                    frame.ip += 1;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        Instr::IntLocalConstOpToLocal { src, dst, op, rhs } => {
            match frame.compute_int_local_const_to_local(*src, *dst, *op, *rhs) {
                Some(Ok(())) => {
                    frame.ip += 1;
                    Ok(true)
                }
                Some(Err(VmErrorKind::DivisionByZero)) => Err(super::err_at(
                    VmErrorKind::DivisionByZero,
                    match op {
                        IntLocalConstOp::Div => "division by zero",
                        IntLocalConstOp::Mod => "modulo by zero",
                        _ => "division by zero",
                    },
                    function_name,
                    ip,
                )),
                Some(Err(VmErrorKind::TypeMismatch)) => Ok(false),
                Some(Err(_)) => Ok(false),
                None => Err(if *src >= frame.locals.len() {
                    invalid_local_slot(function_name, ip, *src)
                } else {
                    invalid_local_slot(function_name, ip, *dst)
                }),
            }
        }
        Instr::IntStackOpToLocal { slot, op } => match frame.apply_stack_int_to_local(*slot, *op) {
            Some(Ok(())) => {
                frame.ip += 1;
                Ok(true)
            }
            Some(Err(VmErrorKind::StackUnderflow)) => Err(super::err_at(
                VmErrorKind::StackUnderflow,
                "Stack underflow on IntStackOpToLocal",
                function_name,
                ip,
            )),
            Some(Err(VmErrorKind::DivisionByZero)) => Err(super::err_at(
                VmErrorKind::DivisionByZero,
                match op {
                    IntLocalConstOp::Div => "division by zero",
                    IntLocalConstOp::Mod => "modulo by zero",
                    _ => "division by zero",
                },
                function_name,
                ip,
            )),
            Some(Err(VmErrorKind::TypeMismatch)) => Ok(false),
            Some(Err(_)) => Ok(false),
            None => Err(invalid_local_slot(function_name, ip, *slot)),
        },
        Instr::IntStackConstOp { op, rhs } => {
            let Some(value) = frame.stack.last_mut() else {
                return Err(super::err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on IntStackConstOp",
                    function_name,
                    ip,
                ));
            };
            match value {
                Value::Int(lhs) => {
                    match op {
                        IntLocalConstOp::Add => *lhs += *rhs,
                        IntLocalConstOp::Sub => *lhs -= *rhs,
                        IntLocalConstOp::Mul => *lhs *= *rhs,
                        IntLocalConstOp::Div => {
                            if *rhs == 0 {
                                return Err(super::err_at(
                                    VmErrorKind::DivisionByZero,
                                    "division by zero",
                                    function_name,
                                    ip,
                                ));
                            }
                            *lhs /= *rhs;
                        }
                        IntLocalConstOp::Mod => {
                            if *rhs == 0 {
                                return Err(super::err_at(
                                    VmErrorKind::DivisionByZero,
                                    "modulo by zero",
                                    function_name,
                                    ip,
                                ));
                            }
                            *lhs %= *rhs;
                        }
                    }
                    frame.ip += 1;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        Instr::IntStackConstOpToLocal {
            slot,
            stack_op,
            local_op,
            rhs,
        } => match frame.apply_stack_const_op_to_local(*slot, *stack_op, *local_op, *rhs) {
            Some(Ok(())) => {
                frame.ip += 1;
                Ok(true)
            }
            Some(Err(VmErrorKind::StackUnderflow)) => Err(super::err_at(
                VmErrorKind::StackUnderflow,
                "Stack underflow on IntStackConstOpToLocal",
                function_name,
                ip,
            )),
            Some(Err(VmErrorKind::DivisionByZero)) => Err(super::err_at(
                VmErrorKind::DivisionByZero,
                "division by zero",
                function_name,
                ip,
            )),
            Some(Err(VmErrorKind::TypeMismatch)) => Ok(false),
            Some(Err(_)) => Ok(false),
            None => Err(invalid_local_slot(function_name, ip, *slot)),
        },
        Instr::IntLocalLocalOpToLocal { lhs, rhs, dst, op } => {
            match frame.compute_int_local_local_to_local(*lhs, *rhs, *dst, *op) {
                Some(Ok(())) => {
                    frame.ip += 1;
                    Ok(true)
                }
                Some(Err(VmErrorKind::DivisionByZero)) => Err(super::err_at(
                    VmErrorKind::DivisionByZero,
                    match op {
                        IntLocalConstOp::Div => "division by zero",
                        IntLocalConstOp::Mod => "modulo by zero",
                        _ => "division by zero",
                    },
                    function_name,
                    ip,
                )),
                Some(Err(VmErrorKind::TypeMismatch)) => Ok(false),
                Some(Err(_)) => Ok(false),
                None => {
                    let bad = if *lhs >= frame.locals.len() {
                        *lhs
                    } else if *rhs >= frame.locals.len() {
                        *rhs
                    } else {
                        *dst
                    };
                    Err(invalid_local_slot(function_name, ip, bad))
                }
            }
        }
        Instr::StrLen
        | Instr::StrLenLocal(_)
        | Instr::StrIndexOfConst(_)
        | Instr::StrIndexOfLocalConst { .. }
        | Instr::StrSliceConst { .. }
        | Instr::StrSliceLocalConst { .. }
        | Instr::StrContainsConst(_)
        | Instr::StrContainsLocalConst { .. } => {
            strings::handle_hot_string_instr(frame, instr, function_name, ip)
        }
        Instr::Add => {
            let Some(r) = frame.stack.pop() else {
                return Err(super::err_at(
                    VmErrorKind::StackUnderflow,
                    "Add expects rhs",
                    function_name,
                    ip,
                ));
            };
            let Some(l) = frame.stack.last_mut() else {
                frame.stack.push(r);
                return Err(super::err_at(
                    VmErrorKind::StackUnderflow,
                    "Add expects lhs",
                    function_name,
                    ip,
                ));
            };
            match (&mut *l, r) {
                (Value::Int(a), Value::Int(b)) => {
                    *a += b;
                }
                (_, r) => {
                    let l = std::mem::replace(l, Value::Unit);
                    frame.stack.push(l);
                    frame.stack.push(r);
                    arith::add(&mut frame.stack, function_name, ip)?;
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::LteInt => {
            let Some((l, r)) = frame.pop2() else {
                return Err(super::err_at(
                    VmErrorKind::StackUnderflow,
                    "int binary op expects lhs/rhs",
                    function_name,
                    ip,
                ));
            };
            let stack = &mut frame.stack;
            match (l, r) {
                (Value::Int(l), Value::Int(r)) => stack.push(Value::Bool(l <= r)),
                (l, r) => {
                    stack.push(l);
                    stack.push(r);
                    arith::numeric_binop(stack, instr, function_name, ip)?;
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::LtInt => {
            let Some((l, r)) = frame.pop2() else {
                return Err(super::err_at(
                    VmErrorKind::StackUnderflow,
                    "int binary op expects lhs/rhs",
                    function_name,
                    ip,
                ));
            };
            let stack = &mut frame.stack;
            match (l, r) {
                (Value::Int(l), Value::Int(r)) => stack.push(Value::Bool(l < r)),
                (l, r) => {
                    stack.push(l);
                    stack.push(r);
                    arith::numeric_binop(stack, instr, function_name, ip)?;
                }
            }
            frame.ip += 1;
            Ok(true)
        }
        Instr::Jump(target) => {
            frame.ip = control_flow::jump(*target);
            Ok(true)
        }
        Instr::JumpIfLocalLtConst { slot, rhs, target } => {
            let Some(value) = frame.locals.get(*slot) else {
                return Err(invalid_local_slot(function_name, ip, *slot));
            };
            match value {
                Value::Int(current) => {
                    if *current < *rhs {
                        frame.ip += 1;
                    } else {
                        frame.ip = *target;
                    }
                    Ok(true)
                }
                _ => Err(super::err_at(
                    VmErrorKind::TypeMismatch,
                    "JumpIfLocalLtConst expects Int local",
                    function_name,
                    ip,
                )),
            }
        }
        Instr::JumpIfFalse(target) => {
            let Some(cond) = frame.stack.pop() else {
                return Err(super::err_at(
                    VmErrorKind::TypeMismatch,
                    "JumpIfFalse expects Bool",
                    function_name,
                    ip,
                ));
            };
            match cond {
                Value::Bool(false) => frame.ip = *target,
                Value::Bool(true) => frame.ip += 1,
                _ => {
                    return Err(super::err_at(
                        VmErrorKind::TypeMismatch,
                        "JumpIfFalse expects Bool",
                        function_name,
                        ip,
                    ));
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
