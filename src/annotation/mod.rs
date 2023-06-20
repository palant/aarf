use crate::error::{Error, ParseError};
use crate::literal::Literal;
use crate::r#type::Type;
use crate::tokenizer::Tokenizer;

mod jimple;
mod smali;

#[derive(Debug, PartialEq)]
pub enum AnnotationVisibility {
    Build,
    Runtime,
    System,
}

impl AnnotationVisibility {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let start = input;
        let (input, keyword) = input.read_keyword()?;
        let visiblity = Self::try_from(keyword.as_str())
            .map_err(|_| start.unexpected("annotation visibility".into()))?;
        Ok((input, visiblity))
    }
}

impl TryFrom<&str> for AnnotationVisibility {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "build" => Self::Build,
            "runtime" => Self::Runtime,
            "system" => Self::System,
            other => return Err(Error::UnrecognizedToken(other.to_string())),
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum AnnotationParameterValue {
    Literal(Literal),
    Enum(Type, String),
    Array(Vec<AnnotationParameterValue>),
    SubAnnotation(Annotation),
}

#[derive(Debug, PartialEq)]
pub struct AnnotationParameter {
    pub name: String,
    pub value: AnnotationParameterValue,
}

#[derive(Debug, PartialEq)]
pub struct Annotation {
    pub annotation_type: Type,
    pub visibility: AnnotationVisibility,
    pub parameters: Vec<AnnotationParameter>,
}
