use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::error::{Error, ParseError};

#[derive(Debug, Clone)]
pub struct Tokenizer {
    pos: usize,
    data: Rc<String>,
    path: Rc<PathBuf>,
}

impl Tokenizer {
    pub fn new(data: String, path: &Path) -> Self {
        Self {
            pos: 0,
            data: Rc::new(data),
            path: Rc::new(path.to_path_buf()),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let data = std::fs::read(path).map_err(|_| Error::ReadFailure(path.to_path_buf()))?;
        let data = String::from_utf8(data).map_err(|_| Error::Utf8Error(path.to_path_buf()))?;
        Ok(Self::new(data, path))
    }

    fn data(&self) -> &str {
        &self.data[self.pos..]
    }

    fn skip_whitespace(&self) -> Self {
        let mut input = self.clone();
        for c in self.data().chars() {
            if c != ' ' && c != '\t' {
                break;
            }
            input.pos += 1;
        }
        input
    }

    pub fn read_to(&self, chars: &[char]) -> (Self, String) {
        let max = self.data().find('\n').unwrap_or(self.data().len());
        let index = self.data().find(chars).unwrap_or(max);
        let index = std::cmp::min(index, max);

        let mut input = self.clone();
        input.pos += index;

        (input, self.data[self.pos..self.pos + index].to_string())
    }

    fn read_to_whitespace(&self) -> (Self, String) {
        self.read_to(&[' ', '\t'])
    }

    pub fn next_char(&self) -> Option<char> {
        self.data().chars().next()
    }

    pub fn read_char(&self) -> Result<(Self, char), ParseError> {
        let mut input = self.skip_whitespace();
        let c = input
            .data()
            .chars()
            .next()
            .ok_or_else(|| input.unexpected("a char".into()))?;

        input.pos += 1;
        Ok((input, c))
    }

    pub fn expect_char(&self, c: char) -> Result<Self, ParseError> {
        fn expected(c: char) -> Cow<'static, str> {
            if c == '\n' {
                Cow::Borrowed("<EOL>")
            } else {
                Cow::Owned(format!("the character '{c}'"))
            }
        }

        let (input, next) = self.read_char().map_err(|_| self.unexpected(expected(c)))?;

        if next == c {
            Ok(input)
        } else {
            Err(self.unexpected(expected(c)))
        }
    }

    pub fn expect_eol(&self) -> Result<Self, ParseError> {
        let input = if let Ok(input) = self.expect_char('#') {
            let (input, _) = input.read_to(&['\n']);
            input
        } else {
            self.skip_whitespace()
        };

        if input.expect_eof().is_ok() {
            return Ok(input);
        }

        let mut input = input.expect_char('\n')?;
        loop {
            if let Ok(i) = input.expect_char('#') {
                input = i;
                (input, _) = input.read_to(&['\n']);
                if let Ok(i) = input.expect_char('\n') {
                    input = i;
                }
            } else if let Ok(i) = input.expect_char('\n') {
                input = i;
            } else {
                break;
            }
        }
        Ok(input)
    }

    pub fn read_keyword(&self) -> Result<(Self, String), ParseError> {
        let input = self.skip_whitespace();
        let (input, keyword) = input.read_to(&[' ', '\t', ',', ':', '(', ')', '{', '}', '#', '@']);
        if keyword.is_empty() {
            Err(input.unexpected("a keyword".into()))
        } else {
            Ok((input, keyword))
        }
    }

    pub fn expect_keyword(&self, expected: &str) -> Result<Self, ParseError> {
        let (input, keyword) = self
            .read_keyword()
            .map_err(|_| self.unexpected(expected.to_string().into()))?;
        if keyword == expected {
            Ok(input)
        } else {
            Err(self.unexpected(expected.to_string().into()))
        }
    }

    pub fn read_directive(&self) -> Result<(Self, String), ParseError> {
        let input = self
            .expect_char('.')
            .map_err(|_| self.unexpected("a directive".into()))?;
        let (input, directive) = input.read_to_whitespace();
        if directive.is_empty() {
            Err(self.unexpected("a directive".into()))
        } else {
            Ok((input, directive))
        }
    }

    pub fn expect_directive(&self, expected: &str) -> Result<Self, ParseError> {
        let (input, directive) = self
            .read_directive()
            .map_err(|_| self.unexpected((".".to_string() + expected).into()))?;
        if directive == expected {
            Ok(input)
        } else {
            Err(self.unexpected((".".to_string() + expected).into()))
        }
    }

