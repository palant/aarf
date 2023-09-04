use super::{Annotation, AnnotationParameter, AnnotationParameterValue, AnnotationVisibility};
use crate::error::ParseError;
use crate::literal::Literal;
use crate::r#type::Type;
use crate::tokenizer::Tokenizer;

impl AnnotationParameterValue {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        if input.expect_directive("enum").is_ok() {
            let input = input.expect_directive("enum")?;
            let (input, enum_type) = Type::read(&input)?;
            let input = input.expect_char('-')?;
            let input = input.expect_char('>')?;
            let (input, value) = input.read_keyword()?;
            let input = input.expect_char(':')?;

            let (input2, enum_type2) = Type::read(&input)?;
            if enum_type != enum_type2 {
                return Err(input.unexpected(enum_type.get_name().to_string().into()));
            }
            let input = input2;

            Ok((input, Self::Enum(enum_type, value)))
        } else if input.expect_directive("subannotation").is_ok() {
            let input = input.expect_directive("subannotation")?;
            let (input, annotation) = Annotation::read(&input, true)?;
            Ok((input, Self::SubAnnotation(annotation)))
        } else if input.expect_char('{').is_ok() {
            let mut input = input.expect_char('{')?;
            let mut entries = Vec::new();
            if input.expect_char('}').is_err() {
                input = input.expect_eol()?;

                while input.expect_char('}').is_err() {
                    let (i, entry) = Self::read(&input)?;
                    input = i;
                    if let Ok(i) = input.expect_char(',') {
                        input = i;
                    }
                    entries.push(entry);

                    input = input.expect_eol()?;
                }
            }
            let input = input.expect_char('}')?;
            Ok((input, Self::Array(entries)))
        } else {
            let (input, value) = Literal::read(input)?;
            Ok((input, Self::Literal(value)))
        }
    }
}

impl AnnotationParameter {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, name) = input.read_keyword()?;
        let input = input.expect_char('=')?;
        let (input, value) = AnnotationParameterValue::read(&input)?;
        let input = input.expect_eol()?;
        Ok((input, Self { name, value }))
    }
}

