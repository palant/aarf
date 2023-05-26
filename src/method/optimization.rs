use std::collections::HashMap;

use super::Method;
use crate::instruction::{CommandData, Instruction};

impl Method {
    fn extract_data(&mut self) -> HashMap<String, CommandData> {
        let mut result = HashMap::new();
        let mut i = 0;
        while i < self.instructions.len() {
            if matches!(self.instructions[i], Instruction::Data(_)) {
                let instruction = self.instructions.remove(i);

                if let Some(Instruction::Label(label)) = self.instructions.get(i - 1) {
                    if let Instruction::Data(data) = instruction {
                        result.insert(label.clone(), data);
                    }
                    self.instructions.remove(i - 1);
                    i -= 1;
                } else {
                    eprintln!(
                        "Warning: Data block not preceded by a label in method <{} {}()>",
                        self.return_type, self.name
                    );
                }
            } else {
                i += 1;
            }
        }
        result
    }

    fn merge_line_numbers(&mut self, i: usize) -> usize {
        if i == 0 {
            return i;
        }

        let to = if let Instruction::LineNumber(_, to) = self.instructions[i] {
            to
        } else {
            return i;
        };

        if let Instruction::LineNumber(_, prev_to) = &mut self.instructions[i - 1] {
            *prev_to = to;
            self.instructions.remove(i);
            return i - 1;
        }
        i
    }

    fn inline_results(&mut self, i: usize) -> usize {
        if let Some(result) = self.instructions[i].get_moved_result() {
            // Got move-result variation, find preceding command
            let mut j = i;
            while j > 0 && !self.instructions[j - 1].is_command() {
                j -= 1;
            }

            if j > 0 {
                // Attempt to merge the instructions
                if self.instructions[j - 1].inline_result(result) {
                    self.instructions.remove(i);
                    return i - 1;
                }
            }
            eprintln!(
                "Warning: Failed inlining result in method <{} {}()>",
                self.return_type, self.name
            );
        }
        i
    }

    pub fn optimize(&mut self) {
        let command_data = self.extract_data();

        let mut i = 0;
        while i < self.instructions.len() {
            self.instructions[i].resolve_data(&command_data);
            i = self.merge_line_numbers(i);
            i = self.inline_results(i);
            i += 1;
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

    fn stringify(method: Method) -> String {
        let mut cursor = std::io::Cursor::new(Vec::new());
        method.write_jimple(&mut cursor).unwrap();
        String::from_utf8_lossy(&cursor.into_inner())
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn write_instruction() -> Result<(), ParseErrorDisplayed> {
        let input = tokenizer(r#"
            .method constructor <init>()V
                invoke-direct {v16, v17}, Ls1/b$a;-><init>(Lkotlin/jvm/internal/DefaultConstructorMarker;)Ljava/lang/String;
                move-result-object v15

                invoke-static {v18, v19}, Ls1/b;->d(J)J
                move-result-wide v13

                .line 1
                packed-switch v2, :pswitch_data_0

                sparse-switch v1, :sswitch_data_0

                .line 2
                .line 3
                .line 4
                .line 5
                fill-array-data v3, :array_0

                :pswitch_data_0
                .packed-switch -0x1
                    :pswitch_0
                    :pswitch_1
                    :pswitch_2
                .end packed-switch

                :sswitch_data_0
                .sparse-switch
                    -0x80t -> :sswitch_5
                    -0x4bt -> :sswitch_4
                    -0x47t -> :sswitch_3
                    -0x41t -> :sswitch_2
                    -0x2ct -> :sswitch_1
                    0x4et -> :sswitch_0
                .end sparse-switch

                :array_0
                .array-data 1
                    0x10
                    0x1f
                    -0x10
                    0x7f
                    0x7f
                .end array-data
            .end method
        "#.trim());

        let input = input.expect_directive("method")?;
        let (input, mut method) = Method::read(&input)?;
        assert!(input.expect_eof().is_ok());

        let expected = r#"
            void <init>()
            {
                v15 = invoke-direct v16.<java.lang.String s1.b$a.<init>(kotlin.jvm.internal.DefaultConstructorMarker)>(v17);

                v13 = invoke-static <long s1.b.d(long)>(v18, v19);

                // line 1
                switch(v2)
                {
                    case -0x1: goto pswitch_0;
                    case 0x0: goto pswitch_1;
                    case 0x1: goto pswitch_2;
                };

                switch(v1)
                {
                    case -0x80: goto sswitch_5;
                    case -0x4b: goto sswitch_4;
                    case -0x47: goto sswitch_3;
                    case -0x41: goto sswitch_2;
                    case -0x2c: goto sswitch_1;
                    case 0x4e: goto sswitch_0;
                };

                // line 2-5
                v3 = {
                    0x10,
                    0x1f,
                    -0x10,
                    0x7f,
                    0x7f,
                };
            }
        "#.split('\n').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n");

        method.optimize();
        assert_eq!(stringify(method), expected);

        Ok(())
    }
}
