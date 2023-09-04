use std::collections::HashMap;

use super::{
    CommandData, CommandParameter, Instruction, Register, ResultType, ResultTypeDef, DEFS,
};
use crate::literal::Literal;
use crate::r#type::{MethodSignature, Type};

impl Instruction {
    pub fn get_moved_result(&self) -> Option<Register> {
        if let Self::Command {
            command,
            parameters,
        } = self
        {
            if DEFS
                .get(command)
                .map(|d| d.is_moved_result)
                .unwrap_or(false)
            {
                if let Some(CommandParameter::Result(result)) = parameters.get(0) {
                    return Some(result.clone());
                }
            }
        }
        None
    }

    pub fn inline_result(&mut self, r: Register) -> bool {
        if let Self::Command { parameters, .. } = self {
            if let Some(CommandParameter::DefaultEmptyResult(result)) = parameters.get_mut(0) {
                if result.is_none() {
                    *result = Some(r);
                    return true;
                }
            }
        }

        false
    }

    pub fn resolve_data(&mut self, d: &HashMap<String, CommandData>) {
        if let Self::Command { parameters, .. } = self {
            for parameter in parameters.iter_mut() {
                if let CommandParameter::Data(data) = parameter {
                    if let CommandData::Label(label) = data {
                        if let Some(d) = d.get(label) {
                            *data = d.clone();
                        } else {
                            eprintln!("Warning: Failed resolving command data {label}");
                        }
                    }
                }
            }
        }
    }

    pub fn fix_check_cast(&mut self) {
        if let Self::Command {
            command,
            parameters,
        } = self
        {
            if command != "check-cast" {
                return;
            }
            if let Some(
                [CommandParameter::DefaultEmptyResult(None), CommandParameter::Register(register)],
            ) = parameters.get(0..2)
            {
                parameters[0] = CommandParameter::DefaultEmptyResult(Some(register.clone()));
            }
        }
    }

