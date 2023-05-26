#![deny(elided_lifetimes_in_paths)]
#![deny(explicit_outlives_requirements)]
#![deny(keyword_idents)]
#![deny(meta_variable_misuse)]
#![deny(missing_debug_implementations)]
#![deny(non_ascii_idents)]
#![warn(noop_method_call)]
#![deny(pointer_structural_match)]
#![deny(single_use_lifetimes)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![warn(unused_crate_dependencies)]
#![deny(unused_import_braces)]
#![deny(unused_lifetimes)]
#![warn(unused_macro_rules)]
#![warn(unused_tuple_struct_fields)]
#![deny(variant_size_differences)]

pub mod access_flag;
pub mod annotation;
pub mod class;
pub mod error;
pub mod field;
pub mod instruction;
pub mod literal;
pub mod method;
pub mod tokenizer;
pub mod r#type;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::class::Class;
use crate::tokenizer::Tokenizer;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the apktool command or apktool.jar package
    #[arg(short, long)]
    apktool_path: Option<String>,

    #[command(subcommand)]
    command: ArgsCommand,
}

#[derive(Subcommand, Debug)]
enum ArgsCommand {
    /// Decompile APK into Jimple code
    Decompile {
        apk_path: PathBuf,
        output_dir: PathBuf,
    },
}

fn locate_apktool(apktool_path: Option<String>) -> std::process::Command {
    if let Some(apktool_path) = apktool_path {
        if apktool_path.ends_with(".jar") {
            if let Ok(java_path) = which::which("java") {
                let mut command = std::process::Command::new(java_path);
                command.arg("-jar").arg(apktool_path);
                command
            } else {
                eprintln!("Supposed to run apktool as JAR file, yet Java could not be found. Is it installed?");
                std::process::exit(1);
            }
        } else {
            std::process::Command::new(apktool_path)
        }
    } else if let Ok(apktool_path) = which::which("apktool") {
        std::process::Command::new(apktool_path)
    } else {
        eprintln!("Could not find apktool. If you installed it, please pass --apktool-path command line parameter explicitly.");
        std::process::exit(1);
    }
}

fn main() {
    let args = Args::parse();

    match &args.command {
        ArgsCommand::Decompile {
            apk_path,
            output_dir,
        } => {
            locate_apktool(args.apktool_path)
                .arg("decode")
                .arg("--output")
                .arg(output_dir)
                .arg(apk_path)
                .output()
                .expect("Failed to run apktool");

            for entry in walkdir::WalkDir::new(output_dir)
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file()
                    || entry.path().extension().filter(|s| *s == "smali").is_none()
                {
                    continue;
                }

                match Tokenizer::from_file(entry.path()) {
                    Ok(input) => match Class::read(&input) {
                        Ok((_, mut class)) => {
                            let target = entry.path().with_extension("jimple");
                            let mut output =
                                std::io::BufWriter::new(std::fs::File::create(target).unwrap());
                            class.optimize();
                            class.write_jimple(&mut output).unwrap();
                        }
                        Err(error) => {
                            eprintln!("{}", error);
                            break;
                        }
                    },
                    Err(error) => {
                        eprintln!("{}", error);
                        break;
                    }
                }
            }
        }
    }
}
