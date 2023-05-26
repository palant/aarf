use super::AccessFlag;

use crate::error::ParseError;
use crate::tokenizer::Tokenizer;

impl AccessFlag {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let start = input;
        let (input, keyword) = input.read_keyword()?;
        let access_flag = Self::try_from(keyword.as_str())
            .map_err(|_| start.unexpected("an access flag".into()))?;
        if input
            .next_char()
            .filter(|&c| c == ' ' || c == '\t')
            .is_some()
        {
            Ok((input, access_flag))
        } else {
            Err(input.unexpected("a space".into()))
        }
    }

    pub fn read_list(input: &Tokenizer) -> (Tokenizer, Vec<Self>) {
        let mut input = input.clone();
        let mut result = Vec::new();
        while let Ok((i, access_flag)) = Self::read(&input) {
            input = i;
            result.push(access_flag);
        }
        (input, result)
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
    fn read_access_flag() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer("public static final");

        let (input, access_flag) = AccessFlag::read(&input)?;
        assert_eq!(access_flag, AccessFlag::Public);

        let (input, access_flag) = AccessFlag::read(&input)?;
        assert_eq!(access_flag, AccessFlag::Static);

        assert!(AccessFlag::read(&input).is_err());

        Ok(())
    }

    #[test]
    fn read_access_flag_list() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer("public static final final:I");
        let (input, access_flags) = AccessFlag::read_list(&input);
        assert_eq!(
            access_flags,
            vec![AccessFlag::Public, AccessFlag::Static, AccessFlag::Final,]
        );

        let (_, keyword) = input.read_keyword()?;
        assert_eq!(keyword, "final");

        Ok(())
    }
}
