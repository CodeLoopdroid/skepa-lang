use std::rc::Rc;

use super::super::{Instr, IntLocalConstOp, StructShape, Value};

pub(super) struct Reader<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) idx: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn read_exact(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.idx + n > self.bytes.len() {
            return Err("Unexpected EOF while decoding bytecode".to_string());
        }
        let s = &self.bytes[self.idx..self.idx + n];
        self.idx += n;
        Ok(s)
    }

    pub(super) fn read_u8(&mut self) -> Result<u8, String> {
        Ok(self.read_exact(1)?[0])
    }

    pub(super) fn read_u32(&mut self) -> Result<u32, String> {
        let b = self.read_exact(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_i64(&mut self) -> Result<i64, String> {
        let b = self.read_exact(8)?;
        Ok(i64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        let b = self.read_exact(8)?;
        Ok(f64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn read_bool(&mut self) -> Result<bool, String> {
        Ok(self.read_u8()? != 0)
    }

    pub(super) fn read_str(&mut self) -> Result<String, String> {
        let n = self.read_u32()? as usize;
        let b = self.read_exact(n)?;
        String::from_utf8(b.to_vec()).map_err(|e| e.to_string())
    }
}

fn decode_value(rd: &mut Reader<'_>) -> Result<Value, String> {
    match rd.read_u8()? {
        0 => Ok(Value::Int(rd.read_i64()?)),
        1 => Ok(Value::Float(rd.read_f64()?)),
        2 => Ok(Value::Bool(rd.read_bool()?)),
        3 => Ok(Value::String(Rc::<str>::from(rd.read_str()?))),
        4 => {
            let n = rd.read_u32()? as usize;
            let mut items = Vec::with_capacity(n);
            for _ in 0..n {
                items.push(decode_value(rd)?);
            }
            Ok(Value::Array(Rc::<[Value]>::from(items)))
        }
        5 => Ok(Value::Function(Rc::<str>::from(rd.read_str()?))),
        6 => Ok(Value::Unit),
        7 => {
            let name = rd.read_str()?;
            let field_names_len = rd.read_u32()? as usize;
            let mut field_names = Vec::with_capacity(field_names_len);
            for _ in 0..field_names_len {
                field_names.push(rd.read_str()?);
            }
            let n = rd.read_u32()? as usize;
            let mut fields = Vec::with_capacity(n);
            for _ in 0..n {
                fields.push(decode_value(rd)?);
            }
            Ok(Value::Struct {
                shape: Rc::new(StructShape {
                    name,
                    field_names: Rc::<[String]>::from(field_names),
                }),
                fields: Rc::<[Value]>::from(fields),
            })
        }
        8 => Ok(Value::FunctionIdx(rd.read_u32()? as usize)),
        t => Err(format!("Unknown value tag {t}")),
    }
}

pub(super) fn decode_instr(rd: &mut Reader<'_>) -> Result<Instr, String> {
    Ok(match rd.read_u8()? {
        0 => Instr::LoadConst(decode_value(rd)?),
        1 => Instr::LoadLocal(rd.read_u32()? as usize),
        2 => Instr::StoreLocal(rd.read_u32()? as usize),
        48 => Instr::AddLocalToLocal {
            dst: rd.read_u32()? as usize,
            src: rd.read_u32()? as usize,
        },
        49 => Instr::AddConstToLocal {
            slot: rd.read_u32()? as usize,
            rhs: rd.read_i64()?,
        },
        62 => Instr::IntLocalLocalOp {
            lhs: rd.read_u32()? as usize,
            rhs: rd.read_u32()? as usize,
            op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntLocalLocalOp tag {other}")),
            },
        },
        61 => Instr::IntLocalConstOp {
            slot: rd.read_u32()? as usize,
            op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntLocalConstOp tag {other}")),
            },
            rhs: rd.read_i64()?,
        },
        65 => Instr::IntLocalConstOpToLocal {
            src: rd.read_u32()? as usize,
            dst: rd.read_u32()? as usize,
            op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntLocalConstOpToLocal tag {other}")),
            },
            rhs: rd.read_i64()?,
        },
        63 => Instr::IntStackOpToLocal {
            slot: rd.read_u32()? as usize,
            op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntStackOpToLocal tag {other}")),
            },
        },
        67 => Instr::IntStackConstOp {
            op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntStackConstOp tag {other}")),
            },
            rhs: rd.read_i64()?,
        },
        71 => Instr::IntStackConstOpToLocal {
            slot: rd.read_u32()? as usize,
            stack_op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntStackConstOpToLocal stack tag {other}")),
            },
            local_op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntStackConstOpToLocal local tag {other}")),
            },
            rhs: rd.read_i64()?,
        },
        66 => Instr::IntLocalLocalOpToLocal {
            lhs: rd.read_u32()? as usize,
            rhs: rd.read_u32()? as usize,
            dst: rd.read_u32()? as usize,
            op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => return Err(format!("Unknown IntLocalLocalOpToLocal tag {other}")),
            },
        },
        3 => Instr::LoadGlobal(rd.read_u32()? as usize),
        4 => Instr::StoreGlobal(rd.read_u32()? as usize),
        5 => Instr::Pop,
        6 => Instr::NegInt,
        7 => Instr::NotBool,
        8 => Instr::Add,
        9 => Instr::SubInt,
        10 => Instr::MulInt,
        11 => Instr::DivInt,
        12 => Instr::ModInt,
        13 => Instr::Eq,
        14 => Instr::Neq,
        15 => Instr::LtInt,
        16 => Instr::LteInt,
        17 => Instr::GtInt,
        18 => Instr::GteInt,
        19 => Instr::AndBool,
        20 => Instr::OrBool,
        21 => Instr::Jump(rd.read_u32()? as usize),
        22 => Instr::JumpIfFalse(rd.read_u32()? as usize),
        23 => Instr::JumpIfTrue(rd.read_u32()? as usize),
        46 => Instr::JumpIfLocalLtConst {
            slot: rd.read_u32()? as usize,
            rhs: rd.read_i64()?,
            target: rd.read_u32()? as usize,
        },
        24 => Instr::Call {
            name: rd.read_str()?,
            argc: rd.read_u32()? as usize,
        },
        38 => Instr::CallIdx {
            idx: rd.read_u32()? as usize,
            argc: rd.read_u32()? as usize,
        },
        47 => Instr::CallIdxAddConst(rd.read_i64()?),
        51 => Instr::CallIdxStructFieldAdd(rd.read_u32()? as usize),
        37 => Instr::CallValue {
            argc: rd.read_u32()? as usize,
        },
        36 => Instr::CallMethod {
            name: rd.read_str()?,
            argc: rd.read_u32()? as usize,
        },
        43 => Instr::CallMethodId {
            id: rd.read_u32()? as usize,
            argc: rd.read_u32()? as usize,
        },
        25 => Instr::CallBuiltin {
            package: rd.read_str()?,
            name: rd.read_str()?,
            argc: rd.read_u32()? as usize,
        },
        41 => Instr::CallBuiltinId {
            id: rd.read_u32()? as u16,
            argc: rd.read_u32()? as usize,
        },
        52 => Instr::StrLen,
        56 => Instr::StrLenLocal(rd.read_u32()? as usize),
        53 => Instr::StrIndexOfConst(Rc::<str>::from(rd.read_str()?)),
        57 => Instr::StrIndexOfLocalConst {
            slot: rd.read_u32()? as usize,
            needle: Rc::<str>::from(rd.read_str()?),
        },
        54 => Instr::StrSliceConst {
            start: rd.read_i64()?,
            end: rd.read_i64()?,
        },
        58 => Instr::StrSliceLocalConst {
            slot: rd.read_u32()? as usize,
            start: rd.read_i64()?,
            end: rd.read_i64()?,
        },
        55 => Instr::StrContainsConst(Rc::<str>::from(rd.read_str()?)),
        59 => Instr::StrContainsLocalConst {
            slot: rd.read_u32()? as usize,
            needle: Rc::<str>::from(rd.read_str()?),
        },
        26 => Instr::MakeArray(rd.read_u32()? as usize),
        27 => Instr::MakeArrayRepeat(rd.read_u32()? as usize),
        28 => Instr::ArrayGet,
        60 => Instr::ArrayGetLocal(rd.read_u32()? as usize),
        29 => Instr::ArraySet,
        45 => Instr::ArraySetLocal(rd.read_u32()? as usize),
        50 => Instr::ArrayIncLocal(rd.read_u32()? as usize),
        30 => Instr::ArraySetChain(rd.read_u32()? as usize),
        31 => Instr::ArrayLen,
        32 => Instr::Return,
        33 => {
            let name = rd.read_str()?;
            let n = rd.read_u32()? as usize;
            let mut fields = Vec::with_capacity(n);
            for _ in 0..n {
                fields.push(rd.read_str()?);
            }
            Instr::MakeStruct { name, fields }
        }
        44 => Instr::MakeStructId {
            id: rd.read_u32()? as usize,
        },
        34 => Instr::StructGet(rd.read_str()?),
        64 => Instr::StructGetLocalSlot {
            slot: rd.read_u32()? as usize,
            field_slot: rd.read_u32()? as usize,
        },
        68 => Instr::StructGetLocalSlotAddToLocal {
            struct_slot: rd.read_u32()? as usize,
            field_slot: rd.read_u32()? as usize,
            dst: rd.read_u32()? as usize,
        },
        72 => Instr::StructFieldAddMulFieldModLocalToLocal {
            struct_slot: rd.read_u32()? as usize,
            arg_slot: rd.read_u32()? as usize,
            arg_op: match rd.read_u8()? {
                0 => IntLocalConstOp::Add,
                1 => IntLocalConstOp::Sub,
                2 => IntLocalConstOp::Mul,
                3 => IntLocalConstOp::Div,
                4 => IntLocalConstOp::Mod,
                other => {
                    return Err(format!(
                        "Unknown StructFieldAddMulFieldModLocalToLocal arg tag {other}"
                    ));
                }
            },
            arg_rhs: rd.read_i64()?,
            lhs_field_slot: rd.read_u32()? as usize,
            rhs_field_slot: rd.read_u32()? as usize,
            mul: rd.read_i64()?,
            modulo: rd.read_i64()?,
            dst: rd.read_u32()? as usize,
        },
        39 => Instr::StructGetSlot(rd.read_u32()? as usize),
        35 => {
            let n = rd.read_u32()? as usize;
            let mut path = Vec::with_capacity(n);
            for _ in 0..n {
                path.push(rd.read_str()?);
            }
            Instr::StructSetPath(path)
        }
        40 => {
            let n = rd.read_u32()? as usize;
            let mut path = Vec::with_capacity(n);
            for _ in 0..n {
                path.push(rd.read_u32()? as usize);
            }
            Instr::StructSetPathSlots(path)
        }
        t => return Err(format!("Unknown instruction tag {t}")),
    })
}
