use super::{smali::read_label, CommandData, CommandParameters, Registers};
use crate::error::ParseError;
use crate::literal::Literal;
use crate::r#type::{CallSignature, FieldSignature, MethodSignature, Type};
use crate::tokenizer::Tokenizer;

impl CommandParameters {
    pub fn read(
        input: &Tokenizer,
        command: &str,
        start: &Tokenizer,
    ) -> Result<(Tokenizer, Self), ParseError> {
        Ok(match command {
            "nop" | "return-void" => (input.clone(), Self::None),
            "move" | "move/from16" | "move/16" | "move-wide" | "move-wide/from16"
            | "move-wide/16" | "move-object" | "move-object/from16" | "move-object/16"
            | "array-length" | "neg-int" | "not-int" | "neg-long" | "not-long" | "neg-float"
            | "neg-double" | "int-to-long" | "int-to-float" | "int-to-double" | "long-to-int"
            | "long-to-float" | "long-to-double" | "float-to-int" | "float-to-long"
            | "float-to-double" | "double-to-int" | "double-to-long" | "double-to-float"
            | "int-to-byte" | "int-to-char" | "int-to-short" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register) = input.read_keyword()?;
                (input, Self::ResultRegister(result, register))
            }
            "move-result" | "move-result-wide" | "move-result-object" | "move-exception" => {
                let (input, result) = input.read_keyword()?;
                (input, Self::Result(result))
            }
            "return" | "return-wide" | "return-object" | "monitor-enter" | "monitor-exit"
            | "throw" => {
                let (input, register) = input.read_keyword()?;
                (input, Self::Register(register))
            }
            "const/4" | "const/16" | "const" | "const/high16" | "const-wide/16"
            | "const-wide/32" | "const-wide" | "const-wide/high16" | "const-string"
            | "const-string/jumbo" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, literal) = Literal::read(&input)?;
                (input, Self::ResultLiteral(result, literal))
            }
            "const-class" | "new-instance" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, r#type) = Type::read(&input)?;
                (input, Self::ResultType(result, r#type))
            }
            "check-cast" => {
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, r#type) = Type::read(&input)?;
                (input, Self::RegisterType(register, r#type))
            }
            "instance-of" | "new-array" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, r#type) = Type::read(&input)?;
                (input, Self::ResultRegisterType(result, register, r#type))
            }
            "filled-new-array" | "filled-new-array/range" => {
                let (input, registers) = Registers::read(input)?;
                let input = input.expect_char(',')?;
                let (input, r#type) = Type::read(&input)?;
                (input, Self::ResultRegistersType(None, registers, r#type))
            }
            "fill-array-data" | "packed-switch" | "sparse-switch" => {
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, label) = read_label(&input)?;
                (
                    input,
                    Self::RegisterData(register, CommandData::Label(label)),
                )
            }
            "if-eqz" | "if-nez" | "if-ltz" | "if-gez" | "if-gtz" | "if-lez" => {
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, label) = read_label(&input)?;
                (input, Self::RegisterLabel(register, label))
            }
            "goto" | "goto/16" | "goto/32" => {
                let (input, label) = read_label(input)?;
                (input, Self::Label(label))
            }
            "cmpl-float" | "cmpg-float" | "cmpl-double" | "cmpg-double" | "cmp-long" | "aget"
            | "aget-wide" | "aget-object" | "aget-boolean" | "aget-byte" | "aget-char"
            | "aget-short" | "add-int" | "sub-int" | "mul-int" | "div-int" | "rem-int"
            | "and-int" | "or-int" | "xor-int" | "shl-int" | "shr-int" | "ushr-int"
            | "add-long" | "sub-long" | "mul-long" | "div-long" | "rem-long" | "and-long"
            | "or-long" | "xor-long" | "shl-long" | "shr-long" | "ushr-long" | "add-float"
            | "sub-float" | "mul-float" | "div-float" | "rem-float" | "add-double"
            | "sub-double" | "mul-double" | "div-double" | "rem-double" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register1) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register2) = input.read_keyword()?;
                (
                    input,
                    Self::ResultRegisterRegister(result, register1, register2),
                )
            }
            "if-eq" | "if-ne" | "if-lt" | "if-ge" | "if-gt" | "if-le" => {
                let (input, register1) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register2) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, label) = read_label(&input)?;
                (
                    input,
                    Self::RegisterRegisterLabel(register1, register2, label),
                )
            }
            "aput" | "aput-wide" | "aput-object" | "aput-boolean" | "aput-byte" | "aput-char"
            | "aput-short" => {
                let (input, register1) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register2) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register3) = input.read_keyword()?;
                (
                    input,
                    Self::RegisterRegisterRegister(register1, register2, register3),
                )
            }
            "iget" | "iget-wide" | "iget-object" | "iget-boolean" | "iget-byte" | "iget-char"
            | "iget-short" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, field) = FieldSignature::read(&input)?;
                (input, Self::ResultRegisterField(result, register, field))
            }
            "iput" | "iput-wide" | "iput-object" | "iput-boolean" | "iput-byte" | "iput-char"
            | "iput-short" => {
                let (input, register1) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register2) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, field) = FieldSignature::read(&input)?;
                (
                    input,
                    Self::RegisterRegisterField(register1, register2, field),
                )
            }
            "sget" | "sget-wide" | "sget-object" | "sget-boolean" | "sget-byte" | "sget-char"
            | "sget-short" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, field) = FieldSignature::read(&input)?;
                (input, Self::ResultField(result, field))
            }
            "sput" | "sput-wide" | "sput-object" | "sput-boolean" | "sput-byte" | "sput-char"
            | "sput-short" => {
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, field) = FieldSignature::read(&input)?;
                (input, Self::RegisterField(register, field))
            }
            "invoke-virtual"
            | "invoke-super"
            | "invoke-direct"
            | "invoke-static"
            | "invoke-interface"
            | "invoke-virtual/range"
            | "invoke-super/range"
            | "invoke-direct/range"
            | "invoke-static/range"
            | "invoke-interface/range" => {
                let (input, registers) = Registers::read(input)?;
                let input = input.expect_char(',')?;
                let (input, method) = MethodSignature::read(&input)?;
                (input, Self::ResultRegistersMethod(None, registers, method))
            }
            "add-int/2addr" | "sub-int/2addr" | "mul-int/2addr" | "div-int/2addr"
            | "rem-int/2addr" | "and-int/2addr" | "or-int/2addr" | "xor-int/2addr"
            | "shl-int/2addr" | "shr-int/2addr" | "ushr-int/2addr" | "add-long/2addr"
            | "sub-long/2addr" | "mul-long/2addr" | "div-long/2addr" | "rem-long/2addr"
            | "and-long/2addr" | "or-long/2addr" | "xor-long/2addr" | "shl-long/2addr"
            | "shr-long/2addr" | "ushr-long/2addr" | "add-float/2addr" | "sub-float/2addr"
            | "mul-float/2addr" | "div-float/2addr" | "rem-float/2addr" | "add-double/2addr"
            | "sub-double/2addr" | "mul-double/2addr" | "div-double/2addr" | "rem-double/2addr" => {
                let (input, register1) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register2) = input.read_keyword()?;
                (input, Self::RegisterRegister(register1, register2))
            }
            "add-int/lit16" | "rsub-int" | "mul-int/lit16" | "div-int/lit16" | "rem-int/lit16"
            | "and-int/lit16" | "or-int/lit16" | "xor-int/lit16" | "add-int/lit8"
            | "rsub-int/lit8" | "mul-int/lit8" | "div-int/lit8" | "rem-int/lit8"
            | "and-int/lit8" | "or-int/lit8" | "xor-int/lit8" | "shl-int/lit8" | "shr-int/lit8"
            | "ushr-int/lit8" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, literal) = Literal::read(&input)?;
                (
                    input,
                    Self::ResultRegisterLiteral(result, register, literal),
                )
            }
            "invoke-polymorphic" | "invoke-polymorphic/range" => {
                let (input, registers) = Registers::read(input)?;
                let input = input.expect_char(',')?;
                let (input, method) = MethodSignature::read(&input)?;
                let input = input.expect_char(',')?;
                let (input, call) = CallSignature::read(&input)?;
                (
                    input,
                    Self::ResultRegistersMethodCall(None, registers, method, call),
                )
            }
            // TODO: invoke-custom and invoke-custom/range
            "const-method-handle" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, invoke_type) = input.read_keyword()?;
                let input = input.expect_char('@')?;
                let (input, method) = MethodSignature::read(&input)?;
                (input, Self::ResultMethodHandle(result, invoke_type, method))
            }
            "const-method-type" => {
                let (input, result) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, call) = CallSignature::read(&input)?;
                (input, Self::ResultCall(result, call))
            }
            _ => {
                return Err(start.unexpected("a valid command".into()));
            }
        })
    }
}
