use super::Class;
use crate::access_flag::AccessFlag;
use crate::annotation::Annotation;
use crate::error::ParseError;
use crate::field::Field;
use crate::literal::Literal;
use crate::method::Method;
use crate::r#type::Type;
use crate::tokenizer::Tokenizer;

impl Class {
    fn read_super_class(input: &Tokenizer) -> Result<(Tokenizer, Option<Type>), ParseError> {
        let (input, super_class) = Type::read(input)?;
        let input = input.expect_eol()?;
        Ok((
            input,
            if matches!(&super_class, Type::Object(name) if name == "java.lang.Object" || name == "java.lang.Enum")
            {
                None
            } else {
                Some(super_class)
            },
        ))
    }

    fn read_interface(input: &Tokenizer) -> Result<(Tokenizer, Type), ParseError> {
        let (input, interface) = Type::read(input)?;
        let input = input.expect_eol()?;
        Ok((input, interface))
    }

    fn read_source_file(input: &Tokenizer) -> Result<(Tokenizer, String), ParseError> {
        let start = input;
        let (input, literal) = Literal::read(input)?;
        let source = literal
            .get_string()
            .ok_or_else(|| start.unexpected("a string literal".into()))?;
        let input = input.expect_eol()?;
        Ok((input, source))
    }

    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let input = input.expect_directive("class")?;
        let (input, access_flags) = AccessFlag::read_list(&input);
        let (input, class_type) = Type::read(&input)?;
        let input = input.expect_eol()?;

        let mut input = input;
        let mut super_class = None;
        let mut interfaces = Vec::new();
        let mut source_file = None;
        let mut annotations = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while input.expect_eof().is_err() {
            let (i, directive) = input.read_directive()?;
            let start = input;
            input = i;

            match directive.as_str() {
                "super" => {
                    (input, super_class) = Self::read_super_class(&input)?;
                }
                "implements" => {
                    let interface;
                    (input, interface) = Self::read_interface(&input)?;
                    interfaces.push(interface);
                }
                "source" => {
                    let file_name;
                    (input, file_name) = Self::read_source_file(&input)?;
                    source_file = Some(file_name);
                }
                "annotation" => {
                    let annotation;
                    (input, annotation) = Annotation::read(&input, false)?;
                    annotations.push(annotation);
                }
                "field" => {
                    let field;
                    (input, field) = Field::read(&input)?;
                    fields.push(field);
                }
                "method" => {
                    let method;
                    (input, method) = Method::read(&input)?;
                    methods.push(method);
                }
                _ => return Err(start.unexpected("a supported directive".into())),
            };
        }

        Ok((
            input,
            Self {
                class_type,
                access_flags,
                super_class,
                interfaces,
                source_file,
                annotations,
                fields,
                methods,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorDisplayed;

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn read_super_class() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" .super  Labc/def;  \n");
        let input = input.expect_directive("super")?;
        assert!(matches!(
            Class::read_super_class(&input),
            Ok((input, Some(Type::Object(name)))) if name == "abc.def" && input.expect_eof().is_ok()
        ));

        let input = tokenizer("  .super Ljava/lang/Object;\nwhatever");
        let input = input.expect_directive("super")?;
        assert!(matches!(
            Class::read_super_class(&input),
            Ok((input, None)) if input.read_keyword().map(|(_, k)| k).unwrap_or(String::new()) == "whatever"
        ));

        let input = tokenizer("  .super x Ljava/lang/Object;\n");
        let input = input.expect_directive("super")?;
        assert!(Class::read_super_class(&input).is_err());

        let input = tokenizer("  .super \n Ljava/lang/Object;\n");
        let input = input.expect_directive("super")?;
        assert!(Class::read_super_class(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_interface() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" .implements  Labc/def;  \n");
        let input = input.expect_directive("implements")?;
        assert!(matches!(
            Class::read_interface(&input),
            Ok((input, Type::Object(name))) if name == "abc.def" && input.expect_eof().is_ok()
        ));

        let input = tokenizer("  .implements Ljava/lang/Object;\nwhatever");
        let input = input.expect_directive("implements")?;
        assert!(matches!(
            Class::read_interface(&input),
            Ok((input, Type::Object(name))) if name == "java.lang.Object" &&
                input.read_keyword().map(|(_, k)| k).unwrap_or(String::new()) == "whatever"
        ));

        let input = tokenizer("  .implements x Ljava/lang/Object;\n");
        let input = input.expect_directive("implements")?;
        assert!(Class::read_interface(&input).is_err());

        let input = tokenizer("  .implements \n Ljava/lang/Object;\n");
        let input = input.expect_directive("implements")?;
        assert!(Class::read_interface(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_source_file() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(" .source \"File.java\"\n");
        let input = input.expect_directive("source")?;
        assert!(matches!(
            Class::read_source_file(&input),
            Ok((input, name)) if name == "File.java" && input.expect_eof().is_ok()
        ));

        let input = tokenizer(" .source \"File\\\".java\\\\\"\nwhatever");
        let input = input.expect_directive("source")?;
        assert!(matches!(
            Class::read_source_file(&input),
            Ok((input, name)) if name == "File\\\".java\\\\" && input.expect_eof().is_err()
        ));

        let input = tokenizer(" .source \"File.java\\\"\nwhatever");
        let input = input.expect_directive("source")?;
        assert!(Class::read_source_file(&input).is_err());

        Ok(())
    }
}
