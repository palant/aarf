use std::io::Write;

use super::{CommandData, CommandParameter, Instruction, DEFS};

fn stringify_parameter(parameter: &CommandParameter) -> String {
    match parameter {
        CommandParameter::Result(register)
        | CommandParameter::DefaultEmptyResult(Some(register))
        | CommandParameter::Register(register) => register.to_string(),
        CommandParameter::DefaultEmptyResult(None) => String::new(),
        CommandParameter::Variable(variable) => variable.to_string(),
        CommandParameter::Registers(registers) => registers.to_list(false).1,
        CommandParameter::Literal(literal) => literal.to_string(),
        CommandParameter::Label(label) => label.clone(),
        CommandParameter::Type(r#type) => r#type.to_string(),
        CommandParameter::Field(field) => field.to_string(),
        CommandParameter::Method(method) => method.to_string(),
        CommandParameter::MethodHandle(invoke_type, method) => format!("{invoke_type}@{method}"),
        CommandParameter::Call(call) => call.to_string(),
        CommandParameter::Data(CommandData::Label(label)) => {
            eprintln!("Warning: Writing out unresolved command data label {label}");
            "??<label>??".to_string()
        }
        CommandParameter::Data(CommandData::PackedSwitch(first_key, targets)) => targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let key = first_key + (index as i64);
                format!(
                    "            case {}{:#x}: goto {target};\n",
                    if key.is_negative() { "-" } else { "" },
                    key.abs_diff(0)
                )
            })
            .collect(),
        CommandParameter::Data(CommandData::SparseSwitch(targets)) => targets
            .iter()
            .map(|(value, target)| format!("            case {value}: goto {target};\n"))
            .collect(),
        CommandParameter::Data(CommandData::Array(values)) => values
            .iter()
            .map(|value| format!("            {value},\n"))
            .collect(),
    }
}

impl Instruction {
    pub fn write_jimple(&self, output: &mut dyn Write) -> Result<(), std::io::Error> {
        match self {
            Self::LineNumber(from, to) => {
                if from == to {
                    writeln!(output, "        // line {from}")
                } else {
                    writeln!(output, "        // line {from}-{to}")
                }
            }
            Self::Label(label) => writeln!(output, "    {label}:"),
            Self::Command {
                command,
                parameters,
            } => {
                let defs = DEFS.get(command).ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Attempt to write unknown command to Jimple",
                    )
                })?;

                write!(output, "        ")?;
                if let Some(CommandParameter::Result(result))
                | Some(CommandParameter::DefaultEmptyResult(Some(result))) = parameters.get(0)
                {
                    write!(output, "{} = ", result)?;
                }

