use super::Field;
use crate::access_flag::AccessFlag;
use crate::annotation::Annotation;
use crate::error::ParseError;
use crate::literal::Literal;
use crate::r#type::Type;
use crate::tokenizer::Tokenizer;

impl Field {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, visibility) = AccessFlag::read_list(input);

        let (input, name) = input.read_keyword()?;
        let input = input.expect_char(':')?;

        let (mut input, field_type) = Type::read(&input)?;

        let mut initial_value = None;
        if let Ok(i) = input.expect_char('=') {
            input = i;

            let literal;
            (input, literal) = Literal::read(&input)?;
            initial_value = Some(literal);
        }

        let mut input = input.expect_eol()?;

        let mut annotations = Vec::new();
        if input.expect_directive("annotation").is_ok() {
            while input.expect_directive("end").is_err() {
                input = input.expect_directive("annotation")?;

                let annotation;
                (input, annotation) = Annotation::read(&input, false)?;
                annotations.push(annotation);
            }
            input = input.expect_directive("end")?;
            input = input.expect_keyword("field")?;
            input = input.expect_eol()?;
        }

        Ok((
            input,
            Self {
                name,
                field_type,
                visibility,
                initial_value,
                annotations,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{AnnotationParameter, AnnotationParameterValue, AnnotationVisibility};
    use crate::error::ParseErrorDisplayed;
    use crate::literal::Literal;
    use crate::r#type::Type;

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn read_field() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(
            r#"
                .field private final description:Ljava/lang/String; = "hi"

                .field private final final:Z = false

                .field public final f:Lnu/b;
                    .annotation system Ldalvik/annotation/Signature;
                        value = {
                            "Lnu/b<",
                            "Ljava/lang/String;",
                            ">;"
                        }
                    .end annotation
                .end field
            "#
            .trim(),
        );

        let input = input.expect_directive("field")?;
        let (input, field) = Field::read(&input)?;
        assert_eq!(
            field,
            Field {
                name: "description".to_string(),
                field_type: Type::Object("java.lang.String".to_string()),
                visibility: vec![AccessFlag::Private, AccessFlag::Final],
                initial_value: Some(Literal::String("hi".to_string())),
                annotations: Vec::new(),
            }
        );
        assert!(input.expect_eof().is_err());

        let input = input.expect_directive("field")?;
        let (input, field) = Field::read(&input)?;
        assert_eq!(
            field,
            Field {
                name: "final".to_string(),
                field_type: Type::Bool,
                visibility: vec![AccessFlag::Private, AccessFlag::Final],
                initial_value: Some(Literal::Bool(false)),
                annotations: Vec::new(),
            }
        );
        assert!(input.expect_eof().is_err());

        let input = input.expect_directive("field")?;
        let (input, field) = Field::read(&input)?;
        assert_eq!(
            field,
            Field {
                name: "f".to_string(),
                field_type: Type::Object("nu.b".to_string()),
                visibility: vec![AccessFlag::Public, AccessFlag::Final],
                initial_value: None,
                annotations: vec![Annotation {
                    annotation_type: Type::Object("dalvik.annotation.Signature".to_string()),
                    visibility: AnnotationVisibility::System,
                    parameters: vec![AnnotationParameter {
                        name: "value".to_string(),
                        value: AnnotationParameterValue::Array(
                            vec!["Lnu/b<", "Ljava/lang/String;", ">;"]
                                .iter()
                                .map(|v| AnnotationParameterValue::Literal(Literal::String(
                                    v.to_string()
                                )))
                                .collect()
                        ),
                    }]
                },],
            }
        );
        assert!(input.expect_eof().is_ok());

        Ok(())
    }
}
