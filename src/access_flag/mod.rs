use std::fmt::{Display, Formatter};

use crate::error::Error;

mod jimple;
mod smali;

/// An access flag specified on a class, field or method. See [dex format documentation](https://source.android.com/docs/core/runtime/dex-format#access-flags).
#[derive(Debug, PartialEq)]
pub enum AccessFlag {
    Public,
    Private,
    Protected,
    Static,
    Final,
    Synchronized,
    Volatile,
    Bridge,
    Transient,
    Varargs,
    Native,
    Interface,
    Abstract,
    Strictfp,
    Synthetic,
    Annotation,
    Enum,
    Constructor,
    DeclaredSynchronized,
}

impl TryFrom<&str> for AccessFlag {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "public" => Self::Public,
            "private" => Self::Private,
            "protected" => Self::Protected,
            "static" => Self::Static,
            "final" => Self::Final,
            "synchronized" => Self::Synchronized,
            "volatile" => Self::Volatile,
            "bridge" => Self::Bridge,
            "transient" => Self::Transient,
            "varargs" => Self::Varargs,
            "native" => Self::Native,
            "interface" => Self::Interface,
            "abstract" => Self::Abstract,
            "strictfp" => Self::Strictfp,
            "synthetic" => Self::Synthetic,
            "annotation" => Self::Annotation,
            "enum" => Self::Enum,
            "constructor" => Self::Constructor,
            "declared-synchronized" => Self::DeclaredSynchronized,
            other => return Err(Error::UnrecognizedToken(other.to_string())),
        })
    }
}

impl Display for AccessFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Public => "public",
                Self::Private => "private",
                Self::Protected => "protected",
                Self::Static => "static",
                Self::Final => "final",
                Self::Synchronized => "synchronized",
                Self::Volatile => "volatile",
                Self::Bridge => "bridge",
                Self::Transient => "transient",
                Self::Varargs => "varargs",
                Self::Native => "native",
                Self::Interface => "interface",
                Self::Abstract => "abstract",
                Self::Strictfp => "strictfp",
                Self::Synthetic => "synthetic",
                Self::Annotation => "annotation",
                Self::Enum => "enum",
                Self::Constructor => "constructor",
                Self::DeclaredSynchronized => "declared-synchronized",
            }
        )
    }
}
