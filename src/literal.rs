use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::error::ParseError;
use crate::r#type::{CallSignature, MethodSignature, Type};
use crate::tokenizer::Tokenizer;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Bool(bool),
    Char(char),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Class(Type),
    Method(MethodSignature),
    MethodHandle(String, MethodSignature),
    MethodType(CallSignature),
}

macro_rules! parse_integer {
    ($keyword:ident, $type:ty) => {
        if let Some($keyword) = $keyword.strip_prefix("-0x") {
            <$type>::from_str_radix(&("-".to_string() + $keyword), 16)
        } else if let Some($keyword) = $keyword.strip_prefix("0x") {
            <$type>::from_str_radix($keyword, 16)
        } else {
            $keyword.parse()
        }
    };
}

fn is_escaped(value: &str) -> bool {
    (value.len() - value.trim_end_matches('\\').len()) % 2 == 1
}

fn read_escaped(input: &Tokenizer, delimiter: char) -> Result<(Tokenizer, String), ParseError> {
    let (input, mut value) = input.read_to(&[delimiter]);
    let mut input = input.expect_char(delimiter)?;
    while is_escaped(&value) {
        let v;
        (input, v) = input.read_to(&[delimiter]);
        input = input.expect_char(delimiter)?;
        value = value + &delimiter.to_string() + &v;
    }
    Ok((input, value))
}

impl Literal {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        Ok(if let Ok(input) = input.expect_char('"') {
            let (input, value) = read_escaped(&input, '"')?;
            (input, Self::String(value))
        } else if let Ok(input) = input.expect_char('\'') {
            let start = &input;
            let (input, value) = read_escaped(&input, '\'')?;
            let value = value.chars().collect::<Vec<_>>();
            if value.len() == 1 {
                (input, Self::Char(value[0]))
            } else if value.len() == 2 && value[0] == '\\' {
                (input, Self::Char(value[1]))
            } else if value.len() > 2 && value[0] == '\\' && value[1] == 'u' {
                let c = u32::from_str_radix(&value[2..].iter().collect::<String>(), 16)
                    .map_err(|_| start.unexpected("a literal".into()))?;
                let c = char::from_u32(c).ok_or_else(|| start.unexpected("a literal".into()))?;
                (input, Self::Char(c))
            } else {
                return Err(start.unexpected("a literal".into()));
            }
        } else if input.expect_char('(').is_ok() {
            let (input, call) = CallSignature::read(input)?;
            (input, Self::MethodType(call))
        } else {
            let start = &input;
            let (input, keyword) = input.read_keyword()?;
            let keyword = keyword.to_ascii_lowercase();
            if keyword == "null" {
                (input, Self::Null)
            } else if keyword == "true" {
                (input, Self::Bool(true))
            } else if keyword == "false" {
                (input, Self::Bool(false))
            } else if keyword.starts_with("invoke-") {
                let input = input.expect_char('@')?;
                let (input, method) = MethodSignature::read(&input)?;
                (input, Self::MethodHandle(keyword, method))
            } else if let Ok((input, method)) = MethodSignature::read(start) {
                (input, Self::Method(method))
            } else if let Some(value) = keyword.strip_suffix('t') {
                let number = parse_integer!(value, i8)
                    .map_err(|_| start.unexpected("a byte literal".into()))?;
                (input, Self::Byte(number))
            } else if let Some(value) = keyword.strip_suffix('s') {
                let number = parse_integer!(value, i16)
                    .map_err(|_| start.unexpected("a short literal".into()))?;
                (input, Self::Short(number))
            } else if let Some(value) = keyword.strip_suffix('l') {
                let number = parse_integer!(value, i64)
                    .map_err(|_| start.unexpected("a long literal".into()))?;
                (input, Self::Long(number))
            } else if keyword.find('.').is_some()
                || keyword.starts_with("infinity")
                || keyword.starts_with("-infinity")
                || keyword.starts_with("nan")
            {
                if let Some(value) = keyword.strip_suffix('f') {
                    let number = f32::from_str(value)
                        .map_err(|_| start.unexpected("a float literal".into()))?;
                    (input, Self::Float(number))
                } else {
                    let value = if let Some(v) = keyword.strip_suffix('d') {
                        v
                    } else {
                        &keyword
                    };
                    let number = f64::from_str(value)
                        .map_err(|_| start.unexpected("a double literal".into()))?;
                    (input, Self::Double(number))
                }
            } else if let Ok(number) = parse_integer!(keyword, i32) {
                (input, Self::Int(number))
            } else if let Ok((input, class)) = Type::read(start) {
                (input, Self::Class(class))
            } else {
                return Err(start.unexpected("a literal".into()));
            }
        })
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn get_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_char(&self) -> bool {
        matches!(self, Self::Char(_))
    }

