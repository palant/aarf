use std::io::Write;

use super::{Annotation, AnnotationParameter, AnnotationParameterValue, AnnotationVisibility};

impl AnnotationParameterValue {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        match self {
            Self::Literal(literal) => write!(output, "{literal}"),
            Self::Type(type_name) => write!(output, "class {type_name}"),
            Self::Method(signature) => write!(output, "public static {signature}"),
            Self::Enum(type_name, constant) => write!(output, "{type_name}.{constant}"),
            Self::Array(array) => {
                write!(output, "[")?;
                let mut first = true;
                for value in array {
                    if first {
                        first = false;
                    } else {
                        write!(output, ", ")?;
                    }
                    value.write_jimple(output)?;
                }
                write!(output, "]")
            }
            Self::SubAnnotation(annotation) => annotation.write_jimple(output, -1),
        }
    }
}

impl AnnotationParameter {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        write!(output, "{} = ", self.name)?;
        self.value.write_jimple(output)
    }
}

impl Annotation {
    pub fn write_jimple(
        &self,
        output: &mut dyn Write,
        indent_level: i32,
    ) -> Result<(), std::io::Error> {
        if indent_level >= 0 {
            for _ in 0..indent_level {
                write!(output, "    ")?;
            }
        }

        write!(output, "@{}(", self.annotation_type)?;

        let mut first = true;
        for parameter in &self.parameters {
            if first {
                first = false;
            } else {
                write!(output, ", ")?;
            }
            parameter.write_jimple(output)?;
        }

        write!(output, ")")?;
        if indent_level >= 0 {
            match self.visibility {
                AnnotationVisibility::Build => write!(output, " // build")?,
                AnnotationVisibility::System => write!(output, " // system")?,
                AnnotationVisibility::Runtime => (),
            };

            writeln!(output)?;
        }

        Ok(())
    }
}
