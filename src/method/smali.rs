use super::{Method, MethodParameter};
use crate::access_flag::AccessFlag;
use crate::annotation::Annotation;
use crate::error::ParseError;
use crate::instruction::Instruction;
use crate::r#type::Type;
use crate::tokenizer::Tokenizer;

impl Method {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, visibility) = AccessFlag::read_list(input);
        let (input, name) = input.read_keyword()?;

        let mut input = input.expect_char('(')?;
        let mut parameters = Vec::new();
        while input.expect_char(')').is_err() {
            let parameter_type;
            (input, parameter_type) = Type::read(&input)?;
            parameters.push(MethodParameter {
                parameter_type,
                annotations: Vec::new(),
            });
        }

        let input = input.expect_char(')')?;
        let (input, return_type) = Type::read(&input)?;
        let mut input = input.expect_eol()?;

        let mut annotations = Vec::new();
        let mut instructions = Vec::new();
        while input.expect_directive("end").is_err() {
            if let Ok(i) = input.expect_directive("annotation") {
                input = i;

                let annotation;
                (input, annotation) = Annotation::read(&input, false)?;
                annotations.push(annotation);
            } else if let Ok(i) = input.expect_directive("locals") {
                input = i;

                (input, _) = input.read_number()?;
                input = input.expect_eol()?;
            } else if let Ok(i) = input.expect_directive("param") {
                input = i;

                let start = input.clone();
                input = input.expect_char('p')?;

                let mut index;
                (input, index) = input.read_number()?;
                if !visibility.contains(&AccessFlag::Static) {
                    index -= return_type.register_count() as i64;
                }

                let mut param_index = 0;
                while param_index < parameters.len() && index > 0 {
                    index -= parameters[param_index].parameter_type.register_count() as i64;
                    param_index += 1;
                }

                if index < 0 || param_index >= parameters.len() {
                    return Err(start.unexpected("a valid parameter number".into()));
                }

                (input, _) = input.read_to(&['\n']);
                input = input.expect_eol()?;
                while input.expect_directive("end").is_err() {
                    input = input.expect_directive("annotation")?;

                    let annotation;
                    (input, annotation) = Annotation::read(&input, false)?;
                    parameters[param_index].annotations.push(annotation);
                }

                input = input.expect_directive("end")?;
                input = input.expect_keyword("param")?;
                input = input.expect_eol()?;
            } else {
                let instruction;
                (input, instruction) = Instruction::read(&input)?;
                instructions.push(instruction);
            }

            while let Ok(i) = input.expect_directive("end") {
                if let Ok(i) = i.expect_keyword("local") {
                    // Ignore .end local line, it has no meaning for us
                    (input, _) = i.read_to(&['\n']);
                    input = input.expect_eol()?;
                } else {
                    break;
                }
            }
        }

        let input = input.expect_directive("end")?;
        let input = input.expect_keyword("method")?;
        let input = input.expect_eol()?;

        Ok((
            input,
            Self {
                name,
                visibility,
                parameters,
                return_type,
                annotations,
                instructions,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{AnnotationParameter, AnnotationParameterValue, AnnotationVisibility};
    use crate::error::ParseErrorDisplayed;
    use crate::instruction::{CommandParameter, Register, Registers};
    use crate::literal::Literal;
    use crate::r#type::{CallSignature, MethodSignature, Type};

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn read_method() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(
            r#"
                .method public synthetic constructor <init>(Ldv/a;Ldv/b;)V
                    .locals 1
                    .param p1    # Ldv/a;
                        .annotation runtime Lz20/t;
                            value = "something"
                        .end annotation
                    .end param
                    .annotation system Ldalvik/annotation/Signature;
                        value = {
                            "(",
                            "Ldv/a<",
                            "Lqu/x;",
                            ">,Ldv/b;)V"
                        }
                    .end annotation

                    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

                    return-void
                .end method
            "#
            .trim(),
        );

        let input = input.expect_directive("method")?;
        let (input, method) = Method::read(&input)?;
        assert_eq!(
            method,
            Method {
                name: "<init>".to_string(),
                visibility: vec![
                    AccessFlag::Public,
                    AccessFlag::Synthetic,
                    AccessFlag::Constructor
                ],
                parameters: vec![
                    MethodParameter {
                        parameter_type: Type::Object("dv.a".to_string()),
                        annotations: vec![Annotation {
                            annotation_type: Type::Object("z20.t".to_string()),
                            visibility: AnnotationVisibility::Runtime,
                            parameters: vec![AnnotationParameter {
                                name: "value".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::String(
                                    "something".to_string()
                                )),
                            }],
                        }],
                    },
                    MethodParameter {
                        parameter_type: Type::Object("dv.b".to_string()),
                        annotations: Vec::new(),
                    },
                ],
                return_type: Type::Void,
                annotations: vec![Annotation {
                    annotation_type: Type::Object("dalvik.annotation.Signature".to_string()),
                    visibility: AnnotationVisibility::System,
                    parameters: vec![AnnotationParameter {
                        name: "value".to_string(),
                        value: AnnotationParameterValue::Array(
                            vec!["(", "Ldv/a<", "Lqu/x;", ">,Ldv/b;)V"]
                                .iter()
                                .map(|v| AnnotationParameterValue::Literal(Literal::String(
                                    v.to_string()
                                )))
                                .collect()
                        ),
                    }],
                }],
                instructions: vec![
                    Instruction::Command {
                        command: "invoke-direct".to_string(),
                        parameters: vec![
                            CommandParameter::DefaultEmptyResult(None),
                            CommandParameter::Registers(Registers::List(vec![
                                Register::Parameter(0)
                            ])),
                            CommandParameter::Method(MethodSignature {
                                object_type: Type::Object("java.lang.Object".to_string()),
                                method_name: "<init>".to_string(),
                                call_signature: CallSignature {
                                    parameter_types: Vec::new(),
                                    return_type: Type::Void,
                                },
                            })
                        ]
                    },
                    Instruction::Command {
                        command: "return-void".to_string(),
                        parameters: Vec::new(),
                    }
                ],
            }
        );
        assert!(input.expect_eof().is_ok());

        Ok(())
    }
}
