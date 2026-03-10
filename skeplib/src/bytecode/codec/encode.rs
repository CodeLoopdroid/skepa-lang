use super::super::{Instr, IntLocalConstOp, Value};

pub(super) fn write_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

pub(super) fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_i64(out: &mut Vec<u8>, v: i64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_f64(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_bool(out: &mut Vec<u8>, v: bool) {
    write_u8(out, if v { 1 } else { 0 });
}

pub(super) fn write_str(out: &mut Vec<u8>, s: &str) {
    write_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

fn encode_value(v: &Value, out: &mut Vec<u8>) {
    match v {
        Value::Int(n) => {
            write_u8(out, 0);
            write_i64(out, *n);
        }
        Value::Float(n) => {
            write_u8(out, 1);
            write_f64(out, *n);
        }
        Value::Bool(b) => {
            write_u8(out, 2);
            write_bool(out, *b);
        }
        Value::String(s) => {
            write_u8(out, 3);
            write_str(out, s);
        }
        Value::Array(items) => {
            write_u8(out, 4);
            write_u32(out, items.len() as u32);
            for item in items.iter() {
                encode_value(item, out);
            }
        }
        Value::VecHandle(_) => {
            panic!("VecHandle is a runtime-only value and cannot be serialized")
        }
        Value::Function(name) => {
            write_u8(out, 5);
            write_str(out, name);
        }
        Value::FunctionIdx(idx) => {
            write_u8(out, 8);
            write_u32(out, *idx as u32);
        }
        Value::Unit => write_u8(out, 6),
        Value::Struct { shape, fields } => {
            write_u8(out, 7);
            write_str(out, &shape.name);
            write_u32(out, shape.field_names.len() as u32);
            for field_name in shape.field_names.iter() {
                write_str(out, field_name);
            }
            write_u32(out, fields.len() as u32);
            for v in fields.iter() {
                encode_value(v, out);
            }
        }
    }
}

pub(super) fn encode_instr(i: &Instr, out: &mut Vec<u8>) {
    match i {
        Instr::LoadConst(v) => {
            write_u8(out, 0);
            encode_value(v, out);
        }
        Instr::LoadLocal(s) => {
            write_u8(out, 1);
            write_u32(out, *s as u32);
        }
        Instr::StoreLocal(s) => {
            write_u8(out, 2);
            write_u32(out, *s as u32);
        }
        Instr::AddLocalToLocal { dst, src } => {
            write_u8(out, 48);
            write_u32(out, *dst as u32);
            write_u32(out, *src as u32);
        }
        Instr::AddConstToLocal { slot, rhs } => {
            write_u8(out, 49);
            write_u32(out, *slot as u32);
            write_i64(out, *rhs);
        }
        Instr::IntLocalLocalOp { lhs, rhs, op } => {
            write_u8(out, 62);
            write_u32(out, *lhs as u32);
            write_u32(out, *rhs as u32);
            write_u8(
                out,
                match op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
        }
        Instr::IntLocalConstOp { slot, op, rhs } => {
            write_u8(out, 61);
            write_u32(out, *slot as u32);
            write_u8(
                out,
                match op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
            write_i64(out, *rhs);
        }
        Instr::IntLocalConstOpToLocal { src, dst, op, rhs } => {
            write_u8(out, 65);
            write_u32(out, *src as u32);
            write_u32(out, *dst as u32);
            write_u8(
                out,
                match op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
            write_i64(out, *rhs);
        }
        Instr::IntStackOpToLocal { slot, op } => {
            write_u8(out, 63);
            write_u32(out, *slot as u32);
            write_u8(
                out,
                match op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
        }
        Instr::IntStackConstOp { op, rhs } => {
            write_u8(out, 67);
            write_u8(
                out,
                match op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
            write_i64(out, *rhs);
        }
        Instr::IntStackConstOpToLocal {
            slot,
            stack_op,
            local_op,
            rhs,
        } => {
            write_u8(out, 71);
            write_u32(out, *slot as u32);
            write_u8(
                out,
                match stack_op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
            write_u8(
                out,
                match local_op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
            write_i64(out, *rhs);
        }
        Instr::IntLocalLocalOpToLocal { lhs, rhs, dst, op } => {
            write_u8(out, 66);
            write_u32(out, *lhs as u32);
            write_u32(out, *rhs as u32);
            write_u32(out, *dst as u32);
            write_u8(
                out,
                match op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
        }
        Instr::LoadGlobal(s) => {
            write_u8(out, 3);
            write_u32(out, *s as u32);
        }
        Instr::StoreGlobal(s) => {
            write_u8(out, 4);
            write_u32(out, *s as u32);
        }
        Instr::Pop => write_u8(out, 5),
        Instr::NegInt => write_u8(out, 6),
        Instr::NotBool => write_u8(out, 7),
        Instr::Add => write_u8(out, 8),
        Instr::SubInt => write_u8(out, 9),
        Instr::MulInt => write_u8(out, 10),
        Instr::DivInt => write_u8(out, 11),
        Instr::ModInt => write_u8(out, 12),
        Instr::Eq => write_u8(out, 13),
        Instr::Neq => write_u8(out, 14),
        Instr::LtInt => write_u8(out, 15),
        Instr::LteInt => write_u8(out, 16),
        Instr::GtInt => write_u8(out, 17),
        Instr::GteInt => write_u8(out, 18),
        Instr::AndBool => write_u8(out, 19),
        Instr::OrBool => write_u8(out, 20),
        Instr::Jump(t) => {
            write_u8(out, 21);
            write_u32(out, *t as u32);
        }
        Instr::JumpIfFalse(t) => {
            write_u8(out, 22);
            write_u32(out, *t as u32);
        }
        Instr::JumpIfTrue(t) => {
            write_u8(out, 23);
            write_u32(out, *t as u32);
        }
        Instr::JumpIfLocalLtConst { slot, rhs, target } => {
            write_u8(out, 46);
            write_u32(out, *slot as u32);
            write_i64(out, *rhs);
            write_u32(out, *target as u32);
        }
        Instr::Call { name, argc } => {
            write_u8(out, 24);
            write_str(out, name);
            write_u32(out, *argc as u32);
        }
        Instr::CallIdx { idx, argc } => {
            write_u8(out, 38);
            write_u32(out, *idx as u32);
            write_u32(out, *argc as u32);
        }
        Instr::CallIdxAddConst(rhs) => {
            write_u8(out, 47);
            write_i64(out, *rhs);
        }
        Instr::CallIdxStructFieldAdd(slot) => {
            write_u8(out, 51);
            write_u32(out, *slot as u32);
        }
        Instr::CallValue { argc } => {
            write_u8(out, 37);
            write_u32(out, *argc as u32);
        }
        Instr::CallMethod { name, argc } => {
            write_u8(out, 36);
            write_str(out, name);
            write_u32(out, *argc as u32);
        }
        Instr::CallMethodId { id, argc } => {
            write_u8(out, 43);
            write_u32(out, *id as u32);
            write_u32(out, *argc as u32);
        }
        Instr::CallBuiltin {
            package,
            name,
            argc,
        } => {
            write_u8(out, 25);
            write_str(out, package);
            write_str(out, name);
            write_u32(out, *argc as u32);
        }
        Instr::CallBuiltinId { id, argc } => {
            write_u8(out, 41);
            write_u32(out, *id as u32);
            write_u32(out, *argc as u32);
        }
        Instr::StrLen => write_u8(out, 52),
        Instr::StrLenLocal(slot) => {
            write_u8(out, 56);
            write_u32(out, *slot as u32);
        }
        Instr::StrIndexOfConst(needle) => {
            write_u8(out, 53);
            write_str(out, needle);
        }
        Instr::StrIndexOfLocalConst { slot, needle } => {
            write_u8(out, 57);
            write_u32(out, *slot as u32);
            write_str(out, needle);
        }
        Instr::StrSliceConst { start, end } => {
            write_u8(out, 54);
            write_i64(out, *start);
            write_i64(out, *end);
        }
        Instr::StrSliceLocalConst { slot, start, end } => {
            write_u8(out, 58);
            write_u32(out, *slot as u32);
            write_i64(out, *start);
            write_i64(out, *end);
        }
        Instr::StrContainsConst(needle) => {
            write_u8(out, 55);
            write_str(out, needle);
        }
        Instr::StrContainsLocalConst { slot, needle } => {
            write_u8(out, 59);
            write_u32(out, *slot as u32);
            write_str(out, needle);
        }
        Instr::MakeArray(n) => {
            write_u8(out, 26);
            write_u32(out, *n as u32);
        }
        Instr::MakeArrayRepeat(n) => {
            write_u8(out, 27);
            write_u32(out, *n as u32);
        }
        Instr::ArrayGet => write_u8(out, 28),
        Instr::ArrayGetLocal(slot) => {
            write_u8(out, 60);
            write_u32(out, *slot as u32);
        }
        Instr::ArraySet => write_u8(out, 29),
        Instr::ArraySetLocal(slot) => {
            write_u8(out, 45);
            write_u32(out, *slot as u32);
        }
        Instr::ArrayIncLocal(slot) => {
            write_u8(out, 50);
            write_u32(out, *slot as u32);
        }
        Instr::ArraySetChain(n) => {
            write_u8(out, 30);
            write_u32(out, *n as u32);
        }
        Instr::ArrayLen => write_u8(out, 31),
        Instr::Return => write_u8(out, 32),
        Instr::MakeStruct { name, fields } => {
            write_u8(out, 33);
            write_str(out, name);
            write_u32(out, fields.len() as u32);
            for f in fields {
                write_str(out, f);
            }
        }
        Instr::MakeStructId { id } => {
            write_u8(out, 44);
            write_u32(out, *id as u32);
        }
        Instr::StructGet(field) => {
            write_u8(out, 34);
            write_str(out, field);
        }
        Instr::StructGetLocalSlot { slot, field_slot } => {
            write_u8(out, 64);
            write_u32(out, *slot as u32);
            write_u32(out, *field_slot as u32);
        }
        Instr::StructGetLocalSlotAddToLocal {
            struct_slot,
            field_slot,
            dst,
        } => {
            write_u8(out, 68);
            write_u32(out, *struct_slot as u32);
            write_u32(out, *field_slot as u32);
            write_u32(out, *dst as u32);
        }
        Instr::StructFieldAddMulFieldModLocalToLocal {
            struct_slot,
            arg_slot,
            arg_op,
            arg_rhs,
            lhs_field_slot,
            rhs_field_slot,
            mul,
            modulo,
            dst,
        } => {
            write_u8(out, 72);
            write_u32(out, *struct_slot as u32);
            write_u32(out, *arg_slot as u32);
            write_u8(
                out,
                match arg_op {
                    IntLocalConstOp::Add => 0,
                    IntLocalConstOp::Sub => 1,
                    IntLocalConstOp::Mul => 2,
                    IntLocalConstOp::Div => 3,
                    IntLocalConstOp::Mod => 4,
                },
            );
            write_i64(out, *arg_rhs);
            write_u32(out, *lhs_field_slot as u32);
            write_u32(out, *rhs_field_slot as u32);
            write_i64(out, *mul);
            write_i64(out, *modulo);
            write_u32(out, *dst as u32);
        }
        Instr::StructGetSlot(slot) => {
            write_u8(out, 39);
            write_u32(out, *slot as u32);
        }
        Instr::StructSetPath(path) => {
            write_u8(out, 35);
            write_u32(out, path.len() as u32);
            for p in path {
                write_str(out, p);
            }
        }
        Instr::StructSetPathSlots(path) => {
            write_u8(out, 40);
            write_u32(out, path.len() as u32);
            for slot in path {
                write_u32(out, *slot as u32);
            }
        }
    }
}
