use std::collections::HashMap;

use super::{CommandData, CommandParameter, Instruction, Register};

impl Instruction {
    pub fn get_moved_result(&self) -> Option<Register> {
        if let Self::Command {
            command,
            parameters,
        } = self
        {
            if command.starts_with("move-result") {
                if let Some(CommandParameter::Result(result)) = parameters.get(0) {
                    return Some(result.clone());
                }
            }
        }
        None
    }

    pub fn inline_result(&mut self, r: Register) -> bool {
        if let Self::Command { parameters, .. } = self {
            if let Some(CommandParameter::DefaultEmptyResult(result)) = parameters.get_mut(0) {
                if result.is_none() {
                    *result = Some(r);
                    return true;
                }
            }
        }

        false
    }

    pub fn resolve_data(&mut self, d: &HashMap<String, CommandData>) {
        if let Self::Command { parameters, .. } = self {
            for parameter in parameters.iter_mut() {
                if let CommandParameter::Data(data) = parameter {
                    if let CommandData::Label(label) = data {
                        if let Some(d) = d.get(label) {
                            *data = d.clone();
                        } else {
                            eprintln!("Warning: Failed resolving command data {label}");
                        }
                    }
                }
            }
        }
    }
}
