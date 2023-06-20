use super::{smali::read_label, CommandData, CommandParameter, ParameterKind, Register, Registers};
use crate::error::ParseError;
use crate::literal::Literal;
use crate::r#type::{FieldSignature, MethodSignature, Type};
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
            ParameterKind::Literal => {
                let (input, literal) = Literal::read(input)?;
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
            ParameterKind::Data => {
                let (input, label) = read_label(input)?;
                (input, Self::Data(CommandData::Label(label)))
            }
        })
    }
}