                let mut result = defs.format.to_string();
                for (index, parameter) in parameters.iter().enumerate() {
                    let placeholder = format!("{{{index}}}");
                    if result.contains(&placeholder) {
                        result = result.replace(&placeholder, &stringify_parameter(parameter));
                    }

                    if let CommandParameter::Registers(registers) = parameter {
                        let placeholder1 = format!("{{{index}.this}}");
                        let placeholder2 = format!("{{{index}.args}}");
                        if result.contains(&placeholder1) || result.contains(&placeholder2) {
                            let (this, args) = registers.to_list(true);
                            let this = this.unwrap_or_else(|| "???".to_string());
                            result = result.replace(&placeholder1, &this);
                            result = result.replace(&placeholder2, &args);
                        }
                    }
                }
                writeln!(output, "{};", result)
            }
            Self::Catch {
                exception,
                start_label,
                end_label,
                target,
            } => writeln!(
                output,
                "        catch {} from {start_label} to {end_label} with {target};",
                exception
                    .as_ref()
                    .map(|t| format!("{}", t))
                    .unwrap_or_else(|| "java.lang.Throwable".to_string())
            ),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ParseErrorDisplayed;
    use crate::tokenizer::Tokenizer;

    fn tokenizer(data: &str) -> Tokenizer {
        Tokenizer::new(data.to_string(), std::path::Path::new("dummy"))
    }

    fn stringify(instruction: Instruction) -> String {
        let mut cursor = std::io::Cursor::new(Vec::new());
        instruction.write_jimple(&mut cursor).unwrap();
        String::from_utf8_lossy(&cursor.into_inner())
            .trim()
            .to_string()
    }

    #[test]
    fn write_instruction() -> Result<(), ParseErrorDisplayed> {
        let mut input = tokenizer(r#"
            return-void
            :cond_0
            move-object v0, p2
            neg-int v1, p0
            int-to-float v1, v2
            array-length v0, v1
            return-wide v1
            monitor-enter v2
            check-cast p0, Lj2/b;
            .line 6
            const/16 v3, 0x400
            const-class v4, Ljava/lang/String;
            new-instance v5, Ljava/lang/String;
            if-gtz v6, :cond_0
            goto/16 :goto_1
            aget-object v7, p1, v3
            sub-int v8, p3, p2
            if-ge v9, v10, :cond_1
            aput-boolean v11, v3, p1
            iget-object v12, p0, Lo2/h;->a:Landroid/text/Layout;
            iput-wide v13, p0, Lt4/o;->x:J
            sget v14, Ln8/h;->h0:I
            sput-wide v15, Ls1/b;->b:J
            invoke-direct {v16, v17}, Ls1/b$a;-><init>(Lkotlin/jvm/internal/DefaultConstructorMarker;)V
            invoke-static {v18, v19}, Ls1/b;->d(J)J
            invoke-virtual/range {p2 .. p7}, Ls2/t0;->a(Ls2/n;Ls2/c0;IILjava/lang/Object;)Ls2/t0;
            and-long/2addr v20, v21
            shl-int/lit8 v22, p1, 0x3
            rsub-int v23, v24, 0x800
            invoke-polymorphic {p1, v25, v26}, Ljava/lang/invoke/MethodHandle;->invoke([Ljava/lang/Object;)Ljava/lang/Object;, (II)V
        "#.trim());

        let mut expected = r#"
            return;
            cond_0:
            v0 = p2;
            v1 = -p0;
            v1 = (float) v2;
            v0 = array-length v1;
            return v1;
            monitor-enter v2;
            p0 = (j2.b) p0;
            // line 6
            v3 = 0x400;
            v4 = class java.lang.String;
            v5 = new java.lang.String;
            if (v6 > 0) goto cond_0;
            goto goto_1;
            v7 = p1[v3];
            v8 = p3 - p2;
            if (v9 >= v10) goto cond_1;
            v3[p1] = v11;
            v12 = p0.<android.text.Layout o2.h.a>;
            p0.<long t4.o.x> = v13;
            v14 = <int n8.h.h0>;
            <long s1.b.b> = v15;
            invoke-direct v16.<void s1.b$a.<init>(kotlin.jvm.internal.DefaultConstructorMarker)>(v17);
            invoke-static <long s1.b.d(long)>(v18, v19);
            invoke-virtual p2.<s2.t0 s2.t0.a(s2.n, s2.c0, int, int, java.lang.Object)>(p3, p4, p5, p6, p7);
            v20 &= v21;
            v22 = p1 << 0x3;
            v23 = 0x800 - v24;
            invoke-polymorphic p1.<java.lang.Object java.lang.invoke.MethodHandle.invoke(java.lang.Object[])>(v25, v26), <void (int, int)>;
        "#.trim().split('\n').map(|s| s.trim().to_string()).collect::<Vec<_>>();

        while let Ok((i, instruction)) = Instruction::read(&input) {
            assert_eq!(stringify(instruction), expected.remove(0));
            input = i;
        }

        assert!(expected.is_empty());
        assert!(input.expect_eof().is_ok());

        Ok(())
    }
}
