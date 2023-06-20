use super::{smali::read_label, CommandData, CommandParameter, ParameterKind, Register, Registers};
use crate::error::ParseError;
use crate::literal::Literal;
use crate::r#type::{CallSite, FieldSignature, MethodSignature, Type};
use crate::tokenizer::Tokenizer;

impl CommandParameter {
    pub fn read(input: &Tokenizer, kind: &ParameterKind) -> Result<(Tokenizer, Self), ParseError> {
        Ok(match kind {
            ParameterKind::Result => {
                let (input, result) = Register::read(input)?;
                (input, Self::Result(result))
            }
            ParameterKind::DefaultEmptyResult => (input.clone(), Self::DefaultEmptyResult(None)),
            ParameterKind::Register => {
                let (input, register) = Register::read(input)?;
                (input, Self::Register(register))
            }
            ParameterKind::Registers => {
                let (input, registers) = Registers::read(input)?;
                (input, Self::Registers(registers))
            }
            ParameterKind::Int
            | ParameterKind::Long
            | ParameterKind::String
            | ParameterKind::Class
            | ParameterKind::MethodHandle
            | ParameterKind::MethodType => {
                let start = input;
                let (input, mut literal) = Literal::read(input)?;
                if kind == &ParameterKind::Int {
                    let value = literal
                        .get_integer()
                        .and_then(|i| i32::try_from(i).ok())
                        .ok_or_else(|| start.unexpected("an integer literal".into()))?;
                    literal = Literal::Int(value);
                } else if kind == &ParameterKind::Long {
                    let value = literal
                        .get_integer()
                        .ok_or_else(|| start.unexpected("a long literal".into()))?;
                    literal = Literal::Long(value);
                } else if kind == &ParameterKind::Class && !literal.is_class() {
                    return Err(start.unexpected("a class".into()));
                } else if kind == &ParameterKind::MethodHandle && !literal.is_method_handle() {
                    return Err(start.unexpected("a method handle".into()));
                } else if kind == &ParameterKind::MethodType && !literal.is_method_type() {
                    return Err(start.unexpected("a method type".into()));
                }
                (input, Self::Literal(literal))
            }
            ParameterKind::Label => {
                let (input, label) = read_label(input)?;
                (input, Self::Label(label))
            }
            ParameterKind::Type => {
                let (input, r#type) = Type::read(input)?;
                (input, Self::Type(r#type))
            }
            ParameterKind::Field => {
                let (input, field) = FieldSignature::read(input)?;
                (input, Self::Field(field))
            }
            ParameterKind::Method => {
                let (input, method) = MethodSignature::read(input)?;
                (input, Self::Method(method))
            }
            ParameterKind::CallSite => {
                let (input, call_site) = CallSite::read(input)?;
                (input, Self::CallSite(call_site))
            }
            ParameterKind::Data => {
                let (input, label) = read_label(input)?;
                (input, Self::Data(CommandData::Label(label)))
            }
        })
    }
}
