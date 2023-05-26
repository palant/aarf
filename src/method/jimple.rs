use std::io::Write;

use super::Method;
use crate::access_flag::AccessFlag;
use crate::instruction::Instruction;

impl Method {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        for annotation in &self.annotations {
            annotation.write_jimple(output, 1)?;
        }

        write!(output, "    ")?;
        AccessFlag::write_jimple_list(output, &self.visibility)?;
        write!(output, "{} {}(", self.return_type, self.name)?;

        let mut first = true;
        for (i, parameter) in self.parameters.iter().enumerate() {
            if first {
                first = false;
            } else {
                write!(output, ", ")?;
            }

            for annotation in &parameter.annotations {
                annotation.write_jimple(output, -1)?;
                write!(output, " ")?;
            }

            write!(output, "{} @p{i}", parameter.parameter_type)?;
        }
        writeln!(output, ")")?;
        writeln!(output, "    {{")?;

        let mut had_delimiter = true;
        for instruction in &self.instructions {
            if matches!(instruction, Instruction::Command { .. }) {
                had_delimiter = false;
            } else if !had_delimiter {
                writeln!(output)?;
                had_delimiter = true;
            }
            instruction.write_jimple(output)?;
        }

        writeln!(output, "    }}")?;

        Ok(())
    }
}
