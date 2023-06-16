use std::collections::HashMap;

use super::{CommandData, CommandParameters, Register};

impl CommandParameters {
    pub fn inline_result(&mut self, r: Register) -> bool {
        match self {
            Self::ResultRegistersMethod(result, ..)
            | Self::ResultRegistersMethodCall(result, ..)
            | Self::ResultRegistersType(result, ..) => {
                if result.is_some() {
                    false
                } else {
                    *result = Some(r);
                    true
                }
            }
            _ => false,
        }
    }

    pub fn resolve_data(&mut self, d: &HashMap<String, CommandData>) {
        if let Self::RegisterData(_, data) = self {
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
