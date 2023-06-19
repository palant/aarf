use std::io::Write;

use super::{Annotation, AnnotationParameter, AnnotationParameterValue, AnnotationVisibility};

impl AnnotationParameterValue {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        match self {
            Self::Literal(literal) => write!(output, "{literal}"),
            Self::Type(type_name) => write!(output, "{type_name}.class"),
            Self::Enum(type_name, constant) => write!(output, "{type_name}.{constant}"),
            Self::Array(array) => {
                write!(output, "{{")?;
                let mut first = true;
                for value in array {
                    if first {
                        first = false;
                    } else {
                        write!(output, ", ")?;
                    }
                    value.write_jimple(output)?;
                }
                write!(output, "}}")
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::error::ParseErrorDisplayed;
    use crate::tokenizer::Tokenizer;

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    fn normalize(data: &str) -> String {
        data.split(['\r', '\n'])
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .join(" ")
            .replace("( ", "(")
            .replace(" )", ")")
    }

    #[test]
    fn write_annotation() -> Result<(), ParseErrorDisplayed> {
        let mut input = tokenizer(r#"
            .annotation system Ldalvik/annotation/AnnotationDefault;
                value = .subannotation LAnnotationWithValues;
                            nullValue = null
                            booleanValue = false
                            byteValue = 1t
                            charValue = '2'
                            shortValue = 3s
                            intValue = 4
                            longValue = 5l
                            floatValue = 6.0f
                            doubleValue = 7.0
                            stringValue = "8"
                            subAnnotationValue = .subannotation LSubAnnotation;
                                                        stringValue = "9"
                                                .end subannotation
                            typeValue = L10;
                            methodValue = L10;->11()V
                            methodValue2 = Lj2/b;-><init>(Ljava/lang/String;II)V
                            enumValue = .enum LEnum;->12:LEnum;
                        .end subannotation
            .end annotation

            .annotation runtime Ldalvik/annotation/MemberClasses;
                value = {
                    Lj2/b$a;
                }
            .end annotation

            .annotation build Ldalvik/annotation/Signature;
                value = {
                    "Ljava/lang/Enum<",
                    "Lj2/b;",
                    ">;"
                }
            .end annotation

            .annotation runtime Ljava/lang/annotation/Target;
                value = {
                    .enum Ljava/lang/annotation/ElementType;->PACKAGE:Ljava/lang/annotation/ElementType;,
                    .enum Ljava/lang/annotation/ElementType;->TYPE:Ljava/lang/annotation/ElementType;,
                }
            .end annotation
        "#.trim());

        let expected = [
            r#"@dalvik.annotation.AnnotationDefault(
                value = @AnnotationWithValues(
                    nullValue = null,
                    booleanValue = false,
                    byteValue = 0x1,
                    charValue = '2',
                    shortValue = 0x3,
                    intValue = 0x4,
                    longValue = 0x5,
                    floatValue = 6,
                    doubleValue = 7,
                    stringValue = "8",
                    subAnnotationValue = @SubAnnotation(stringValue = "9"),
                    typeValue = 10.class,
                    methodValue = void 10.11(),
                    methodValue2 = void j2.b.<init>(java.lang.String, int, int),
                    enumValue = Enum.12
                )
            )"#,
            "@dalvik.annotation.MemberClasses(value = {j2.b$a.class})",
            r#"@dalvik.annotation.Signature(value = {"Ljava/lang/Enum<", "Lj2/b;", ">;"})"#,
            "@java.lang.annotation.Target(value = {java.lang.annotation.ElementType.PACKAGE, java.lang.annotation.ElementType.TYPE})",
        ];

        for expected_result in expected {
            input = input.expect_directive("annotation")?;

            let annotation;
            (input, annotation) = Annotation::read(&input, false)?;

            let mut cursor = std::io::Cursor::new(Vec::new());
            annotation.write_jimple(&mut cursor, -1).unwrap();

            assert_eq!(
                String::from_utf8_lossy(&cursor.into_inner()),
                normalize(expected_result)
            );
        }

        input.expect_eof()?;

        Ok(())
    }
}
