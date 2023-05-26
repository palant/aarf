use std::io::Write;

use super::AccessFlag;

impl AccessFlag {
    pub fn write_jimple_list(output: &mut dyn Write, list: &[Self]) -> Result<(), std::io::Error> {
        for entry in list {
            match entry {
                Self::Interface | Self::Annotation | Self::Enum | Self::Constructor => (),
                Self::Abstract => {
                    if !list.contains(&Self::Interface) {
                        write!(output, "{entry} ")?;
                    }
                }
                _ => write!(output, "{entry} ")?,
            }
        }
        Ok(())
    }
}
