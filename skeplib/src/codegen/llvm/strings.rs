use crate::ir::{ConstValue, Instr, IrProgram, Operand, Terminator};
use std::collections::HashMap;

pub fn collect_string_literals(program: &IrProgram) -> HashMap<String, String> {
    let mut literals = HashMap::new();
    let mut index = 0usize;
    for func in &program.functions {
        for block in &func.blocks {
            for instr in &block.instrs {
                collect_instr_string_literals(instr, &mut literals, &mut index);
            }
            if let Terminator::Return(Some(Operand::Const(ConstValue::String(value)))) =
                &block.terminator
            {
                literals.entry(value.clone()).or_insert_with(|| {
                    let name = format!("@.str.{index}");
                    index += 1;
                    name
                });
            }
        }
    }
    literals
}

pub fn encode_c_string(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'\\' => out.push_str("\\5C"),
            b'"' => out.push_str("\\22"),
            32..=126 => out.push(byte as char),
            _ => out.push_str(&format!("\\{:02X}", byte)),
        }
    }
    out.push_str("\\00");
    out
}

pub fn runtime_string_symbol(raw_symbol: &str) -> String {
    raw_symbol.replacen("@.str.", "@.rtstr.", 1)
}

fn collect_instr_string_literals(
    instr: &Instr,
    literals: &mut HashMap<String, String>,
    index: &mut usize,
) {
    let mut add_operand = |operand: &Operand| {
        if let Operand::Const(ConstValue::String(value)) = operand {
            literals.entry(value.clone()).or_insert_with(|| {
                let name = format!("@.str.{index}");
                *index += 1;
                name
            });
        }
    };
    match instr {
        Instr::Const {
            value: ConstValue::String(value),
            ..
        } => {
            literals.entry(value.clone()).or_insert_with(|| {
                let name = format!("@.str.{index}");
                *index += 1;
                name
            });
        }
        Instr::Copy { src, .. } => add_operand(src),
        Instr::Unary { operand, .. } => add_operand(operand),
        Instr::Binary { left, right, .. } | Instr::Compare { left, right, .. } => {
            add_operand(left);
            add_operand(right);
        }
        Instr::StoreGlobal { value, .. } | Instr::StoreLocal { value, .. } => add_operand(value),
        Instr::CallDirect { args, .. } => {
            for arg in args {
                add_operand(arg);
            }
        }
        Instr::CallBuiltin { builtin, args, .. } => {
            for arg in args {
                add_operand(arg);
            }
            literals.entry(builtin.package.clone()).or_insert_with(|| {
                let name = format!("@.str.{index}");
                *index += 1;
                name
            });
            literals.entry(builtin.name.clone()).or_insert_with(|| {
                let name = format!("@.str.{index}");
                *index += 1;
                name
            });
        }
        Instr::CallIndirect { callee, args, .. } => {
            add_operand(callee);
            for arg in args {
                add_operand(arg);
            }
        }
        Instr::MakeArray { items, .. } => {
            for item in items {
                add_operand(item);
            }
        }
        Instr::MakeArrayRepeat { value, .. } => add_operand(value),
        Instr::ArrayGet { array, index, .. }
        | Instr::VecGet {
            vec: array, index, ..
        } => {
            add_operand(array);
            add_operand(index);
        }
        Instr::StructGet { base, .. } => add_operand(base),
        Instr::ArraySet {
            array,
            index,
            value,
            ..
        }
        | Instr::VecSet {
            vec: array,
            index,
            value,
            ..
        } => {
            add_operand(array);
            add_operand(index);
            add_operand(value);
        }
        Instr::VecPush { vec, value, .. } => {
            add_operand(vec);
            add_operand(value);
        }
        Instr::VecDelete { vec, index, .. } => {
            add_operand(vec);
            add_operand(index);
        }
        Instr::VecLen { vec, .. } => add_operand(vec),
        Instr::MakeStruct { fields, .. } => {
            for field in fields {
                add_operand(field);
            }
        }
        Instr::StructSet { base, value, .. } => {
            add_operand(base);
            add_operand(value);
        }
        Instr::MakeClosure { .. } => {}
        _ => {}
    }
}
