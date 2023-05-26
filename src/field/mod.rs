use crate::access_flag::AccessFlag;
use crate::annotation::Annotation;
use crate::literal::Literal;
use crate::r#type::Type;

mod jimple;
mod smali;

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub visibility: Vec<AccessFlag>,
    pub initial_value: Option<Literal>,
    pub annotations: Vec<Annotation>,
}
