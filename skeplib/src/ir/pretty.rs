use std::fmt::{self, Display, Formatter};

use crate::ir::{BasicBlock, IrFunction, IrProgram, Terminator};

pub struct PrettyIr<'a> {
    pub program: &'a IrProgram,
}

impl<'a> PrettyIr<'a> {
    pub fn new(program: &'a IrProgram) -> Self {
        Self { program }
    }
}

impl Display for PrettyIr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt_structs(f, self.program)?;
        fmt_globals(f, self.program)?;
        fmt_module_init(f, self.program)?;
        for function in &self.program.functions {
            fmt_function(f, function)?;
        }
        Ok(())
    }
}

fn fmt_structs(f: &mut Formatter<'_>, program: &IrProgram) -> fmt::Result {
    if program.structs.is_empty() {
        return Ok(());
    }

    writeln!(f, "structs {{")?;
    for strukt in &program.structs {
        write!(f, "  {}(", strukt.name)?;
        for (idx, field) in strukt.fields.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {:?}", field.name, field.ty)?;
        }
        writeln!(f, ")")?;
    }
    writeln!(f, "}}")?;
    Ok(())
}

fn fmt_globals(f: &mut Formatter<'_>, program: &IrProgram) -> fmt::Result {
    if program.globals.is_empty() {
        return Ok(());
    }

    writeln!(f, "globals {{")?;
    for global in &program.globals {
        writeln!(f, "  {}: {:?} = {:?}", global.name, global.ty, global.init)?;
    }
    writeln!(f, "}}")?;
    Ok(())
}

fn fmt_module_init(f: &mut Formatter<'_>, program: &IrProgram) -> fmt::Result {
    if let Some(module_init) = &program.module_init {
        writeln!(f, "module_init {:?}", module_init.function)?;
    }
    Ok(())
}

fn fmt_function(f: &mut Formatter<'_>, function: &IrFunction) -> fmt::Result {
    writeln!(f, "fn {} -> {:?} {{", function.name, function.ret_ty)?;
    for block in &function.blocks {
        fmt_block(f, block)?;
    }
    writeln!(f, "}}")
}

fn fmt_block(f: &mut Formatter<'_>, block: &BasicBlock) -> fmt::Result {
    writeln!(f, "  {}:", block.name)?;
    for instr in &block.instrs {
        writeln!(f, "    {:?}", instr)?;
    }
    match &block.terminator {
        Terminator::Unreachable => writeln!(f, "    unreachable"),
        other => writeln!(f, "    {:?}", other),
    }
}