    pub fn get_char(&self) -> Option<char> {
        match *self {
            Self::Char(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Byte(_) | Self::Short(_) | Self::Int(_) | Self::Long(_)
        )
    }

    pub fn get_integer(&self) -> Option<i64> {
        match *self {
            Self::Byte(value) => Some(value as i64),
            Self::Short(value) => Some(value as i64),
            Self::Int(value) => Some(value as i64),
            Self::Long(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_) | Self::Double(_))
    }

    pub fn get_float(&self) -> Option<f64> {
        match *self {
            Self::Float(value) => Some(value as f64),
            Self::Double(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            Self::String(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(_))
    }

    pub fn is_method(&self) -> bool {
        matches!(self, Self::Method(_))
    }

    pub fn is_method_handle(&self) -> bool {
        matches!(self, Self::MethodHandle(_, _))
    }

    pub fn is_method_type(&self) -> bool {
        matches!(self, Self::MethodType(_))
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(value) => {
                if *value {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Self::Char(value) => {
                if *value < ' ' || *value >= '\u{007f}' {
                    write!(f, "'\\u{:04x}'", *value as u32)
                } else if *value == '\\' || *value == '\'' {
                    write!(f, "'\\{value}'")
                } else {
                    write!(f, "'{value}'")
                }
            }
            Self::Byte(value) => {
                write!(
                    f,
                    "{}{:#x}",
                    if value.is_negative() { "-" } else { "" },
                    value.abs_diff(0)
                )
            }
            Self::Short(value) => {
                write!(
                    f,
                    "{}{:#x}",
                    if value.is_negative() { "-" } else { "" },
                    value.abs_diff(0)
                )
            }
            Self::Int(value) => {
                write!(
                    f,
                    "{}{:#x}",
                    if value.is_negative() { "-" } else { "" },
                    value.abs_diff(0)
                )
            }
            Self::Long(value) => {
                write!(
                    f,
                    "{}{:#x}",
                    if value.is_negative() { "-" } else { "" },
                    value.abs_diff(0)
                )
            }
            Self::Float(value) => write!(f, "{value}"),
            Self::Double(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
            Self::Class(class) => write!(f, "{class}.class"),
            Self::Method(method) => write!(f, "{method}"),
            Self::MethodHandle(invoke_type, method) => write!(f, "{invoke_type}@{method}"),
            Self::MethodType(method_type) => write!(f, "{method_type}"),
        }
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
    fn read_keywords() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#" NULL true False dummy "#);
        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Null);

        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Bool(true));

        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Bool(false));

        assert!(Literal::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_string() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#" "a\"b c\\" "#);
        let (_, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::String(r#"a\"b c\\"#.to_string()));

        let input = tokenizer(r#" "a\"b c\\ "#);
        assert!(Literal::read(&input).is_err());

        let input = tokenizer(
            r#" "a\"
            b c\\" "#,
        );
        assert!(Literal::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_char() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#"'c' 'cc' '\c' '\'' '\\' '\u007f' '\' "#);
        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Char('c'));

        assert!(Literal::read(&input).is_err());
        let (input, _) = input.read_keyword()?;

        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Char('c'));

        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Char('\''));

        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Char('\\'));

        let (input, literal) = Literal::read(&input)?;
        assert_eq!(literal, Literal::Char('\u{007f}'));

        assert!(Literal::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_integer() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#" -5,0x1D -0x1f -0x80t  1234S 12x "#);
        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Int(-5));

        assert!(Literal::read(&input).is_err());
        let input = input.expect_char(',')?;

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Int(29));

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Int(-31));

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Byte(-128));

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Short(1234));

        assert!(Literal::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_float() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#" -infinity NANf infinityd .01f 2.3D .x "#);
        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Double(f64::NEG_INFINITY));

        let (input, number) = Literal::read(&input)?;
        assert!(matches!(number, Literal::Float(value) if value.is_nan()));

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Double(f64::INFINITY));

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Float(0.01));

        let (input, number) = Literal::read(&input)?;
        assert_eq!(number, Literal::Double(2.3));

        assert!(Literal::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Literal::Null), "null");

        assert_eq!(format!("{}", Literal::Bool(false)), "false");
        assert_eq!(format!("{}", Literal::Bool(true)), "true");

        assert_eq!(format!("{}", Literal::Char('x')), "'x'");
        assert_eq!(format!("{}", Literal::Char('\\')), "'\\\\'");
        assert_eq!(format!("{}", Literal::Char('\'')), "'\\\''");
        assert_eq!(format!("{}", Literal::Char('\u{0000}')), "'\\u0000'");
        assert_eq!(format!("{}", Literal::Char('\u{007f}')), "'\\u007f'");
        assert_eq!(format!("{}", Literal::Char('\u{1234}')), "'\\u1234'");

        assert_eq!(format!("{}", Literal::Byte(0)), "0x0");
        assert_eq!(format!("{}", Literal::Byte(0x7f)), "0x7f");
        assert_eq!(format!("{}", Literal::Byte(-0x80)), "-0x80");

        assert_eq!(format!("{}", Literal::Short(0)), "0x0");
        assert_eq!(format!("{}", Literal::Short(0x7fff)), "0x7fff");
        assert_eq!(format!("{}", Literal::Short(-0x8000)), "-0x8000");

        assert_eq!(format!("{}", Literal::Int(0)), "0x0");
        assert_eq!(format!("{}", Literal::Int(0x7fffffff)), "0x7fffffff");
        assert_eq!(format!("{}", Literal::Int(-0x80000000)), "-0x80000000");

        assert_eq!(format!("{}", Literal::Long(0)), "0x0");
        assert_eq!(
            format!("{}", Literal::Long(0x7fffffffffffffff)),
            "0x7fffffffffffffff"
        );
        assert_eq!(
            format!("{}", Literal::Long(-0x8000000000000000)),
            "-0x8000000000000000"
        );

        assert_eq!(format!("{}", Literal::Float(0.0)), "0");
        assert_eq!(format!("{}", Literal::Float(5.8)), "5.8");
        assert_eq!(format!("{}", Literal::Float(-0.1)), "-0.1");

        assert_eq!(format!("{}", Literal::Double(0.0)), "0");
        assert_eq!(format!("{}", Literal::Double(5.8)), "5.8");
        assert_eq!(format!("{}", Literal::Double(-0.1)), "-0.1");

        assert_eq!(format!("{}", Literal::String("abc".to_string())), "\"abc\"");
        assert_eq!(
            format!("{}", Literal::String("a\\tb\\\\c".to_string())),
            "\"a\\tb\\\\c\""
        );
    }
}
