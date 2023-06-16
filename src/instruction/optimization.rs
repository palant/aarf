use std::collections::HashMap;

use super::{CommandData, CommandParameters, Instruction, Register};

impl Instruction {
    pub fn get_moved_result(&self) -> Option<Register> {
        if let Self::Command {
            command,
            parameters: CommandParameters::Result(result),
        } = self
        {
            if command.starts_with("move-result") {
                return Some(result.clone());
            }
        }
        None
    }

    pub fn inline_result(&mut self, result: Register) -> bool {
        if let Self::Command { parameters, .. } = self {
            parameters.inline_result(result)
        } else {
            false
        }
    }

    pub fn resolve_data(&mut self, data: &HashMap<String, CommandData>) {
        if let Self::Command { parameters, .. } = self {
            parameters.resolve_data(data);
        }
    }
}
