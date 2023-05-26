use crate::access_flag::AccessFlag;
use crate::annotation::Annotation;
use crate::field::Field;
use crate::method::Method;
use crate::r#type::Type;

mod jimple;
mod smali;

#[derive(Debug)]
pub struct Class {
    pub class_type: Type,
    pub access_flags: Vec<AccessFlag>,
    pub super_class: Option<Type>,
    pub interfaces: Vec<Type>,
    pub source_file: Option<String>,
    pub annotations: Vec<Annotation>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

impl Class {
    pub fn optimize(&mut self) {
        for method in &mut self.methods {
            method.optimize();
        }
    }
}
