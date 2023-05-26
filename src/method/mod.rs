use crate::access_flag::AccessFlag;
use crate::annotation::Annotation;
use crate::instruction::Instruction;
use crate::r#type::Type;

mod jimple;
mod optimization;
mod smali;

#[derive(Debug, PartialEq)]
pub struct MethodParameter {
    pub parameter_type: Type,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, PartialEq)]
pub struct Method {
    pub name: String,
    pub visibility: Vec<AccessFlag>,
    pub parameters: Vec<MethodParameter>,
    pub return_type: Type,
    pub annotations: Vec<Annotation>,
    pub instructions: Vec<Instruction>,
}
