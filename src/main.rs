#![allow(clippy::upper_case_acronyms)]
use clap::{Parser, Subcommand};
use popper_compiler::check_program;
use popper_compiler::get_ast;
use popper_compiler::{compile, execute_llvm};
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author = "NightProg", version = "1.0.0", about = "The Popper CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// parsing a file
    Parse {
        #[arg(short, long)]
        json: bool,

        #[arg(value_hint = clap::ValueHint::DirPath)]
        file: std::path::PathBuf,

        #[arg(long, short, value_hint = clap::ValueHint::DirPath)]
        output: Option<std::path::PathBuf>,
    },
    // check a file
    Check {
        #[arg(value_hint = clap::ValueHint::DirPath)]
        file: std::path::PathBuf,
    },
    /// compile to the Popper MIR
    Mirage {
        #[arg(value_hint = clap::ValueHint::DirPath)]
        file: std::path::PathBuf,

        #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
        output: Option<std::path::PathBuf>,
    },
    /// compile to LLVM Bytecode
    LLVM {
        #[arg(value_hint = clap::ValueHint::DirPath)]
        file: std::path::PathBuf,

        #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
        output: Option<std::path::PathBuf>,

        #[arg(short, long)]
        debug: bool,
    },

    /// Run a popper file
    Run {
        #[arg(value_hint = clap::ValueHint::DirPath)]
        file: std::path::PathBuf,

        #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
        target: Option<std::path::PathBuf>,

        // #[arg(short, long)]
        // inkwell: bool,
        #[arg(short, long)]
        debug: bool,
    },

    Clean {
        #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
        target: Option<std::path::PathBuf>,

        #[arg(short = 'l', long)]
        only_libs: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { json, file, output } => {
            let string_file = file.to_str().expect("Unable to get a str");
            let content = std::fs::read_to_string(string_file).expect("File not found");
            let ast = get_ast(content.as_str(), string_file);
            if let Some(a) = ast {
                if json {
                    let s: String = serde_json::to_string_pretty(&a).unwrap();
                    if let Some(out) = output {
                        std::fs::File::open(out)
                            .expect("File Not Found")
                            .write_all(s.as_bytes())
                            .expect("Cannot write to file");
                    } else {
                        println!("{}", s);
                    };
                } else {
                    println!("{:#?}", a)
                }
            } else {
                eprintln!("Unable to parse file")
            }
        }
        Commands::Check { file } => {
            let string_file = file.to_str().expect("Unable to get a str");
            let content = std::fs::read_to_string(string_file).expect("File not found");
            let ast = get_ast(content.as_str(), string_file);
            if let Some(a) = ast {
                if check_program(a, content.as_str(), string_file) {
                    println!("Program is valid");
                } else {
                    println!("Program is invalid");
                }
            } else {
                eprintln!("Unable to parse file")
            }
        }
        Commands::Mirage { file, output } => {
            let string_file = file.to_str().expect("Unable to get a str");
            let content = std::fs::read_to_string(string_file).expect("File not found");
            let ast = get_ast(content.as_str(), string_file);
            if let Some(a) = ast {
                if check_program(a.clone(), content.as_str(), string_file) {
                    let res = compile(a, string_file, false).print_to_string();
                    if let Some(out) = output {
                        std::fs::File::open(out)
                            .expect("File Not Found")
                            .write_all(res.as_bytes())
                            .expect("Cannot write to file");
                    } else {
                        println!("{}", res);
                    };
                } else {
                    println!("Program is invalid");
                }
            } else {
                eprintln!("Unable to parse file")
            }
        }
        Commands::LLVM {
            file,
            output,
            debug,
        } => {
            let string_file = file.to_str().expect("Unable to get a str");
            let content = std::fs::read_to_string(string_file).expect("File not found");
            let ast = get_ast(content.as_str(), string_file);
            if let Some(a) = ast {
                if check_program(a.clone(), content.as_str(), string_file) {
                    let res = compile(a, string_file, debug).print_llvm_to_string();
                    if let Some(out) = output {
                        std::fs::File::open(out)
                            .expect("File Not Found")
                            .write_all(res.as_bytes())
                            .expect("Cannot write to file");
                    } else {
                        println!("{}", res);
                    };
                } else {
                    println!("Program is invalid");
                }
            } else {
                eprintln!("Unable to parse file")
            }
        }
        Commands::Run {
            file,
            target,
            debug,
        } => {
            let string_file = file.to_str().expect("Unable to get a str");
            let content = std::fs::read_to_string(string_file).expect("File not found");
            let ast = get_ast(content.as_str(), string_file);
            if let Some(a) = ast {
                if check_program(a.clone(), content.as_str(), string_file) {
                    let res = compile(a, string_file, debug).print_llvm_to_string();
                    let target = target.unwrap_or(std::path::PathBuf::from("./target_popper"));
                    execute_llvm(
                        res,
                        string_file.to_string(),
                        target.to_str().unwrap().to_string(),
                        debug,
                    );
                } else {
                    println!("Program is invalid");
                }
            } else {
                eprintln!("Unable to parse file")
            }
        }
        Commands::Clean { target, only_libs } => {
            let target = target.unwrap_or(std::path::PathBuf::from("./target_popper"));
            if only_libs {
                std::fs::remove_dir_all(target.join("libs"))
                    .expect("Unable to remove target directory");
            } else {
                std::fs::remove_dir_all(target).expect("Unable to remove target directory");
            }
        }
    }
}
