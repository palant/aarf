use std::borrow::Cow;
use std::fmt::{Display, Formatter};

use crate::error::ParseError;
use crate::tokenizer::Tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bool,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Void,
    Object(String),
    Array(Box<Type>),
}

impl Type {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let start = input;
        let (input, c) = input
            .read_char()
            .map_err(|_| input.unexpected("a type".into()))?;
        Ok(match c {
            'Z' => (input, Type::Bool),
            'B' => (input, Type::Byte),
            'C' => (input, Type::Char),
            'S' => (input, Type::Short),
            'I' => (input, Type::Int),
            'J' => (input, Type::Long),
            'F' => (input, Type::Float),
            'D' => (input, Type::Double),
            'V' => (input, Type::Void),
            'L' => {
                let (input, name) = input.read_to(&[';']);
                let input = input.expect_char(';')?;
                if name.is_empty() {
                    return Err(start.unexpected("a type".into()));
                }
                (input, Type::Object(name.replace('/', ".")))
            }
            '[' => {
                let (input, subtype) = Type::read(&input)?;
                (input, Type::Array(Box::new(subtype)))
            }
            _ => return Err(start.unexpected("a type".into())),
        })
    }

    pub fn get_name(&self) -> Cow<'_, str> {
        match self {
            Self::Bool => "bool".into(),
            Self::Byte => "byte".into(),
            Self::Char => "char".into(),
            Self::Short => "short".into(),
            Self::Int => "int".into(),
            Self::Long => "long".into(),
            Self::Float => "float".into(),
            Self::Double => "double".into(),
            Self::Void => "void".into(),
            Self::Object(name) => name.into(),
            Self::Array(subtype) => subtype.get_name() + "[]",
        }
    }

    pub fn register_count(&self) -> usize {
        match self {
            Self::Long | Self::Double => 2,
            _ => 1,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.get_name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldSignature {
    pub object_type: Type,
    pub field_name: String,
    pub field_type: Type,
}

impl FieldSignature {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, object_type) = Type::read(input)?;
        let input = input.expect_char('-')?;
        let input = input.expect_char('>')?;
        let (input, field_name) = input.read_keyword()?;
        let input = input.expect_char(':')?;
        let (input, field_type) = Type::read(&input)?;
        Ok((
            input,
            Self {
                object_type,
                field_name,
                field_type,
            },
        ))
    }
}

impl Display for FieldSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{} {}.{}",
            self.field_type, self.object_type, self.field_name
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallSignature {
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
}

impl CallSignature {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let mut input = input.expect_char('(')?;

        let mut parameter_types = Vec::new();
        while input.expect_char(')').is_err() {
            let (i, parameter_type) = Type::read(&input)?;
            input = i;
            parameter_types.push(parameter_type);
        }
        let input = input.expect_char(')')?;

        let (input, return_type) = Type::read(&input)?;
        Ok((
            input,
            Self {
                parameter_types,
                return_type,
            },
        ))
    }
}

impl Display for CallSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let params = self
            .parameter_types
            .iter()
            .map(Type::get_name)
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{} ({params})", self.return_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub object_type: Type,
    pub method_name: String,
    pub call_signature: CallSignature,
}

impl MethodSignature {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, object_type) = Type::read(input)?;
        let input = input.expect_char('-')?;
        let input = input.expect_char('>')?;
        let (input, method_name) = input.read_keyword()?;
        let (input, call_signature) = CallSignature::read(&input)?;
        Ok((
            input,
            Self {
                object_type,
                method_name,
                call_signature,
            },
        ))
    }
}

impl Display for MethodSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let params = self
            .call_signature
            .parameter_types
            .iter()
            .map(Type::get_name)
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "{} {}.{}({params})",
            self.call_signature.return_type, self.object_type, self.method_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorDisplayed;

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn read_type() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" Ljava/lang/Object;[IVW");

        let (input, r#type) = Type::read(&input)?;
        assert_eq!(r#type, Type::Object("java.lang.Object".to_string()));

        let (input, r#type) = Type::read(&input)?;
        assert_eq!(r#type, Type::Array(Box::new(Type::Int)));

        let (input, r#type) = Type::read(&input)?;
        assert_eq!(r#type, Type::Void);

        assert!(Type::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_field_signature() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" Lev/n;->g:Ljava/lang/String;");

        let (_, signature) = FieldSignature::read(&input)?;
        assert_eq!(
            signature,
            FieldSignature {
                object_type: Type::Object("ev.n".to_string()),
                field_name: "g".to_string(),
                field_type: Type::Object("java.lang.String".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn read_method_signature() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" Lev/n;->g(Ljava/lang/Object;Ljava/lang/String;)V");

        let (_, signature) = MethodSignature::read(&input)?;
        assert_eq!(
            signature,
            MethodSignature {
                object_type: Type::Object("ev.n".to_string()),
                method_name: "g".to_string(),
                call_signature: CallSignature {
                    parameter_types: vec![
                        Type::Object("java.lang.Object".to_string()),
                        Type::Object("java.lang.String".to_string()),
                    ],
                    return_type: Type::Void,
                },
            }
        );

        Ok(())
    }
}