    pub fn read_number(&self) -> Result<(Self, i64), ParseError> {
        let (input, keyword) = self.read_keyword()?;
        let keyword = keyword.trim_end_matches(['t', 'T', 's', 'S', 'l', 'L']);
        let number = if let Some(keyword) = keyword.strip_prefix("-0x") {
            i64::from_str_radix(keyword, 16).map(|i| -i)
        } else if let Some(keyword) = keyword.strip_prefix("0x") {
            i64::from_str_radix(keyword, 16)
        } else {
            keyword.parse()
        }
        .map_err(|_| self.unexpected("a number".into()))?;
        Ok((input, number))
    }

    pub fn expect_eof(&self) -> Result<Self, ParseError> {
        if self.data().is_empty() {
            Ok(self.clone())
        } else {
            Err(self.unexpected("<EOF>".into()))
        }
    }

    pub fn unexpected(&self, expected: Cow<'static, str>) -> ParseError {
        ParseError::new(self.path.clone(), self.data.clone(), self.pos, expected)
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
    fn read_to() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer("abc;xyz,def\nghi;");

        let (input, s) = input.read_to(&[';', ',']);
        assert_eq!(s, "abc");

        let (input, s) = input.read_to(&[';', ',']);
        assert_eq!(s, "");

        let input = input.expect_char(';')?;
        let (input, s) = input.read_to(&[';', ',']);
        assert_eq!(s, "xyz");

        let input = input.expect_char(',')?;
        let (input, s) = input.read_to(&[';', ',']);
        assert_eq!(s, "def");

        let (_, s) = input.read_to(&[';', ',']);
        assert_eq!(s, "");

        Ok(())
    }

    #[test]
    fn read_char() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" ab \n  ");

        assert!(matches!(input.next_char(), Some(' ')));
        let (input, c) = input.read_char()?;
        assert_eq!(c, 'a');

        assert!(matches!(input.next_char(), Some('b')));
        assert!(input.expect_char('\n').is_err());

        let input = input.expect_char('b')?;
        assert!(matches!(input.next_char(), Some(' ')));

        let input = input.expect_char('\n')?;
        assert!(matches!(input.next_char(), Some(' ')));
        assert!(input.read_char().is_err());

        Ok(())
    }

    #[test]
    fn read_eol_eof() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" abc#comment \n  \n # comment\n\nxyz");

        assert!(input.expect_eof().is_err());
        assert!(input.expect_eol().is_err());

        let (input, keyword) = input.read_keyword()?;
        assert_eq!(keyword, "abc");

        assert!(input.expect_eof().is_err());

        let input = input.expect_eol()?;
        assert!(matches!(input.next_char(), Some('x')));

        assert!(input.expect_eol().is_err());

        let (input, keyword) = input.read_keyword()?;
        assert_eq!(keyword, "xyz");

        assert!(input.expect_eof().is_ok());
        assert!(input.expect_eol().is_ok());

        Ok(())
    }

    #[test]
    fn read_keyword() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" abc, xyz:def ghi\njkl");

        let (input, keyword) = input.read_keyword()?;
        assert_eq!(keyword, "abc");
        assert!(input.read_keyword().is_err());

        let input = input.expect_char(',')?;

        assert!(input.expect_keyword("def").is_err());
        let input = input.expect_keyword("xyz")?;

        assert!(input.expect_keyword("def").is_err());
        let input = input.expect_char(':')?;
        let input = input.expect_keyword("def")?;
        let input = input.expect_keyword("ghi")?;

        assert!(input.read_keyword().is_err());
        let input = input.expect_eol()?;
        let (input, keyword) = input.read_keyword()?;
        assert_eq!(keyword, "jkl");

        assert!(input.expect_eof().is_ok());

        Ok(())
    }

    #[test]
    fn read_directive() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" .abc, .xyz:.def .ghi\n.jkl");

        let (input, directive) = input.read_directive()?;
        assert_eq!(directive, "abc,");

        assert!(input.expect_directive("xyz").is_err());
        let input = input.expect_directive("xyz:.def")?;

        let (input, directive) = input.read_directive()?;
        assert_eq!(directive, "ghi");

        assert!(input.read_directive().is_err());
        let input = input.expect_eol()?;
        let (input, directive) = input.read_directive()?;
        assert_eq!(directive, "jkl");

        assert!(input.read_directive().is_err());
        assert!(input.expect_eof().is_ok());

        Ok(())
    }

    #[test]
    fn read_number() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#" -5, 0x12 -0x12 0x41t  1234S 12x "#);
        let (input, number) = input.read_number()?;
        assert_eq!(number, -5);

        assert!(input.read_number().is_err());
        let input = input.expect_char(',')?;

        let (input, number) = input.read_number()?;
        assert_eq!(number, 18);

        let (input, number) = input.read_number()?;
        assert_eq!(number, -18);

        let (input, number) = input.read_number()?;
        assert_eq!(number, 65);

        let (input, number) = input.read_number()?;
        assert_eq!(number, 1234);

        assert!(input.read_number().is_err());

        Ok(())
    }
}
