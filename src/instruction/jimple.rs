use std::io::Write;

use super::{CommandData, CommandParameters, Instruction, Register, Registers};
use crate::r#type::MethodSignature;

fn if_op(command: &str) -> &str {
    let command = command.trim_end_matches('z');
    match command {
        "if-eq" => "==",
        "if-ne" => "!=",
        "if-lt" => "<",
        "if-gt" => ">",
        "if-le" => "<=",
        "if-ge" => ">=",
        _ => "",
    }
}

fn unary_op(command: &str) -> &str {
    let command = if let Some((_, t)) = command.split_once("-to-") {
        t
    } else {
        command.split('-').next().unwrap_or(command)
    };
    match command {
        "neg" => "-",
        "not" => "~",
        "byte" => "(byte) ",
        "char" => "(char) ",
        "short" => "(short) ",
        "int" => "(int) ",
        "long" => "(long) ",
        "float" => "(float) ",
        "double" => "(double) ",
        _ => "",
    }
}

fn bin_op(command: &str) -> &str {
    let command = command.split('-').next().unwrap_or(command);
    match command {
        "add" => "+",
        "sub" | "rsub" => "-",
        "mul" => "*",
        "div" => "/",
        "rem" => "%",
        "and" => "&",
        "or" => "|",
        "xor" => "^",
        "shl" => "<<",
        "shr" => ">>",
        "ushr" => ">>>",
        _ => "",
    }
}

