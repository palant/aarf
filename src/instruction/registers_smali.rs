use super::{RawRegister, Register, Registers};
use crate::error::ParseError;
use crate::tokenizer::Tokenizer;

impl Register {
    pub fn read(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        if let Ok(input) = input.expect_char('p') {
            let (input, index) = input.read_number()?;
            Ok((input, Self::Raw(RawRegister::Parameter(index as usize))))
        } else if let Ok(input) = input.expect_char('v') {
            let (input, index) = input.read_number()?;
            Ok((input, Self::Raw(RawRegister::Local(index as usize))))
        } else {
            Err(input.unexpected("a register".into()))
        }
    }
}

impl Registers {
    fn read_range(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let (input, from) = Register::read(input)?;
        let input = input.expect_char('.')?;
        let input = input.expect_char('.')?;
        let (input, to) = Register::read(&input)?;
        Ok((input, Self::Range(from, to)))
    }

    fn read_list(input: &Tokenizer) -> Result<(Tokenizer, Self), ParseError> {
        let mut input = input.clone();
        let mut list = Vec::new();
        while let Ok((i, register)) = Register::read(&input) {
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
