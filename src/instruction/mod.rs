use itertools::Itertools;
use std::fmt::{Display, Formatter};

use crate::literal::Literal;
use crate::r#type::{CallSignature, FieldSignature, MethodSignature, Type};

mod jimple;
mod optimization;
mod parameters_smali;
mod registers_smali;
mod smali;

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
    Result,
    DefaultEmptyResult,
    Register,
    Registers,
    Literal,
    Label,
    Type,
    Field,
    Method,
    MethodHandle,
    Call,
    Data,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultTypeDef {
    None,
    Bool,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Object(&'static str),
    From(usize),
    ElementFrom(usize),
    Exception,
    Method,
    MethodHandle,
}

#[derive(Debug, Clone, PartialEq)]
struct InstructionDef {
    parameters: &'static [ParameterKind],
    format: &'static str,
    is_moved_result: bool,
    result_type: ResultTypeDef,
}

impl InstructionDef {
    const fn default() -> Self {
        Self {
            parameters: &[],
            format: "",
            is_moved_result: false,
            result_type: ResultTypeDef::None,
        }
    }
}

macro_rules! instructions {
    (
        $(
            $command:literal => [$($kind:ident)*]
                $format:literal
                $($field:ident = $value:expr)*,
        )*
    ) => {
        phf::phf_map! {
            $(
                $command => InstructionDef {
                    parameters: &[$(
                        ParameterKind::$kind,
                    )*],
                    format: $format,
                    $($field: $value,)*
                    ..InstructionDef::default()
                },
            )*
        }
    }
}

#[allow(clippy::needless_update)]
const DEFS: phf::Map<&str, InstructionDef> = instructions!(
    "nop" => [] "nop",
    "move" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move/from16" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move/16" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-wide" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-wide/from16" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-wide/16" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-object" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-object/from16" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-object/16" => [Result Register] "{1}" result_type=ResultTypeDef::From(1),
    "move-result" => [Result] "move-result" is_moved_result=true result_type=ResultTypeDef::Int,
    "move-result-wide" => [Result] "move-result" is_moved_result=true result_type=ResultTypeDef::Long,
    "move-result-object" => [Result] "move-result" is_moved_result=true result_type=ResultTypeDef::Object("java.lang.Object"),
    "move-exception" => [Result] "move-exception" result_type=ResultTypeDef::Exception,
    "return-void" => [] "return",
    "return" => [Register] "return {0}",
    "return-wide" => [Register] "return {0}",
    "return-object" => [Register] "return {0}",
    "const/4" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const/16" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const/high16" => [Result Literal] "{1} << 16" result_type=ResultTypeDef::From(1),
    "const-wide/16" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const-wide/32" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const-wide" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const-wide/high16" => [Result Literal] "{1} << 48" result_type=ResultTypeDef::From(1),
    "const-string" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const-string/jumbo" => [Result Literal] "{1}" result_type=ResultTypeDef::From(1),
    "const-class" => [Result Type] "class {1}" result_type=ResultTypeDef::Object("java.lang.Class"),
    "monitor-enter" => [Register] "monitor-enter {0}",
    "monitor-exit" => [Register] "monitor-exit {0}",
    "check-cast" => [DefaultEmptyResult Register Type] "({2}) {1}" result_type=ResultTypeDef::From(2),
    "instance-of" => [Result Register Type] "{1} instance-of {2}" result_type=ResultTypeDef::From(2),
    "array-length" => [Result Register] "array-length {1}" result_type=ResultTypeDef::Int,
    "new-instance" => [Result Type] "new {1}" result_type=ResultTypeDef::From(1),
    "new-array" => [Result Register Type] "new {2}[{1}]" result_type=ResultTypeDef::From(2),
    "filled-new-array" => [DefaultEmptyResult Registers Type] "{{1}}" result_type=ResultTypeDef::From(2),
    "filled-new-array/range" => [DefaultEmptyResult Registers Type] "{{1}}" result_type=ResultTypeDef::From(2),
    "fill-array-data" => [Register Data] "{0} = {\n{1}        }",
    "throw" => [Register] "throw {0}",
    "goto" => [Label] "goto {0}",
    "goto/16" => [Label] "goto {0}",
    "goto/32" => [Label] "goto {0}",
    "packed-switch" => [Register Data] "switch({0})\n        {\n{1}        }",
    "sparse-switch" => [Register Data] "switch({0})\n        {\n{1}        }",
    "cmpl-float" => [Result Register Register] "{1} cmpl {2}" result_type=ResultTypeDef::Bool,
    "cmpg-float" => [Result Register Register] "{1} cmpg {2}" result_type=ResultTypeDef::Bool,
    "cmpl-double" => [Result Register Register] "{1} cmpl {2}" result_type=ResultTypeDef::Bool,
    "cmpg-double" => [Result Register Register] "{1} cmpg {2}" result_type=ResultTypeDef::Bool,
    "cmp-long" => [Result Register Register] "{1} cmp {2}" result_type=ResultTypeDef::Bool,
    "if-eq" => [Register Register Label] "if ({0} == {1}) goto {2}",
    "if-ne" => [Register Register Label] "if ({0} != {1}) goto {2}",
    "if-lt" => [Register Register Label]  "if ({0} < {1}) goto {2}",
    "if-ge" => [Register Register Label] "if ({0} >= {1}) goto {2}",
    "if-gt" => [Register Register Label]  "if ({0} > {1}) goto {2}",
    "if-le" => [Register Register Label]  "if ({0} <= {1}) goto {2}",
    "if-eqz" => [Register Label] "if ({0} == 0) goto {1}",
    "if-nez" => [Register Label] "if ({0} != 0) goto {1}",
    "if-ltz" => [Register Label] "if ({0} < 0) goto {1}",
    "if-gez" => [Register Label] "if ({0} >= 0) goto {1}",
    "if-gtz" => [Register Label] "if ({0} > 0) goto {1}",
    "if-lez" => [Register Label] "if ({0} <= 0) goto {1}",
    "aget" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aget-wide" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aget-object" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aget-boolean" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aget-byte" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aget-char" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aget-short" => [Result Register Register] "{1}[{2}]" result_type=ResultTypeDef::ElementFrom(1),
    "aput" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-wide" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-object" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-boolean" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-byte" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-char" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-short" => [Register Register Register] "{1}[{2}] = {0}",
    "iget" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iget-wide" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iget-object" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iget-boolean" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iget-byte" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iget-char" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iget-short" => [Result Register Field] "{1}.<{2}>" result_type=ResultTypeDef::From(2),
    "iput" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-wide" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-object" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-boolean" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-byte" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-char" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-short" => [Register Register Field] "{1}.<{2}> = {0}",
    "sget" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sget-wide" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sget-object" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sget-boolean" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sget-byte" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sget-char" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sget-short" => [Result Field] "<{1}>" result_type=ResultTypeDef::From(1),
    "sput" => [Register Field] "<{1}> = {0}",
    "sput-wide" => [Register Field] "<{1}> = {0}",
    "sput-object" => [Register Field] "<{1}> = {0}",
    "sput-boolean" => [Register Field] "<{1}> = {0}",
    "sput-byte" => [Register Field] "<{1}> = {0}",
    "sput-char" => [Register Field] "<{1}> = {0}",
    "sput-short" => [Register Field] "<{1}> = {0}",
    "invoke-virtual" => [DefaultEmptyResult Registers Method] "invoke-virtual {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-super" => [DefaultEmptyResult Registers Method] "invoke-super {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-direct" => [DefaultEmptyResult Registers Method] "invoke-direct {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-static" => [DefaultEmptyResult Registers Method] "invoke-static <{2}>({1})" result_type=ResultTypeDef::From(2),
    "invoke-interface" => [DefaultEmptyResult Registers Method] "invoke-interface {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-virtual/range" => [DefaultEmptyResult Registers Method] "invoke-virtual {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-super/range" => [DefaultEmptyResult Registers Method] "invoke-super {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-direct/range" => [DefaultEmptyResult Registers Method] "invoke-direct {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "invoke-static/range" => [DefaultEmptyResult Registers Method] "invoke-static <{2}>({1})" result_type=ResultTypeDef::From(2),
    "invoke-interface/range" => [DefaultEmptyResult Registers Method] "invoke-interface {1.this}.<{2}>({1.args})" result_type=ResultTypeDef::From(2),
    "neg-int" => [Result Register] "-{1}" result_type=ResultTypeDef::From(1),
    "not-int" => [Result Register] "~{1}" result_type=ResultTypeDef::From(1),
    "neg-long" => [Result Register] "-{1}" result_type=ResultTypeDef::From(1),
    "not-long" => [Result Register] "~{1}" result_type=ResultTypeDef::From(1),
    "neg-float" => [Result Register] "-{1}" result_type=ResultTypeDef::From(1),
    "neg-double" => [Result Register] "-{1}" result_type=ResultTypeDef::From(1),
    "int-to-long" => [Result Register] "(long) {1}" result_type=ResultTypeDef::Long,
    "int-to-float" => [Result Register] "(float) {1}" result_type=ResultTypeDef::Float,
    "int-to-double" => [Result Register] "(double) {1}" result_type=ResultTypeDef::Double,
    "long-to-int" => [Result Register] "(int) {1}" result_type=ResultTypeDef::Int,
    "long-to-float" => [Result Register] "(float) {1}" result_type=ResultTypeDef::Float,
    "long-to-double" => [Result Register] "(double) {1}" result_type=ResultTypeDef::Double,
    "float-to-int" => [Result Register] "(int) {1}" result_type=ResultTypeDef::Int,
    "float-to-long" => [Result Register] "(long) {1}" result_type=ResultTypeDef::Long,
    "float-to-double" => [Result Register] "(double) {1}" result_type=ResultTypeDef::Double,
    "double-to-int" => [Result Register] "(int) {1}" result_type=ResultTypeDef::Int,
    "double-to-long" => [Result Register] "(long) {1}" result_type=ResultTypeDef::Long,
    "double-to-float" => [Result Register] "(float) {1}" result_type=ResultTypeDef::Float,
    "int-to-byte" => [Result Register] "(byte) {1}" result_type=ResultTypeDef::Byte,
    "int-to-char" => [Result Register] "(char) {1}" result_type=ResultTypeDef::Char,
    "int-to-short" => [Result Register] "(short) {1}" result_type=ResultTypeDef::Short,
    "add-int" => [Result Register Register] "{1} + {2}" result_type=ResultTypeDef::From(1),
    "sub-int" => [Result Register Register] "{1} - {2}" result_type=ResultTypeDef::From(1),
    "mul-int" => [Result Register Register] "{1} * {2}" result_type=ResultTypeDef::From(1),
    "div-int" => [Result Register Register] "{1} / {2}" result_type=ResultTypeDef::From(1),
    "rem-int" => [Result Register Register] "{1} % {2}" result_type=ResultTypeDef::From(1),
    "and-int" => [Result Register Register] "{1} & {2}" result_type=ResultTypeDef::From(1),
    "or-int" => [Result Register Register] "{1} | {2}" result_type=ResultTypeDef::From(1),
    "xor-int" => [Result Register Register] "{1} ^ {2}" result_type=ResultTypeDef::From(1),
    "shl-int" => [Result Register Register] "{1} << {2}" result_type=ResultTypeDef::From(1),
    "shr-int" => [Result Register Register] "{1} >> {2}" result_type=ResultTypeDef::From(1),
    "ushr-int" => [Result Register Register] "{1} >>> {2}" result_type=ResultTypeDef::From(1),
    "add-long" => [Result Register Register] "{1} + {2}" result_type=ResultTypeDef::From(1),
    "sub-long" => [Result Register Register] "{1} - {2}" result_type=ResultTypeDef::From(1),
    "mul-long" => [Result Register Register] "{1} * {2}" result_type=ResultTypeDef::From(1),
    "div-long" => [Result Register Register] "{1} / {2}" result_type=ResultTypeDef::From(1),
    "rem-long" => [Result Register Register] "{1} % {2}" result_type=ResultTypeDef::From(1),
    "and-long" => [Result Register Register] "{1} & {2}" result_type=ResultTypeDef::From(1),
    "or-long" => [Result Register Register] "{1} | {2}" result_type=ResultTypeDef::From(1),
    "xor-long" => [Result Register Register] "{1} ^ {2}" result_type=ResultTypeDef::From(1),
    "shl-long" => [Result Register Register] "{1} << {2}" result_type=ResultTypeDef::From(1),
    "shr-long" => [Result Register Register] "{1} >> {2}" result_type=ResultTypeDef::From(1),
    "ushr-long" => [Result Register Register] "{1} >>> {2}" result_type=ResultTypeDef::From(1),
    "add-float" => [Result Register Register] "{1} + {2}" result_type=ResultTypeDef::From(1),
    "sub-float" => [Result Register Register] "{1} - {2}" result_type=ResultTypeDef::From(1),
    "mul-float" => [Result Register Register] "{1} * {2}" result_type=ResultTypeDef::From(1),
    "div-float" => [Result Register Register] "{1} / {2}" result_type=ResultTypeDef::From(1),
    "rem-float" => [Result Register Register] "{1} % {2}" result_type=ResultTypeDef::From(1),
    "add-double" => [Result Register Register] "{1} + {2}" result_type=ResultTypeDef::From(1),
    "sub-double" => [Result Register Register] "{1} - {2}" result_type=ResultTypeDef::From(1),
    "mul-double" => [Result Register Register] "{1} * {2}" result_type=ResultTypeDef::From(1),
    "div-double" => [Result Register Register] "{1} / {2}" result_type=ResultTypeDef::From(1),
    "rem-double" => [Result Register Register] "{1} % {2}" result_type=ResultTypeDef::From(1),
    "add-int/2addr" => [Register Register] "{0} += {1}",
    "sub-int/2addr" => [Register Register] "{0} -= {1}",
    "mul-int/2addr" => [Register Register] "{0} *= {1}",
    "div-int/2addr" => [Register Register] "{0} /= {1}",
    "rem-int/2addr" => [Register Register] "{0} %= {1}",
    "and-int/2addr" => [Register Register] "{0} &= {1}",
    "or-int/2addr" => [Register Register] "{0} |= {1}",
    "xor-int/2addr" => [Register Register] "{0} ^= {1}",
    "shl-int/2addr" => [Register Register] "{0} <<= {1}",
    "shr-int/2addr" => [Register Register] "{0} >>= {1}",
    "ushr-int/2addr" => [Register Register] "{0} >>>= {1}",
    "add-long/2addr" => [Register Register] "{0} += {1}",
    "sub-long/2addr" => [Register Register] "{0} -= {1}",
    "mul-long/2addr" => [Register Register] "{0} *= {1}",
    "div-long/2addr" => [Register Register] "{0} /= {1}",
    "rem-long/2addr" => [Register Register] "{0} %= {1}",
    "and-long/2addr" => [Register Register] "{0} &= {1}",
    "or-long/2addr" => [Register Register] "{0} |= {1}",
    "xor-long/2addr" => [Register Register] "{0} ^= {1}",
    "shl-long/2addr" => [Register Register] "{0} <<= {1}",
    "shr-long/2addr" => [Register Register] "{0} >>= {1}",
    "ushr-long/2addr" => [Register Register] "{0} >>>= {1}",
    "add-float/2addr" => [Register Register] "{0} += {1}",
    "sub-float/2addr" => [Register Register] "{0} -= {1}",
    "mul-float/2addr" => [Register Register] "{0} *= {1}",
    "div-float/2addr" => [Register Register] "{0} /= {1}",
    "rem-float/2addr" => [Register Register] "{0} %= {1}",
    "add-double/2addr" => [Register Register] "{0} += {1}",
    "sub-double/2addr" => [Register Register] "{0} -= {1}",
    "mul-double/2addr" => [Register Register] "{0} *= {1}",
    "div-double/2addr" => [Register Register] "{0} /= {1}",
    "rem-double/2addr" => [Register Register] "{0} %= {1}",
    "add-int/lit16" => [Result Register Literal] "{1} + {2}" result_type=ResultTypeDef::From(1),
    "rsub-int" => [Result Register Literal] "{2} - {1}" result_type=ResultTypeDef::From(1),
    "mul-int/lit16" => [Result Register Literal] "{1} * {2}" result_type=ResultTypeDef::From(1),
    "div-int/lit16" => [Result Register Literal] "{1} / {2}" result_type=ResultTypeDef::From(1),
    "rem-int/lit16" => [Result Register Literal] "{1} % {2}" result_type=ResultTypeDef::From(1),
    "and-int/lit16" => [Result Register Literal] "{1} & {2}" result_type=ResultTypeDef::From(1),
    "or-int/lit16" => [Result Register Literal] "{1} | {2}" result_type=ResultTypeDef::From(1),
    "xor-int/lit16" => [Result Register Literal] "{1} ^ {2}" result_type=ResultTypeDef::From(1),
    "add-int/lit8" => [Result Register Literal] "{1} + {2}" result_type=ResultTypeDef::From(1),
    "rsub-int/lit8" => [Result Register Literal] "{2} - {1}" result_type=ResultTypeDef::From(1),
    "mul-int/lit8" => [Result Register Literal] "{1} * {2}" result_type=ResultTypeDef::From(1),
    "div-int/lit8" => [Result Register Literal] "{1} / {2}" result_type=ResultTypeDef::From(1),
    "rem-int/lit8" => [Result Register Literal] "{1} % {2}" result_type=ResultTypeDef::From(1),
    "and-int/lit8" => [Result Register Literal] "{1} & {2}" result_type=ResultTypeDef::From(1),
    "or-int/lit8" => [Result Register Literal] "{1} | {2}" result_type=ResultTypeDef::From(1),
    "xor-int/lit8" => [Result Register Literal] "{1} ^ {2}" result_type=ResultTypeDef::From(1),
    "shl-int/lit8" => [Result Register Literal] "{1} << {2}" result_type=ResultTypeDef::From(1),
    "shr-int/lit8" => [Result Register Literal] "{1} >> {2}" result_type=ResultTypeDef::From(1),
    "ushr-int/lit8" => [Result Register Literal] "{1} >>> {2}" result_type=ResultTypeDef::From(1),
    "invoke-polymorphic" => [DefaultEmptyResult Registers Method Call] "invoke-polymorphic {1.this}.<{2}>({1.args}), <{3}>" result_type=ResultTypeDef::From(2),
    "invoke-polymorphic/range" => [DefaultEmptyResult Registers Method Call] "invoke-polymorphic {1.this}.<{2}>({1.args}), <{3}>" result_type=ResultTypeDef::From(2),
    // TODO: invoke-custom and invoke-custom/range
    "const-method-handle" => [Result MethodHandle] "{1}" result_type=ResultTypeDef::MethodHandle,
    "const-method-type" => [Result Call] "{1}" result_type=ResultTypeDef::Method,
);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Register {
    Parameter(usize),
    Local(usize),
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Parameter(index) => write!(f, "p{index}"),
            Self::Local(index) => write!(f, "v{index}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    This,
    Parameter(usize, Type),
    Local(usize, Type),
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::This => write!(f, "@this"),
            Self::Parameter(index, _) => write!(f, "@p{index}"),
            Self::Local(index, _) => write!(f, "$v{index}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Registers {
    List(Vec<Register>),
    Range(Register, Register),
}

impl Registers {
    fn resolve_range(from: &Register, to: &Register) -> Option<Vec<Register>> {
        if let (Register::Parameter(from_index), Register::Parameter(to_index)) = (from, to) {
            Some(
                (*from_index..to_index + 1)
                    .map(Register::Parameter)
                    .collect(),
            )
        } else if let (Register::Local(from_index), Register::Local(to_index)) = (from, to) {
            Some((*from_index..to_index + 1).map(Register::Local).collect())
        } else {
            eprintln!("Warning: Invalid parameter range: {from} .. {to}");
            None
        }
    }

    fn stringify_list(list: &[Register], split_first: bool) -> (Option<String>, String) {
        if split_first && !list.is_empty() {
            (Some(list[0].to_string()), list[1..].iter().join(", "))
        } else {
            (None, list.iter().join(", "))
        }
    }

    pub fn to_string(&self, split_first: bool) -> (Option<String>, String) {
        match self {
            Self::List(list) => Self::stringify_list(list, split_first),
            Self::Range(from, to) => {
                if let Some(list) = Self::resolve_range(from, to) {
                    Self::stringify_list(&list, split_first)
                } else {
                    (None, format!("{from} .. {to}"))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandData {
    Label(String),
    PackedSwitch(i64, Vec<String>),
    SparseSwitch(Vec<(Literal, String)>),
    Array(Vec<Literal>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandParameter {
    Result(Register),
    DefaultEmptyResult(Option<Register>),
    Register(Register),
    Variable(Variable),
    Registers(Registers),
    Literal(Literal),
    Label(String),
    Type(Type),
    Field(FieldSignature),
    Method(MethodSignature),
    MethodHandle(String, MethodSignature),
    Call(CallSignature),
    Data(CommandData),
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    LineNumber(i64, i64),
    Label(String),
    Command {
        command: String,
        parameters: Vec<CommandParameter>,
    },
    Catch {
        exception: Option<Type>,
        start_label: String,
        end_label: String,
        target: String,
    },
    Local {
        register: String,
        name: Literal,
        local_type: Type,
    },
    LocalRestart {
        register: String,
    },
    Data(CommandData),
}

impl Instruction {
    pub fn is_command(&self) -> bool {
        matches!(self, Instruction::Command { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultType {
    Type(Type),
    Literal(Literal),
    Method,
    MethodHandle,
}

impl From<Type> for ResultType {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

impl From<&Type> for ResultType {
    fn from(value: &Type) -> Self {
        Self::Type(value.clone())
    }
}

impl From<Literal> for ResultType {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl From<&Literal> for ResultType {
    fn from(value: &Literal) -> Self {
        Self::Literal(value.clone())
    }
}