fn stringify_call(
    command: &str,
    result: &Option<Register>,
    method: &MethodSignature,
    registers: &Registers,
) -> String {
    let is_static = command.starts_with("invoke-static");
    let (this, args) = registers.to_list(!is_static);
    let is_static = command.starts_with("invoke-static");

    let prefix = if let Some(result) = result {
        format!("{result} = ")
    } else {
        String::new()
    };

    if let Some(this) = this {
        format!(
            "{prefix}{} {this}.<{method}>({args})",
            command.strip_suffix("/range").unwrap_or(command),
        )
    } else {
        if !is_static {
            eprintln!("Warning: non-static call <{method}> has zero parameters");
        }

        format!(
            "{prefix}{} <{method}>({args})",
            command.strip_suffix("/range").unwrap_or(command),
        )
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
            } => match parameters {
                CommandParameters::None => {
                    writeln!(
                        output,
                        "        {};",
                        if command == "return-void" {
                            "return"
                        } else {
                            command
                        }
                    )
                }
                CommandParameters::Result(result) => {
                    writeln!(output, "        {result} = {command};")
                }
                CommandParameters::Register(register) => {
                    writeln!(
                        output,
                        "        {} {register};",
                        if command.starts_with("return-") {
                            "return"
                        } else {
                            command
                        }
                    )
                }
                CommandParameters::ResultRegister(result, register) => {
                    if command.starts_with("move") {
                        writeln!(output, "        {result} = {register};")
                    } else {
                        let op = unary_op(command);
                        if !op.is_empty() {
                            writeln!(output, "        {result} = {op}{register};")
                        } else {
                            if command != "array-length" {
                                eprintln!("Warning: Unrecognized unary operation {command}");
                            }
                            writeln!(output, "        {result} = {command} {register};")
                        }
                    }
                }
                CommandParameters::RegisterRegister(register1, register2) => {
                    let op = bin_op(command);
                    if !op.is_empty() {
                        writeln!(output, "        {register1} {op}= {register2};")
                    } else {
                        eprintln!("Warning: Unrecognized binary operation {command}");
                        writeln!(output, "        {command} {register1}, {register2};")
                    }
                }
                CommandParameters::ResultRegisterRegister(result, register1, register2) => {
                    if command.starts_with("aget") {
                        writeln!(output, "        {result} = {register1}[{register2}];")
                    } else {
                        let op = bin_op(command);
                        if !op.is_empty() {
                            writeln!(output, "        {result} = {register1} {op} {register2};")
                        } else {
                            writeln!(
                                output,
                                "        {result} = {command} {register1}, {register2};"
                            )
                        }
                    }
                }
                CommandParameters::RegisterRegisterRegister(register1, register2, register3) => {
                    writeln!(output, "        {register2}[{register3}] = {register1};")
                }
                CommandParameters::ResultLiteral(result, literal) => {
                    writeln!(output, "        {result} = {literal};")
                }
                CommandParameters::ResultRegisterLiteral(result, register, literal) => {
                    let op = bin_op(command);
                    if !op.is_empty() {
                        if command.starts_with("rsub-") {
                            writeln!(output, "        {result} = {literal} {op} {register};")
                        } else {
                            writeln!(output, "        {result} = {register} {op} {literal};")
                        }
                    } else {
                        eprintln!("Warning: Unrecognized binary operation {command}");
                        writeln!(
                            output,
                            "        {result} = {command} {register}, {literal};"
                        )
                    }
                }
                CommandParameters::ResultType(result, r#type) => {
                    if command == "new-instance" {
                        writeln!(output, "        {result} = new {type};")
                    } else {
                        writeln!(output, "        {result} = class {type};")
                    }
                }
                CommandParameters::RegisterType(register, r#type) => {
                    writeln!(output, "        {command} {register}, {type};")
                }
                CommandParameters::ResultRegisterType(result, register, r#type) => {
                    writeln!(output, "        {result} = {command} {register}, {type};")
                }
                CommandParameters::ResultRegistersType(result, registers, _) => {
                    if let Some(result) = result {
                        writeln!(
                            output,
                            "        {result} = {{{}}};",
                            registers.to_list(false).1
                        )
                    } else {
                        writeln!(output, "        {{{}}};", registers.to_list(false).1)
                    }
                }
                CommandParameters::ResultField(result, field) => {
                    writeln!(output, "        {result} = <{field}>;")
                }
                CommandParameters::RegisterField(register, field) => {
                    writeln!(output, "        <{field}> = {register};")
                }
                CommandParameters::ResultRegisterField(result, register, field) => {
                    writeln!(output, "        {result} = {register}.<{field}>;")
                }
                CommandParameters::RegisterRegisterField(register1, register2, field) => {
                    writeln!(output, "        {register2}.<{field}> = {register1};")
                }
                CommandParameters::ResultRegistersMethod(result, registers, method) => {
                    writeln!(
                        output,
                        "        {};",
                        stringify_call(command, result, method, registers)
                    )
                }
                CommandParameters::ResultRegistersMethodCall(result, registers, method, call) => {
                    writeln!(
                        output,
                        "        {}, <{call}>;",
                        stringify_call(command, result, method, registers)
                    )
                }
                CommandParameters::Label(label) => writeln!(output, "        goto {label};"),
                CommandParameters::RegisterLabel(register, label) => {
                    let op = if_op(command);
                    if !op.is_empty() {
                        writeln!(output, "        if {register} {op} 0 goto {label};")
                    } else {
                        eprintln!("Warning: Unrecognized conditional operation {command}");
                        writeln!(output, "        {command} {register} goto {label};")
                    }
                }
                CommandParameters::RegisterData(register, data) => match data {
                    CommandData::Label(label) => {
                        writeln!(output, "        {command} {register}, {label};")
                    }
                    CommandData::PackedSwitch(first_key, targets) => {
                        writeln!(output, "        switch({register})")?;
                        writeln!(output, "        {{")?;
                        for (index, target) in targets.iter().enumerate() {
                            let key = first_key + (index as i64);
                            writeln!(
                                output,
                                "            case {}{:#x}: goto {target};",
                                if key.is_negative() { "-" } else { "" },
                                key.abs_diff(0)
                            )?;
                        }
                        writeln!(output, "        }};")
                    }
                    CommandData::SparseSwitch(targets) => {
                        writeln!(output, "        switch({register})")?;
                        writeln!(output, "        {{")?;
                        for (value, target) in targets {
                            writeln!(output, "            case {value}: goto {target};")?;
                        }
                        writeln!(output, "        }};")
                    }
                    CommandData::Array(values) => {
                        writeln!(output, "        {register} = {{")?;
                        for value in values {
                            writeln!(output, "            {value},")?;
                        }
                        writeln!(output, "        }};")
                    }
                },
                CommandParameters::RegisterRegisterLabel(register1, register2, label) => {
                    let op = if_op(command);
                    if op.is_empty() {
                        writeln!(
                            output,
                            "        {command} {register1}, {register2}, {label};"
                        )
                    } else {
                        writeln!(
                            output,
                            "        if {register1} {op} {register2} goto {label};"
                        )
                    }
                }
                CommandParameters::ResultCall(result, call) => {
                    writeln!(output, "        {result} = {call};")
                }
                CommandParameters::ResultMethodHandle(result, invoke_type, method) => {
                    writeln!(output, "        {result} = {invoke_type}@{method};")
                }
            },
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
            // line 6
            v3 = 0x400;
            v4 = class java.lang.String;
            v5 = new java.lang.String;
            if v6 > 0 goto cond_0;
            goto goto_1;
            v7 = p1[v3];
            v8 = p3 - p2;
            if v9 >= v10 goto cond_1;
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