    fn parameter_type(
        parameter: &CommandParameter,
        state: &HashMap<Register, ResultType>,
    ) -> Option<ResultType> {
        match parameter {
            CommandParameter::Result(register)
            | CommandParameter::DefaultEmptyResult(Some(register))
            | CommandParameter::Register(register) => match state.get(register) {
                Some(r#type) => Some(r#type.clone()),
                None => {
                    eprintln!("Warning: Using register {register}, yet its type isn't known yet.");
                    None
                }
            },
            CommandParameter::DefaultEmptyResult(None) => None,
            CommandParameter::Literal(literal) => Some(literal.into()),
            CommandParameter::Type(r#type) => Some(r#type.into()),
            CommandParameter::Field(field) => Some((&field.field_type).into()),
            CommandParameter::Method(method) => Some((&method.call_signature.return_type).into()),
            CommandParameter::CallSite(call_site) => {
                Some((&call_site.method.call_signature.return_type).into())
            }
            CommandParameter::Variable(_)
            | CommandParameter::Registers(_)
            | CommandParameter::Label(_)
            | CommandParameter::Data(_) => {
                eprintln!(
                    "Warning: Trying to deduce type from unexpected parameter {parameter:?}."
                );
                None
            }
        }
    }

    pub fn get_result_type(&self, state: &HashMap<Register, ResultType>) -> Option<ResultType> {
        if let Self::Command {
            command,
            parameters,
        } = self
        {
            match DEFS
                .get(command)
                .map(|d| &d.result_type)
                .unwrap_or(&ResultTypeDef::None)
            {
                ResultTypeDef::None => None,
                ResultTypeDef::Bool => Some(Type::Bool.into()),
                ResultTypeDef::Byte => Some(Type::Byte.into()),
                ResultTypeDef::Char => Some(Type::Char.into()),
                ResultTypeDef::Short => Some(Type::Short.into()),
                ResultTypeDef::Int => Some(Type::Int.into()),
                ResultTypeDef::Long => Some(Type::Long.into()),
                ResultTypeDef::Float => Some(Type::Float.into()),
                ResultTypeDef::Double => Some(Type::Double.into()),
                ResultTypeDef::Object(class) => Some(Type::Object(class.to_string()).into()),
                ResultTypeDef::From(index) => Self::parameter_type(&parameters[*index], state),
                ResultTypeDef::ElementFrom(index) => {
                    match Self::parameter_type(&parameters[*index], state) {
                        None => None,
                        Some(ResultType::Type(Type::Array(element))) => Some((*element).into()),
                        other => {
                            eprintln!("Warning: Trying to deduce element type from non-array parameter {other:?}");
                            None
                        }
                    }
                }
                ResultTypeDef::ReturnOf(index) => {
                    match Self::parameter_type(&parameters[*index], state) {
                        None => None,
                        Some(ResultType::Literal(Literal::Method(MethodSignature {
                            call_signature,
                            ..
                        })))
                        | Some(ResultType::Literal(Literal::MethodHandle(
                            _,
                            MethodSignature { call_signature, .. },
                        )))
                        | Some(ResultType::Literal(Literal::MethodType(call_signature))) => {
                            Some((&call_signature.return_type).into())
                        }
                        other => {
                            eprintln!("Warning: Trying to deduce return type from a non-call parameter {other:?}");
                            None
                        }
                    }
                }
                ResultTypeDef::Exception => {
                    Some(Type::Object("java.lang.exception".to_string()).into())
                }
            }
        } else {
            None
        }
    }

    pub fn get_jump_target(&self) -> Option<String> {
        if let Self::Command { parameters, .. } = self {
            for parameter in parameters {
                if let CommandParameter::Label(label) = parameter {
                    return Some(label.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorDisplayed;
    use crate::literal::Literal;
    use crate::r#type::{CallSignature, MethodSignature};
    use crate::tokenizer::Tokenizer;

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    #[test]
    fn get_result_type() -> Result<(), ParseErrorDisplayed> {
        let mut state = HashMap::new();
        state.insert(Register::Local(5), ResultType::Type(Type::Double));
        state.insert(
            Register::Local(2),
            ResultType::Type(Type::Array(Box::new(Type::Object(
                "java.lang.String".to_string(),
            )))),
        );

        let mut input = tokenizer(r#"
            return v0
            const/16 v7, 0x3f
            check-cast p0, Lj2/b;
            move-wide v11, v5
            const-class v0, Lhd/e;
            new-array v1, v1, [I
            aget-object v5, v2, v4
            iget-boolean p2, p0, Lhd/c;->x:Z
            invoke-direct {v16, v17}, Ls1/b$a;-><init>(Lkotlin/jvm/internal/DefaultConstructorMarker;)Ljava/lang/String;
            const-method-handle v0, invoke-static@Ljava/lang/Integer;->toString(I)Ljava/lang/String;
            const-method-type v0, (II)I
            invoke-polymorphic {p1, v0, v1}, Ljava/lang/invoke/MethodHandle;->invoke([Ljava/lang/Object;)Ljava/lang/Object;, (II)V
            invoke-custom/range {p0 .. p1}, backwardsLinkedCallSite("doSomething", (LCustom;Ljava/lang/String;)Ljava/lang/String;, "just testing")@LBootstrapLinker;->backwardsLink(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/String;)Ljava/lang/invoke/CallSite;
        "#.trim());

        let expected = [
            None,
            Some(ResultType::Literal(Literal::Int(0x3f))),
            Some(ResultType::Type(Type::Object("j2.b".to_string()))),
            Some(ResultType::Type(Type::Double)),
            Some(ResultType::Literal(Literal::Class(Type::Object(
                "hd.e".to_string(),
            )))),
            Some(ResultType::Type(Type::Array(Box::new(Type::Int)))),
            Some(ResultType::Type(Type::Object(
                "java.lang.String".to_string(),
            ))),
            Some(ResultType::Type(Type::Bool)),
            Some(ResultType::Type(Type::Object(
                "java.lang.String".to_string(),
            ))),
            Some(ResultType::Literal(Literal::MethodHandle(
                "invoke-static".to_string(),
                MethodSignature {
                    object_type: Type::Object("java.lang.Integer".to_string()),
                    method_name: "toString".to_string(),
                    call_signature: CallSignature {
                        parameter_types: vec![Type::Int],
                        return_type: Type::Object("java.lang.String".to_string()),
                    },
                },
            ))),
            Some(ResultType::Literal(Literal::MethodType(CallSignature {
                parameter_types: vec![Type::Int, Type::Int],
                return_type: Type::Int,
            }))),
            Some(ResultType::Type(Type::Void)),
            Some(ResultType::Type(Type::Object(
                "java.lang.Object".to_string(),
            ))),
        ];

        for expected_result_type in expected {
            let instruction;
            (input, instruction) = Instruction::read(&input)?;
            assert_eq!(instruction.get_result_type(&state), expected_result_type);
        }

        input.expect_eof()?;

        Ok(())
    }

    #[test]
    fn get_jump_target() -> Result<(), ParseErrorDisplayed> {
        let mut input = tokenizer(
            r#"
            return v0
            goto/16 :goto_5
            if-eqz v2, :cond_0
        "#
            .trim(),
        );

        let expected = [None, Some("goto_5".to_string()), Some("cond_0".to_string())];

        for expected_label in expected {
            let instruction;
            (input, instruction) = Instruction::read(&input)?;
            assert_eq!(instruction.get_jump_target(), expected_label);
        }

        input.expect_eof()?;

        Ok(())
    }
}
