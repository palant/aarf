use itertools::Itertools;
use std::fmt::{Display, Formatter};

use crate::literal::Literal;
use crate::r#type::{CallSignature, FieldSignature, MethodSignature, Type};

mod jimple;
mod optimization;
mod parameters_optimization;
mod parameters_smali;
mod registers_smali;
mod smali;

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

#[derive(Debug, PartialEq)]
pub enum CommandParameters {
    None,
    Result(Register),
    Register(Register),
    ResultRegister(Register, Register),
    RegisterRegister(Register, Register),
    ResultRegisterRegister(Register, Register, Register),
    RegisterRegisterRegister(Register, Register, Register),
    ResultLiteral(Register, Literal),
    ResultRegisterLiteral(Register, Register, Literal),
    ResultType(Register, Type),
    RegisterType(Register, Type),
    ResultRegisterType(Register, Register, Type),
    ResultRegistersType(Option<Register>, Registers, Type),
    ResultField(Register, FieldSignature),
    RegisterField(Register, FieldSignature),
    ResultRegisterField(Register, Register, FieldSignature),
    RegisterRegisterField(Register, Register, FieldSignature),
    ResultRegistersMethod(Option<Register>, Registers, MethodSignature),
    ResultRegistersMethodCall(Option<Register>, Registers, MethodSignature, CallSignature),
    Label(String),
    RegisterLabel(Register, String),
    RegisterData(Register, CommandData),
    RegisterRegisterLabel(Register, Register, String),
    ResultCall(Register, CallSignature),
    ResultMethodHandle(Register, String, MethodSignature),
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    LineNumber(i64, i64),
    Label(String),
    Command {
        command: String,
        parameters: CommandParameters,
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