impl Annotation {
    pub fn read(input: &Tokenizer, subannotation: bool) -> Result<(Tokenizer, Self), ParseError> {
        let (input, visibility) = if subannotation {
            (input.clone(), AnnotationVisibility::Build)
        } else {
            AnnotationVisibility::read(input)?
        };
        let (input, annotation_type) = Type::read(&input)?;
        let mut input = input.expect_eol()?;

        let mut parameters = Vec::new();
        while input.expect_directive("end").is_err() {
            let (i, parameter) = AnnotationParameter::read(&input)?;
            input = i;
            parameters.push(parameter);
        }

        let mut input = input.expect_directive("end")?;
        if subannotation {
            input = input.expect_keyword("subannotation")?;
        } else {
            input = input.expect_keyword("annotation")?;
            input = input.expect_eol()?;
        }

        Ok((
            input,
            Self {
                annotation_type,
                visibility,
                parameters,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorDisplayed;
    use crate::r#type::{CallSignature, MethodSignature};

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn read_annotation() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#"
            .annotation system Ldalvik/annotation/AnnotationDefault;
                value = .subannotation LAnnotationWithValues;
                            booleanValue = false
                            byteValue = 1t
                            charValue = '2'
                            shortValue = 3s
                            intValue = 4
                            longValue = 5l
                            floatValue = 6.0f
                            doubleValue = 7.0
                            stringValue = "8"
                            subAnnotationValue = .subannotation LSubAnnotation;
                                                        stringValue = "9"
                                                .end subannotation
                            typeValue = L10;
                            methodValue = L10;->11()V
                            methodValue2 = Lj2/b;->connect(Ljava/lang/String;II)V
                            methodHandle = invoke-static@Lj2/b;-><init>(Ljava/lang/String;II)V
                            methodType = (Ljava/lang/String;II)V
                            enumValue = .enum LEnum;->12:LEnum;
                        .end subannotation
            .end annotation

            .annotation runtime Ldalvik/annotation/MemberClasses;
                value = {
                    Lj2/b$a;
                }
            .end annotation

            .annotation build Ldalvik/annotation/Signature;
                value = {
                    "Ljava/lang/Enum<",
                    "Lj2/b;",
                    ">;"
                }
            .end annotation

            .annotation runtime Ljava/lang/annotation/Target;
                value = {
                    .enum Ljava/lang/annotation/ElementType;->PACKAGE:Ljava/lang/annotation/ElementType;,
                    .enum Ljava/lang/annotation/ElementType;->TYPE:Ljava/lang/annotation/ElementType;,
                }
            .end annotation
        "#.trim());

        let input = input.expect_directive("annotation")?;
        let (input, annotation) = Annotation::read(&input, false)?;
        assert_eq!(
            annotation,
            Annotation {
                annotation_type: Type::Object("dalvik.annotation.AnnotationDefault".to_string()),
                visibility: AnnotationVisibility::System,
                parameters: vec![AnnotationParameter {
                    name: "value".to_string(),
                    value: AnnotationParameterValue::SubAnnotation(Annotation {
                        annotation_type: Type::Object("AnnotationWithValues".to_string()),
                        visibility: AnnotationVisibility::Build,
                        parameters: vec![
                            AnnotationParameter {
                                name: "booleanValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Bool(false)),
                            },
                            AnnotationParameter {
                                name: "byteValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Byte(1)),
                            },
                            AnnotationParameter {
                                name: "charValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Char('2' as u16)),
                            },
                            AnnotationParameter {
                                name: "shortValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Short(3)),
                            },
                            AnnotationParameter {
                                name: "intValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Int(4)),
                            },
                            AnnotationParameter {
                                name: "longValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Long(5)),
                            },
                            AnnotationParameter {
                                name: "floatValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Float(6.0)),
                            },
                            AnnotationParameter {
                                name: "doubleValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Double(7.0)),
                            },
                            AnnotationParameter {
                                name: "stringValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::String(
                                    "8".to_string()
                                )),
                            },
                            AnnotationParameter {
                                name: "subAnnotationValue".to_string(),
                                value: AnnotationParameterValue::SubAnnotation(Annotation {
                                    annotation_type: Type::Object("SubAnnotation".to_string()),
                                    visibility: AnnotationVisibility::Build,
                                    parameters: vec![AnnotationParameter {
                                        name: "stringValue".to_string(),
                                        value: AnnotationParameterValue::Literal(Literal::String(
                                            "9".to_string()
                                        )),
                                    },]
                                }),
                            },
                            AnnotationParameter {
                                name: "typeValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Class(
                                    Type::Object("10".to_string())
                                )),
                            },
                            AnnotationParameter {
                                name: "methodValue".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Method(
                                    MethodSignature {
                                        object_type: Type::Object("10".to_string()),
                                        method_name: "11".to_string(),
                                        call_signature: CallSignature {
                                            parameter_types: Vec::new(),
                                            return_type: Type::Void,
                                        },
                                    }
                                )),
                            },
                            AnnotationParameter {
                                name: "methodValue2".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::Method(
                                    MethodSignature {
                                        object_type: Type::Object("j2.b".to_string()),
                                        method_name: "connect".to_string(),
                                        call_signature: CallSignature {
                                            parameter_types: vec![
                                                Type::Object("java.lang.String".to_string()),
                                                Type::Int,
                                                Type::Int,
                                            ],
                                            return_type: Type::Void,
                                        },
                                    }
                                )),
                            },
                            AnnotationParameter {
                                name: "methodHandle".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::MethodHandle(
                                    "invoke-static".to_string(),
                                    MethodSignature {
                                        object_type: Type::Object("j2.b".to_string()),
                                        method_name: "<init>".to_string(),
                                        call_signature: CallSignature {
                                            parameter_types: vec![
                                                Type::Object("java.lang.String".to_string()),
                                                Type::Int,
                                                Type::Int,
                                            ],
                                            return_type: Type::Void,
                                        },
                                    }
                                )),
                            },
                            AnnotationParameter {
                                name: "methodType".to_string(),
                                value: AnnotationParameterValue::Literal(Literal::MethodType(
                                    CallSignature {
                                        parameter_types: vec![
                                            Type::Object("java.lang.String".to_string()),
                                            Type::Int,
                                            Type::Int,
                                        ],
                                        return_type: Type::Void,
                                    }
                                )),
                            },
                            AnnotationParameter {
                                name: "enumValue".to_string(),
                                value: AnnotationParameterValue::Enum(
                                    Type::Object("Enum".to_string()),
                                    "12".to_string(),
                                ),
                            },
                        ],
                    }),
                },],
            }
        );
        assert!(input.expect_eof().is_err());

        let input = input.expect_directive("annotation")?;
        let (input, annotation) = Annotation::read(&input, false)?;
        assert_eq!(
            annotation,
            Annotation {
                annotation_type: Type::Object("dalvik.annotation.MemberClasses".to_string()),
                visibility: AnnotationVisibility::Runtime,
                parameters: vec![AnnotationParameter {
                    name: "value".to_string(),
                    value: AnnotationParameterValue::Array(vec![
                        AnnotationParameterValue::Literal(Literal::Class(Type::Object(
                            "j2.b$a".to_string()
                        ))),
                    ]),
                }],
            }
        );
        assert!(input.expect_eof().is_err());

        let input = input.expect_directive("annotation")?;
        let (input, annotation) = Annotation::read(&input, false)?;
        assert_eq!(
            annotation,
            Annotation {
                annotation_type: Type::Object("dalvik.annotation.Signature".to_string()),
                visibility: AnnotationVisibility::Build,
                parameters: vec![AnnotationParameter {
                    name: "value".to_string(),
                    value: AnnotationParameterValue::Array(vec![
                        AnnotationParameterValue::Literal(Literal::String(
                            "Ljava/lang/Enum<".to_string()
                        )),
                        AnnotationParameterValue::Literal(Literal::String("Lj2/b;".to_string())),
                        AnnotationParameterValue::Literal(Literal::String(">;".to_string())),
                    ]),
                }],
            }
        );
        assert!(input.expect_eof().is_err());

        let input = input.expect_directive("annotation")?;
        let (input, annotation) = Annotation::read(&input, false)?;
        assert_eq!(
            annotation,
            Annotation {
                annotation_type: Type::Object("java.lang.annotation.Target".to_string()),
                visibility: AnnotationVisibility::Runtime,
                parameters: vec![AnnotationParameter {
                    name: "value".to_string(),
                    value: AnnotationParameterValue::Array(vec![
                        AnnotationParameterValue::Enum(
                            Type::Object("java.lang.annotation.ElementType".to_string()),
                            "PACKAGE".to_string()
                        ),
                        AnnotationParameterValue::Enum(
                            Type::Object("java.lang.annotation.ElementType".to_string()),
                            "TYPE".to_string()
                        ),
                    ]),
                }],
            }
        );
        assert!(input.expect_eof().is_ok());

        Ok(())
    }
}
