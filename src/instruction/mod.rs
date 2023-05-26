use crate::literal::Literal;
use crate::r#type::{CallSignature, FieldSignature, MethodSignature, Type};

mod jimple;
mod optimization;
mod parameters_optimization;
mod parameters_smali;
mod registers_smali;
mod smali;

#[derive(Debug, Clone, PartialEq)]
pub enum Registers {
    List(Vec<String>),
    Range(String, String),
}

impl Registers {
    fn resolve_range(from: &str, to: &str) -> Option<(char, usize, usize)> {
        let first1 = from.chars().next()?;
        let first2 = to.chars().next()?;
        if first1 != first2 {
            eprintln!("Warning: Invalid parameter range: {from} .. {to}");
            return None;
        }

        let num1: usize = from[1..].parse().ok()?;
        let num2: usize = to[1..].parse().ok()?;
        if num1 > num2 {
            eprintln!("Warning: Invalid parameter range: {from} .. {to}");
            return None;
        }

        Some((first1, num1, num2))
    }

    fn stringify_list(list: &[String], split_first: bool) -> (Option<String>, String) {
        if split_first && !list.is_empty() {
            (Some(list[0].to_string()), list[1..].join(", "))
        } else {
            (None, list.join(", "))
        }
    }

    pub fn to_list(&self, split_first: bool) -> (Option<String>, String) {
        match self {
            Self::List(list) => Self::stringify_list(list, split_first),
            Self::Range(from, to) => {
                if let Some((prefix, from, to)) = Self::resolve_range(from, to) {
                    let list = (from..to + 1)
                        .map(|i| format!("{prefix}{i}"))
                        .collect::<Vec<_>>();
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
    Result(String),
    Register(String),
    ResultRegister(String, String),
    RegisterRegister(String, String),
    ResultRegisterRegister(String, String, String),
    RegisterRegisterRegister(String, String, String),
    ResultLiteral(String, Literal),
    ResultRegisterLiteral(String, String, Literal),
    ResultType(String, Type),
    RegisterType(String, Type),
    ResultRegisterType(String, String, Type),
    ResultRegistersType(Option<String>, Registers, Type),
    ResultField(String, FieldSignature),
    RegisterField(String, FieldSignature),
    ResultRegisterField(String, String, FieldSignature),
    RegisterRegisterField(String, String, FieldSignature),
    ResultRegistersMethod(Option<String>, Registers, MethodSignature),
    ResultRegistersMethodCall(Option<String>, Registers, MethodSignature, CallSignature),
    Label(String),
    RegisterLabel(String, String),
    RegisterData(String, CommandData),
    RegisterRegisterLabel(String, String, String),
    ResultCall(String, CallSignature),
    ResultMethodHandle(String, String, MethodSignature),
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
