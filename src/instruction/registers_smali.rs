use super::Registers;
use crate::error::ParseError;
use crate::tokenizer::Tokenizer;

impl Registers {
    fn read_range(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, from) = input.read_keyword()?;
        let input = input.expect_char('.')?;
        let input = input.expect_char('.')?;
        let (input, to) = input.read_keyword()?;
        Ok((input, Self::Range(from, to)))
    }

    fn read_list(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let mut input = input.clone();
        let mut list = Vec::new();
        while let Ok((i, register)) = input.read_keyword() {
            input = i;
            list.push(register);
            if input.expect_char('}').is_ok() {
                break;
            }
            input = input.expect_char(',')?;
        }
        Ok((input, Self::List(list)))
    }

    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let input = input.expect_char('{')?;
        let (input, result) = if let Ok(result) = Self::read_range(&input) {
            result
        } else {
            Self::read_list(&input)?
        };
        let input = input.expect_char('}')?;
        Ok((input, result))
    }
}
