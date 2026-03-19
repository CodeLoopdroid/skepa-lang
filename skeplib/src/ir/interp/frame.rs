use std::collections::HashMap;

use crate::ir::{IrFunction, Operand};
use skepart::RtValue;

use super::{IrInterpError, IrInterpreter};

pub(super) struct Frame {
    pub(super) locals: HashMap<usize, RtValue>,
    pub(super) temps: HashMap<crate::ir::TempId, RtValue>,
}

impl Frame {
    pub(super) fn new(func: &IrFunction, args: Vec<RtValue>) -> Self {
        let mut locals = HashMap::new();
        for ((_, value), local) in func
            .params
            .iter()
            .zip(args)
            .zip(func.locals.iter().take(func.params.len()))
        {
            locals.insert(local.id.0, value);
        }
        Self {
            locals,
            temps: HashMap::new(),
        }
    }

    pub(super) fn read_operand(
        &self,
        operand: &Operand,
        globals: &[RtValue],
    ) -> Result<RtValue, IrInterpError> {
        match operand {
            Operand::Const(value) => Ok(IrInterpreter::const_to_value(value)),
            Operand::Temp(id) => self
                .temps
                .get(id)
                .cloned()
                .ok_or(IrInterpError::InvalidOperand("temp missing")),
            Operand::Local(id) => self
                .locals
                .get(&id.0)
                .cloned()
                .ok_or(IrInterpError::InvalidOperand("local missing")),
            Operand::Global(id) => globals
                .get(id.0)
                .cloned()
                .ok_or(IrInterpError::InvalidOperand("global missing")),
        }
    }
}
