use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnrecognizedToken(String),
    ReadFailure(PathBuf),
    Utf8Error(PathBuf),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::UnrecognizedToken(token) => write!(f, "Unrecognized token {token}"),
            Self::ReadFailure(path) => write!(f, "Failed to read file {}", path_to_string(path)),
            Self::Utf8Error(path) => write!(
                f,
                "Failed to decode file {}, not valid UTF-8",
                path_to_string(path)
            ),
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.as_os_str().to_str().unwrap_or("<unknown>").to_string()
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    path: Rc<PathBuf>,
    data: Rc<String>,
    pos: usize,
    expected: Cow<'static, str>,
}

impl ParseError {
    pub fn new(
        path: Rc<PathBuf>,
        data: Rc<String>,
        pos: usize,
        expected: Cow<'static, str>,
    ) -> Self {
        ParseError {
            path,
            data,
            pos,
            expected,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let prefix = &self.data[..self.pos];
        let line = prefix.matches('\n').count() + 1;
        let col = if let Some(index) = prefix.rfind('\n') {
            prefix.len() - index
        } else {
            prefix.len() + 1
        };

        let mut token = self.data[self.pos..].trim_start_matches([' ', '\t']);
        if token.is_empty() {
            token = "<EOF>";
        } else {
            if let Some(index) = token.find([' ', '\t', '\n']) {
                token = &token[..index];
            }
            if token.is_empty() {
                token = "<EOL>";
            }
        }

        write!(
            f,
            "Unexpected token {token} in {} at {line}:{col}, expected {}",
            path_to_string(&self.path),
            self.expected
        )
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq)]
pub struct ParseErrorDisplayed {
    pos: usize,
    message: String,
}

#[cfg(test)]
impl From<ParseError> for ParseErrorDisplayed {
    fn from(error: ParseError) -> Self {
        Self {
            pos: error.pos,
            message: format!("{error}"),
        }
    }
}
