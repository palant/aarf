use std::io::Write;

use super::Field;
use crate::access_flag::AccessFlag;

impl Field {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        for annotation in &self.annotations {
            annotation.write_jimple(output, 1)?;
        }

        write!(output, "    ")?;
        AccessFlag::write_jimple_list(output, &self.visibility)?;
        write!(output, "{} {}", self.field_type, self.name)?;

        if let Some(initial_value) = &self.initial_value {
            write!(output, " = {}", initial_value)?;
        }
        writeln!(output, ";")?;

        Ok(())
    }
}
