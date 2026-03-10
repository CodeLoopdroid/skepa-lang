use std::collections::HashMap;
use std::rc::Rc;

mod decode;
mod encode;

use self::decode::{Reader, decode_instr};
use self::encode::{encode_instr, write_str, write_u32};
use super::{BytecodeModule, FunctionChunk, StructShape};

impl BytecodeModule {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"SKBC");
        write_u32(&mut out, 4);
        write_u32(&mut out, self.functions.len() as u32);
        write_u32(&mut out, self.method_names.len() as u32);
        for name in &self.method_names {
            write_str(&mut out, name);
        }
        write_u32(&mut out, self.struct_shapes.len() as u32);
        for shape in &self.struct_shapes {
            write_str(&mut out, &shape.name);
            write_u32(&mut out, shape.field_names.len() as u32);
            for field_name in shape.field_names.iter() {
                write_str(&mut out, field_name);
            }
        }
        let mut funcs: Vec<_> = self.functions.values().collect();
        funcs.sort_by(|a, b| a.name.cmp(&b.name));
        for f in funcs {
            write_str(&mut out, &f.name);
            write_u32(&mut out, f.locals_count as u32);
            write_u32(&mut out, f.param_count as u32);
            write_u32(&mut out, f.code.len() as u32);
            for instr in &f.code {
                encode_instr(instr, &mut out);
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut rd = Reader { bytes, idx: 0 };
        let magic = rd.read_exact(4)?;
        if magic != b"SKBC" {
            return Err("Invalid bytecode magic header".to_string());
        }
        let version = rd.read_u32()?;
        if version != 4 {
            return Err(format!("Unsupported bytecode version {version}"));
        }
        let funcs_len = rd.read_u32()? as usize;
        let method_names_len = rd.read_u32()? as usize;
        let mut method_names = Vec::with_capacity(method_names_len);
        for _ in 0..method_names_len {
            method_names.push(rd.read_str()?);
        }
        let struct_shapes_len = rd.read_u32()? as usize;
        let mut struct_shapes = Vec::with_capacity(struct_shapes_len);
        for _ in 0..struct_shapes_len {
            let name = rd.read_str()?;
            let field_names_len = rd.read_u32()? as usize;
            let mut field_names = Vec::with_capacity(field_names_len);
            for _ in 0..field_names_len {
                field_names.push(rd.read_str()?);
            }
            struct_shapes.push(StructShape {
                name,
                field_names: Rc::<[String]>::from(field_names),
            });
        }
        let mut functions = HashMap::new();
        for _ in 0..funcs_len {
            let name = rd.read_str()?;
            let locals_count = rd.read_u32()? as usize;
            let param_count = rd.read_u32()? as usize;
            let code_len = rd.read_u32()? as usize;
            let mut code = Vec::with_capacity(code_len);
            for _ in 0..code_len {
                code.push(decode_instr(&mut rd)?);
            }
            functions.insert(
                name.clone(),
                FunctionChunk {
                    name,
                    code,
                    locals_count,
                    param_count,
                },
            );
        }
        Ok(Self {
            functions,
            method_names,
            struct_shapes,
        })
    }
}
