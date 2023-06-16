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
struct InstructionDef {
    parameters: &'static [ParameterKind],
    format: &'static str,
}

impl InstructionDef {
    const fn default() -> Self {
        Self {
            parameters: &[],
            format: "",
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

const DEFS: phf::Map<&str, InstructionDef> = instructions!(
    "nop" => [] "nop",
    "move" => [Result Register] "{1}",
    "move/from16" => [Result Register] "{1}",
    "move/16" => [Result Register] "{1}",
    "move-wide" => [Result Register] "{1}",
    "move-wide/from16" => [Result Register] "{1}",
    "move-wide/16" => [Result Register] "{1}",
    "move-object" => [Result Register] "{1}",
    "move-object/from16" => [Result Register] "{1}",
    "move-object/16" => [Result Register] "{1}",
    "move-result" => [Result] "move-result",
    "move-result-wide" => [Result] "move-result",
    "move-result-object" => [Result] "move-result",
    "move-exception" => [Result] "move-exception",
    "return-void" => [] "return",
    "return" => [Register] "return {0}",
    "return-wide" => [Register] "return {0}",
    "return-object" => [Register] "return {0}",
    "const/4" => [Result Literal] "{1}",
    "const/16" => [Result Literal] "{1}",
    "const" => [Result Literal] "{1}",
    "const/high16" => [Result Literal] "{1} << 16",
    "const-wide/16" => [Result Literal] "{1}",
    "const-wide/32" => [Result Literal] "{1}",
    "const-wide" => [Result Literal] "{1}",
    "const-wide/high16" => [Result Literal] "{1} << 48",
    "const-string" => [Result Literal] "{1}",
    "const-string/jumbo" => [Result Literal] "{1}",
    "const-class" => [Result Type] "class {1}",
    "monitor-enter" => [Register] "monitor-enter {0}",
    "monitor-exit" => [Register] "monitor-exit {0}",
    "check-cast" => [Register Type] "check-cast = ({1}) {0}",
    "instance-of" => [Result Register Type] "{1} instance-of {2}",
    "array-length" => [Result Register] "array-length {1}",
    "new-instance" => [Result Type] "new {1}",
    "new-array" => [Result Register Type] "new {2}[{1}]",
    "filled-new-array" => [DefaultEmptyResult Registers Type] "{{1}}",
    "filled-new-array/range" => [DefaultEmptyResult Registers Type] "{{1}}",
    "fill-array-data" => [Register Data] "{0} = {\n{1}        }",
    "throw" => [Register] "throw {0}",
    "goto" => [Label] "goto {0}",
    "goto/16" => [Label] "goto {0}",
    "goto/32" => [Label] "goto {0}",
    "packed-switch" => [Register Data] "switch({0})\n        {\n{1}        }",
    "sparse-switch" => [Register Data] "switch({0})\n        {\n{1}        }",
    "cmpl-float" => [Result Register Register] "{1} cmpl {2}",
    "cmpg-float" => [Result Register Register] "{1} cmpg {2}",
    "cmpl-double" => [Result Register Register] "{1} cmpl {2}",
    "cmpg-double" => [Result Register Register] "{1} cmpg {2}",
    "cmp-long" => [Result Register Register] "{1} cmp {2}",
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
    "aget" => [Result Register Register] "{1}[{2}]",
    "aget-wide" => [Result Register Register] "{1}[{2}]",
    "aget-object" => [Result Register Register] "{1}[{2}]",
    "aget-boolean" => [Result Register Register] "{1}[{2}]",
    "aget-byte" => [Result Register Register] "{1}[{2}]",
    "aget-char" => [Result Register Register] "{1}[{2}]",
    "aget-short" => [Result Register Register] "{1}[{2}]",
    "aput" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-wide" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-object" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-boolean" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-byte" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-char" => [Register Register Register] "{1}[{2}] = {0}",
    "aput-short" => [Register Register Register] "{1}[{2}] = {0}",
    "iget" => [Result Register Field] "{1}.<{2}>",
    "iget-wide" => [Result Register Field] "{1}.<{2}>",
    "iget-object" => [Result Register Field] "{1}.<{2}>",
    "iget-boolean" => [Result Register Field] "{1}.<{2}>",
    "iget-byte" => [Result Register Field] "{1}.<{2}>",
    "iget-char" => [Result Register Field] "{1}.<{2}>",
    "iget-short" => [Result Register Field] "{1}.<{2}>",
    "iput" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-wide" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-object" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-boolean" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-byte" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-char" => [Register Register Field] "{1}.<{2}> = {0}",
    "iput-short" => [Register Register Field] "{1}.<{2}> = {0}",
    "sget" => [Result Field] "<{1}>",
    "sget-wide" => [Result Field] "<{1}>",
    "sget-object" => [Result Field] "<{1}>",
    "sget-boolean" => [Result Field] "<{1}>",
    "sget-byte" => [Result Field] "<{1}>",
    "sget-char" => [Result Field] "<{1}>",
    "sget-short" => [Result Field] "<{1}>",
    "sput" => [Register Field] "<{1}> = {0}",
    "sput-wide" => [Register Field] "<{1}> = {0}",
    "sput-object" => [Register Field] "<{1}> = {0}",
    "sput-boolean" => [Register Field] "<{1}> = {0}",
    "sput-byte" => [Register Field] "<{1}> = {0}",
    "sput-char" => [Register Field] "<{1}> = {0}",
    "sput-short" => [Register Field] "<{1}> = {0}",
    "invoke-virtual" => [DefaultEmptyResult Registers Method] "invoke-virtual {1.this}.<{2}>({1.args})",
    "invoke-super" => [DefaultEmptyResult Registers Method] "invoke-super {1.this}.<{2}>({1.args})",
    "invoke-direct" => [DefaultEmptyResult Registers Method] "invoke-direct {1.this}.<{2}>({1.args})",
    "invoke-static" => [DefaultEmptyResult Registers Method] "invoke-static <{2}>({1})",
    "invoke-interface" => [DefaultEmptyResult Registers Method] "invoke-interface {1.this}.<{2}>({1.args})",
    "invoke-virtual/range" => [DefaultEmptyResult Registers Method] "invoke-virtual {1.this}.<{2}>({1.args})",
    "invoke-super/range" => [DefaultEmptyResult Registers Method] "invoke-super {1.this}.<{2}>({1.args})",
    "invoke-direct/range" => [DefaultEmptyResult Registers Method] "invoke-direct {1.this}.<{2}>({1.args})",
    "invoke-static/range" => [DefaultEmptyResult Registers Method] "invoke-static <{2}>({1})",
    "invoke-interface/range" => [DefaultEmptyResult Registers Method] "invoke-interface {1.this}.<{2}>({1.args})",
    "neg-int" => [Result Register] "-{1}",
    "not-int" => [Result Register] "~{1}",
    "neg-long" => [Result Register] "-{1}",
    "not-long" => [Result Register] "~{1}",
    "neg-float" => [Result Register] "-{1}",
    "neg-double" => [Result Register] "-{1}",
    "int-to-long" => [Result Register] "(long) {1}",
    "int-to-float" => [Result Register] "(float) {1}",
    "int-to-double" => [Result Register] "(double) {1}",
    "long-to-int" => [Result Register] "(int) {1}",
    "long-to-float" => [Result Register] "(float) {1}",
    "long-to-double" => [Result Register] "(double) {1}",
    "float-to-int" => [Result Register] "(int) {1}",
    "float-to-long" => [Result Register] "(long) {1}",
    "float-to-double" => [Result Register] "(double) {1}",
    "double-to-int" => [Result Register] "(int) {1}",
    "double-to-long" => [Result Register] "(long) {1}",
    "double-to-float" => [Result Register] "(float) {1}",
    "int-to-byte" => [Result Register] "(byte) {1}",
    "int-to-char" => [Result Register] "(char) {1}",
    "int-to-short" => [Result Register] "(short) {1}",
    "add-int" => [Result Register Register] "{1} + {2}",
    "sub-int" => [Result Register Register] "{1} - {2}",
    "mul-int" => [Result Register Register] "{1} * {2}",
    "div-int" => [Result Register Register] "{1} / {2}",
    "rem-int" => [Result Register Register] "{1} % {2}",
    "and-int" => [Result Register Register] "{1} & {2}",
    "or-int" => [Result Register Register] "{1} | {2}",
    "xor-int" => [Result Register Register] "{1} ^ {2}",
    "shl-int" => [Result Register Register] "{1} << {2}",
    "shr-int" => [Result Register Register] "{1} >> {2}",
    "ushr-int" => [Result Register Register] "{1} >>> {2}",
    "add-long" => [Result Register Register] "{1} + {2}",
    "sub-long" => [Result Register Register] "{1} - {2}",
    "mul-long" => [Result Register Register] "{1} * {2}",
    "div-long" => [Result Register Register] "{1} / {2}",
    "rem-long" => [Result Register Register] "{1} % {2}",
    "and-long" => [Result Register Register] "{1} & {2}",
    "or-long" => [Result Register Register] "{1} | {2}",
    "xor-long" => [Result Register Register] "{1} ^ {2}",
    "shl-long" => [Result Register Register] "{1} << {2}",
    "shr-long" => [Result Register Register] "{1} >> {2}",
    "ushr-long" => [Result Register Register] "{1} >>> {2}",
    "add-float" => [Result Register Register] "{1} + {2}",
    "sub-float" => [Result Register Register] "{1} - {2}",
    "mul-float" => [Result Register Register] "{1} * {2}",
    "div-float" => [Result Register Register] "{1} / {2}",
    "rem-float" => [Result Register Register] "{1} % {2}",
    "add-double" => [Result Register Register] "{1} + {2}",
    "sub-double" => [Result Register Register] "{1} - {2}",
    "mul-double" => [Result Register Register] "{1} * {2}",
    "div-double" => [Result Register Register] "{1} / {2}",
    "rem-double" => [Result Register Register] "{1} % {2}",
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
    "add-int/lit16" => [Result Register Literal] "{1} + {2}",
    "rsub-int" => [Result Register Literal] "{2} - {1}",
    "mul-int/lit16" => [Result Register Literal] "{1} * {2}",
    "div-int/lit16" => [Result Register Literal] "{1} / {2}",
    "rem-int/lit16" => [Result Register Literal] "{1} % {2}",
    "and-int/lit16" => [Result Register Literal] "{1} & {2}",
    "or-int/lit16" => [Result Register Literal] "{1} | {2}",
    "xor-int/lit16" => [Result Register Literal] "{1} ^ {2}",
    "add-int/lit8" => [Result Register Literal] "{1} + {2}",
    "rsub-int/lit8" => [Result Register Literal] "{2} - {1}",
    "mul-int/lit8" => [Result Register Literal] "{1} * {2}",
    "div-int/lit8" => [Result Register Literal] "{1} / {2}",
    "rem-int/lit8" => [Result Register Literal] "{1} % {2}",
    "and-int/lit8" => [Result Register Literal] "{1} & {2}",
    "or-int/lit8" => [Result Register Literal] "{1} | {2}",
    "xor-int/lit8" => [Result Register Literal] "{1} ^ {2}",
    "shl-int/lit8" => [Result Register Literal] "{1} << {2}",
    "shr-int/lit8" => [Result Register Literal] "{1} >> {2}",
    "ushr-int/lit8" => [Result Register Literal] "{1} >>> {2}",
    "invoke-polymorphic" => [DefaultEmptyResult Registers Method Call] "invoke-polymorphic {1.this}.<{2}>({1.args}), <{3}>",
    "invoke-polymorphic/range" => [DefaultEmptyResult Registers Method Call] "invoke-polymorphic {1.this}.<{2}>({1.args}), <{3}>",
    // TODO: invoke-custom and invoke-custom/range
    "const-method-handle" => [Result MethodHandle] "{1}",
    "const-method-type" => [Result Call] "{1}",
);

#[derive(Debug, Clone, PartialEq)]
pub enum RawRegister {
    Parameter(usize),
    Local(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableRegister {
    This,
    Parameter(usize, Type),
    Local(usize, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    Raw(RawRegister),
    Variable(VariableRegister),
    Literal(Literal),
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Raw(RawRegister::Parameter(index)) => write!(f, "p{index}"),
            Self::Raw(RawRegister::Local(index)) => write!(f, "v{index}"),
            Self::Variable(VariableRegister::This) => write!(f, "@this"),
            Self::Variable(VariableRegister::Parameter(index, _)) => write!(f, "@p{index}"),
            Self::Variable(VariableRegister::Local(index, _)) => write!(f, "$v{index}"),
            Self::Literal(literal) => write!(f, "{literal}"),
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
        if let (
            Register::Raw(RawRegister::Parameter(from_index)),
            Register::Raw(RawRegister::Parameter(to_index)),
        ) = (from, to)
        {
            Some(
                (*from_index..to_index + 1)
                    .map(|index| Register::Raw(RawRegister::Parameter(index)))
                    .collect(),
            )
        } else if let (
            Register::Raw(RawRegister::Local(from_index)),
            Register::Raw(RawRegister::Local(to_index)),
        ) = (from, to)
        {
            Some(
                (*from_index..to_index + 1)
                    .map(|index| Register::Raw(RawRegister::Local(index)))
                    .collect(),
            )
        } else {
            eprintln!("Warning: Invalid parameter range: {from} .. {to}");
            None
        }
    }

    fn stringify_list(list: &[Register], split_first: bool) -> (Option<String>, String) {
        if split_first && !list.is_empty() {
            (
                Some(list[0].to_string()),
                list[1..].iter().map(Register::to_string).join(", "),
            )
        } else {
            (None, list.iter().map(Register::to_string).join(", "))
        }
    }

    pub fn to_list(&self, split_first: bool) -> (Option<String>, String) {
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
