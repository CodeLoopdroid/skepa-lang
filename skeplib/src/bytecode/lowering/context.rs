use std::collections::HashMap;

use crate::ast::TypeName;

use super::Instr;

#[derive(Debug, Clone, Default)]
pub(super) struct LoopCtx {
    pub(super) continue_target: usize,
    pub(super) break_jumps: Vec<usize>,
}

#[derive(Default)]
pub(super) struct FnCtx {
    locals: HashMap<String, usize>,
    local_named_types: HashMap<String, String>,
    local_primitive_types: HashMap<String, PrimitiveType>,
    pub(super) next_local: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PrimitiveType {
    Int,
    Float,
    Bool,
    String,
    Void,
}

pub(super) enum SpecializedArrayAssign {
    IncLocal,
}

impl SpecializedArrayAssign {
    pub(super) fn with_slot(self, slot: usize) -> Instr {
        match self {
            Self::IncLocal => Instr::ArrayIncLocal(slot),
        }
    }
}

impl FnCtx {
    pub(super) fn alloc_local(&mut self, name: String) -> usize {
        let slot = self.next_local;
        self.next_local += 1;
        self.locals.insert(name, slot);
        slot
    }

    pub(super) fn alloc_local_with_type(&mut self, name: String, ty: &TypeName) -> usize {
        match ty {
            TypeName::Named(type_name) => self.alloc_local_with_named_type(name, type_name.clone()),
            TypeName::Int => self.alloc_local_with_primitive_type(name, PrimitiveType::Int),
            TypeName::Float => self.alloc_local_with_primitive_type(name, PrimitiveType::Float),
            TypeName::Bool => self.alloc_local_with_primitive_type(name, PrimitiveType::Bool),
            TypeName::String => self.alloc_local_with_primitive_type(name, PrimitiveType::String),
            TypeName::Void => self.alloc_local_with_primitive_type(name, PrimitiveType::Void),
            TypeName::Array { .. } | TypeName::Vec { .. } | TypeName::Fn { .. } => {
                self.alloc_local(name)
            }
        }
    }

    pub(super) fn alloc_local_with_primitive_type(
        &mut self,
        name: String,
        ty: PrimitiveType,
    ) -> usize {
        let slot = self.alloc_local(name.clone());
        self.local_primitive_types.insert(name, ty);
        slot
    }

    pub(super) fn alloc_local_with_named_type(&mut self, name: String, type_name: String) -> usize {
        let slot = self.alloc_local(name.clone());
        self.local_named_types.insert(name, type_name);
        slot
    }

    pub(super) fn alloc_anonymous_local(&mut self) -> usize {
        let slot = self.next_local;
        self.next_local += 1;
        slot
    }

    pub(super) fn lookup(&self, name: &str) -> Option<usize> {
        self.locals.get(name).copied()
    }

    pub(super) fn named_type(&self, name: &str) -> Option<String> {
        self.local_named_types.get(name).cloned()
    }

    pub(super) fn primitive_type(&self, name: &str) -> Option<PrimitiveType> {
        self.local_primitive_types.get(name).copied()
    }
}
