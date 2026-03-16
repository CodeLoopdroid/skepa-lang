use crate::{RtArray, RtString, RtVec};

#[derive(Debug, Clone, PartialEq)]
pub struct RtStruct {
    pub name: String,
    pub fields: Vec<RtValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RtValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(RtString),
    Array(RtArray),
    Vec(RtVec),
    Struct(RtStruct),
    Unit,
}

impl RtValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::Bool(_) => "Bool",
            Self::String(_) => "String",
            Self::Array(_) => "Array",
            Self::Vec(_) => "Vec",
            Self::Struct(_) => "Struct",
            Self::Unit => "Void",
        }
    }
}
