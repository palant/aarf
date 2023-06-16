use super::{CommandData, CommandParameter, Instruction, ParameterKind, DEFS};
use crate::error::ParseError;
use crate::literal::Literal;
use crate::r#type::Type;
use crate::tokenizer::Tokenizer;

pub(crate) fn read_label(input: &Tokenizer) -> Result<(Tokenizer, String), ParseError> {
    let input = input.expect_char(':')?;
    let (input, label) = input.read_keyword()?;
    Ok((input, label))
}

impl Instruction {
    fn read_directive(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let start = input;
        let (input, directive) = input.read_directive()?;
        Ok(match directive.as_str() {
            "line" => {
                let start = &input;
                let (input, literal) = Literal::read(&input)?;
                let number = literal
                    .get_integer()
                    .ok_or_else(|| start.unexpected("a number".into()))?;
                (input, Self::LineNumber(number, number))
            }
            "catchall" | "catch" => {
                let (input, exception) = if directive == "catch" {
                    let (input, exception) = Type::read(&input)?;
                    (input, Some(exception))
                } else {
                    (input, None)
                };

                let input = input.expect_char('{')?;
                let (input, start_label) = read_label(&input)?;
                let input = input.expect_char('.')?;
                let input = input.expect_char('.')?;
                let (input, end_label) = read_label(&input)?;
                let input = input.expect_char('}')?;
                let (input, target) = read_label(&input)?;
                (
                    input,
                    Self::Catch {
                        exception,
                        start_label,
                        end_label,
                        target,
                    },
                )
            }
            "packed-switch" => {
                let start = &input;
                let (input, first_key) = Literal::read(&input)?;
                let first_key = first_key
                    .get_integer()
                    .ok_or_else(|| start.unexpected("a number".into()))?;
                let mut input = input.expect_eol()?;

                let mut targets = Vec::new();
                while input.expect_directive("end").is_err() {
                    let target;
                    (input, target) = read_label(&input)?;
                    input = input.expect_eol()?;
                    targets.push(target);
                }

                let input = input.expect_directive("end")?;
                let input = input.expect_keyword("packed-switch")?;
                (
                    input,
                    Self::Data(CommandData::PackedSwitch(first_key, targets)),
                )
            }
            "sparse-switch" => {
                let mut input = input.expect_eol()?;

                let mut targets = Vec::new();
                while input.expect_directive("end").is_err() {
                    let value;
                    (input, value) = Literal::read(&input)?;
                    input = input.expect_char('-')?;
                    input = input.expect_char('>')?;

                    let target;
                    (input, target) = read_label(&input)?;
                    input = input.expect_eol()?;
                    targets.push((value, target));
                }

                let input = input.expect_directive("end")?;
                let input = input.expect_keyword("sparse-switch")?;
                (input, Self::Data(CommandData::SparseSwitch(targets)))
            }
            "array-data" => {
                let start = &input;
                let (input, literal) = Literal::read(&input)?;
                let _element_size = literal
                    .get_integer()
                    .ok_or_else(|| start.unexpected("a number".into()))?;
                let mut input = input.expect_eol()?;

                let mut elements = Vec::new();
                while input.expect_directive("end").is_err() {
                    let element;
                    (input, element) = Literal::read(&input)?;
                    input = input.expect_eol()?;
                    elements.push(element);
                }

                let input = input.expect_directive("end")?;
                let input = input.expect_keyword("array-data")?;
                (input, Self::Data(CommandData::Array(elements)))
            }
            "local" => {
                let (input, register) = input.read_keyword()?;
                let input = input.expect_char(',')?;
                let (input, name) = Literal::read(&input)?;
                let input = input.expect_char(':')?;
                let (input, local_type) = Type::read(&input)?;

                // There might be an additional signature here, ignore this part
                let (input, _) = input.read_to(&['\n']);

                (
                    input,
                    Self::Local {
                        register,
                        name,
                        local_type,
                    },
                )
            }
            "restart" => {
                let input = input.expect_keyword("local")?;
                let (input, register) = input.read_keyword()?;

                (input, Self::LocalRestart { register })
            }
            _ => return Err(start.unexpected("a supported directive".into())),
        })
    }

    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, result) = if input.expect_char('.').is_ok() {
            Self::read_directive(input)?
        } else if let Ok((input, label)) = read_label(input) {
            (input, Self::Label(label))
        } else {
            let start = input;
            let (mut input, command) = input.read_keyword()?;
            let command = command.to_ascii_lowercase();
            let mut parameters = Vec::new();

            if let Some(defs) = DEFS.get(&command) {
                let mut first = true;
                for kind in defs.parameters {
                    if !first {
                        input = input.expect_char(',')?;
                    } else if *kind != ParameterKind::DefaultEmptyResult {
                        first = false;
                    }

                    let parameter;
                    (input, parameter) = CommandParameter::read(&input, kind)?;
                    parameters.push(parameter);
                }
            } else {
                return Err(start.unexpected("a supported command".into()));
            }

            (
                input,
                Self::Command {
                    command,
                    parameters,
                },
            )
        };

        let input = input.expect_eol()?;
        Ok((input, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorDisplayed;
    use crate::instruction::{Register, Registers};
    use crate::r#type::{CallSignature, MethodSignature};

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn read_instruction() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(
            r#"
                .line 12
                nop
                :label
                invoke-polymorphic {p1, v0, v1}, Ljava/lang/invoke/MethodHandle;->invoke([Ljava/lang/Object;)Ljava/lang/Object;, (II)V
                invoke-polymorphic/range {v0 .. v2}, Ljava/lang/invoke/MethodHandle;->invoke([Ljava/lang/Object;)Ljava/lang/Object;, (II)V
                const-method-handle v0, invoke-static@Ljava/lang/Integer;->toString(I)Ljava/lang/String;
                const-method-type v0, (II)I
                .catch Ljava/lang/NullPointerException; {:try_start_0 .. :try_end_0} :catch_0
                .catchall {:try_start_1 .. :try_end_1} :catch_1
            "#.trim()
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(instruction, Instruction::LineNumber(12, 12),);

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Command {
                command: "nop".to_string(),
                parameters: Vec::new(),
            },
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(instruction, Instruction::Label("label".to_string()),);

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Command {
                command: "invoke-polymorphic".to_string(),
                parameters: vec![
                    CommandParameter::DefaultEmptyResult(None),
                    CommandParameter::Registers(Registers::List(vec![
                        Register::Parameter(1),
                        Register::Local(0),
                        Register::Local(1),
                    ])),
                    CommandParameter::Method(MethodSignature {
                        object_type: Type::Object("java.lang.invoke.MethodHandle".to_string()),
                        method_name: "invoke".to_string(),
                        call_signature: CallSignature {
                            parameter_types: vec![Type::Array(Box::new(Type::Object(
                                "java.lang.Object".to_string()
                            )))],
                            return_type: Type::Object("java.lang.Object".to_string())
                        },
                    }),
                    CommandParameter::Call(CallSignature {
                        parameter_types: vec![Type::Int, Type::Int],
                        return_type: Type::Void,
                    }),
                ],
            }
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Command {
                command: "invoke-polymorphic/range".to_string(),
                parameters: vec![
                    CommandParameter::DefaultEmptyResult(None),
                    CommandParameter::Registers(Registers::Range(
                        Register::Local(0),
                        Register::Local(2),
                    )),
                    CommandParameter::Method(MethodSignature {
                        object_type: Type::Object("java.lang.invoke.MethodHandle".to_string()),
                        method_name: "invoke".to_string(),
                        call_signature: CallSignature {
                            parameter_types: vec![Type::Array(Box::new(Type::Object(
                                "java.lang.Object".to_string()
                            )))],
                            return_type: Type::Object("java.lang.Object".to_string())
                        },
                    }),
                    CommandParameter::Call(CallSignature {
                        parameter_types: vec![Type::Int, Type::Int],
                        return_type: Type::Void,
                    })
                ],
            }
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Command {
                command: "const-method-handle".to_string(),
                parameters: vec![
                    CommandParameter::Result(Register::Local(0)),
                    CommandParameter::MethodHandle(
                        "invoke-static".to_string(),
                        MethodSignature {
                            object_type: Type::Object("java.lang.Integer".to_string()),
                            method_name: "toString".to_string(),
                            call_signature: CallSignature {
                                parameter_types: vec![Type::Int],
                                return_type: Type::Object("java.lang.String".to_string())
                            },
                        },
                    ),
                ],
            }
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Command {
                command: "const-method-type".to_string(),
                parameters: vec![
                    CommandParameter::Result(Register::Local(0)),
                    CommandParameter::Call(CallSignature {
                        parameter_types: vec![Type::Int, Type::Int],
                        return_type: Type::Int
                    }),
                ],
            }
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Catch {
                exception: Some(Type::Object("java.lang.NullPointerException".to_string())),
                start_label: "try_start_0".to_string(),
                end_label: "try_end_0".to_string(),
                target: "catch_0".to_string(),
            },
        );

        let (input, instruction) = Instruction::read(&input)?;
        assert_eq!(
            instruction,
            Instruction::Catch {
                exception: None,
                start_label: "try_start_1".to_string(),
                end_label: "try_end_1".to_string(),
                target: "catch_1".to_string(),
            },
        );

        assert!(input.expect_eof().is_ok());
        Ok(())
    }
}
