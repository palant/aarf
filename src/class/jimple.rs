use std::io::Write;

use super::Class;
use crate::access_flag::AccessFlag;
use crate::r#type::Type;

impl Class {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        if let Some(source_file) = &self.source_file {
            writeln!(output, "// source: {}", &source_file)?;
        }

        for annotation in &self.annotations {
            annotation.write_jimple(output, 0)?;
        }

        AccessFlag::write_jimple_list(output, &self.access_flags)?;

        write!(
            output,
            "{} {}",
            if self.access_flags.contains(&AccessFlag::Interface) {
                "interface"
            } else if self.access_flags.contains(&AccessFlag::Annotation) {
                "@interface"
            } else if self.access_flags.contains(&AccessFlag::Enum) {
                "enum"
            } else {
                "class"
            },
            self.class_type
        )?;

        if let Some(super_class) = &self.super_class {
            write!(output, " extends {super_class}")?;
        }

        if !self.interfaces.is_empty() {
            let implements = self
                .interfaces
                .iter()
                .map(Type::get_name)
                .collect::<Vec<_>>();
            write!(output, " implements {}", implements.join(", "))?;
        }
        writeln!(output)?;
        writeln!(output, "{{")?;

        let mut first = true;
        for field in &self.fields {
            if first {
                first = false;
            } else {
                writeln!(output)?;
            }
            field.write_jimple(output)?;
        }

        for method in &self.methods {
            if first {
                first = false;
            } else {
                writeln!(output)?;
            }
            method.write_jimple(output)?;
        }

        writeln!(output, "}}")?;
        Ok(())
    }
}
