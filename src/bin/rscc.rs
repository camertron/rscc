use std::{env, fs, str, mem};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use clap::{Parser, Subcommand};
use colored::Colorize;
use rscc::parser::Diagnostic;
use target_lexicon::Triple;
use std::str::FromStr;
use target_lexicon;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub mod version_info {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

#[derive(Parser, Debug)]
#[command(
    name="rscc",
    author="Cameron C. Dutro",
    version=version_info::version(),
    about="The RSC (Reasonably Simple Computer) compiler"
)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        about="Compile an RSC program into an executable",
        arg_required_else_help = true,
    )]
    Build {
        #[arg(long, short, value_name="FILE", help="The file containing the program to compile")]
        file: String,

        #[arg(long, short, value_name="OUTPUT_PATH", default_value=".", help="The directory into which build artifacts and the resulting compiled executable should be written")]
        output_path: String,
    },

    #[command(
        about="Run an RSC program",
        arg_required_else_help = true,
    )]
    Run {
        #[arg(long, short, value_name="FILE", help="The file containing the program to run")]
        file: String,
    },

    #[command(
        about="Check an RSC program for problems",
        long_about="Check an RSC program for problems. If there are errors, this command will print them and exit with a status code of 1. If there are no errors, this command exits with a status code of 0.",
        arg_required_else_help = true,
    )]
    Check {
        #[arg(long, short, value_name="FILE", help="The file containing the program to check")]
        file: String,
    }
}

fn main() -> ExitCode {
    let options = CLI::parse();

    match options.command {
        Commands::Build { file, output_path } => {
            build(&file, &output_path)
        }

        Commands::Run { file } => {
            run(&file)
        }

        Commands::Check { file } => {
            check(&file)
        }
    }
}

fn build(file: &str, output_path: &str) -> ExitCode {
    match parse_file_and_diagnose(file) {
        Some(parse_result) => {
            build_instrs(file, output_path, parse_result.instructions)
        }

        None => ExitCode::from(1)
    }
}

fn run(file: &str) -> ExitCode {
    match parse_file_and_diagnose(file) {
        Some(parse_result) => {
            run_instrs(parse_result.instructions)
        }

        None => ExitCode::from(1)
    }
}

fn check(file: &str) -> ExitCode {
    match parse_file_and_diagnose(file) {
        Some(_) => ExitCode::from(0),
        None => ExitCode::from(1)
    }
}

fn build_instrs(file: &str, output_path: &str, instructions: Vec<rscc::parser::Instruction>) -> ExitCode {
    let rsc_module = rscc::emitter::emit_object_module(
        Triple::from_str(built_info::TARGET).unwrap(),
        instructions
    );

    let path = Path::new(file);
    let base_name = path.file_stem().unwrap().to_str().unwrap();
    let out_dir = Path::new(output_path).join("target").join(base_name);

    fs::create_dir_all(&out_dir).unwrap();

    let o_file = out_dir.join(&format!("{}.o", base_name));
    let mut file = File::create(o_file.as_os_str()).unwrap();
    rsc_module.product.object.write_stream(&mut file).unwrap();

    let mut build = cc::Build::new();

    let c_helper = match find_c_helper_file() {
        Some(c_helper) => c_helper,
        None => {
            println!("Could not find helper file rsc.c");
            return ExitCode::from(2);
        }
    };

    modify_path_if_necessary();

    build.file(c_helper)
        .object(o_file)
        .target(built_info::TARGET)
        .opt_level(built_info::OPT_LEVEL.parse().unwrap())
        .host(built_info::HOST)
        .out_dir(out_dir.as_os_str());

    build.compile(base_name);

    let compiler = build.get_compiler();
    let a_file = out_dir.join(format!("lib{}.a", base_name));

    let compile_result = std::process::Command::new(compiler.path().to_str().unwrap())
        .arg(a_file)
        .arg("-o")
        .arg(out_dir.join(base_name))
        .status();

    match compile_result {
        Ok(exit_status) => {
            ExitCode::from(exit_status.code().unwrap_or(0) as u8)
        }

        Err(_) => {
            println!("Compilation failed");
            ExitCode::from(1)
        }
    }
}

fn run_instrs(instructions: Vec<rscc::parser::Instruction>) -> ExitCode {
    let rsc_module = rscc::emitter::emit_jit_module(
        Triple::from_str(built_info::TARGET).unwrap(),
        instructions
    );

    let main = rsc_module.module.get_finalized_function(rsc_module.main_id);
    let code_fn = unsafe { mem::transmute::<_, fn() -> ()>(main) };

    code_fn();

    ExitCode::from(0)
}

fn parse_file_and_diagnose(file: &str) -> Option<rscc::parser::ParseResult> {
    match parse_file(file) {
        Ok(parse_result) => {
            if parse_result.diagnostics.len() > 0 {
                print_diagnostics(&parse_result.diagnostics, &parse_result.code);
                None
            } else {
                Some(parse_result)
            }
        }

        Err(_) => None
    }
}

fn parse_file(file: &str) -> Result<rscc::parser::ParseResult, ExitCode> {
    let path = Path::new(file);

    match fs::exists(path) {
        Ok(exists) => {
            if !exists {
                println!("No such file '{}'", file);
                return Err(ExitCode::from(1))
            }
        }

        Err(e) => {
            println!("{}", e);
            return Err(ExitCode::from(1));
        }
    }

    match fs::read_to_string(path) {
        Ok(contents) => {
            Ok(rscc::parser::parse(&contents))
        }

        Err(e) => {
            println!("{}", e);
            Err(ExitCode::from(1))
        }
    }
}

fn print_diagnostics(diagnostics: &Vec<Diagnostic>, code: &str) {
    println!("Found {} compilation problem(s)\n", diagnostics.len());

    for (idx, diagnostic) in diagnostics.iter().enumerate() {
        println!("{}", format!("-------------- PROBLEM {} ---------------", idx + 1).magenta());
        println!("{}", diagnostic.annotate(code));
    }
}

fn find_c_helper_file() -> Option<PathBuf> {
    let found = match &mut env::current_exe() {
        Ok(exe_path) => {
            // look next to rscc executable
            exe_path.pop();
            let p = exe_path.join("rsc.c");

            if p.exists() {
                Some(p)
            } else {
                None
            }
        }

        Err(_) => {
            None
        }
    };

    found.or_else(|| {
        // default to looking in current working directory
        let p = Path::new("rsc.c").to_path_buf();

        if p.exists() {
            Some(p)
        } else {
            None
        }
    })
}

fn modify_path_if_necessary() {
    match &mut env::current_exe() {
        Ok(exe_path) => {
            exe_path.pop();
            let mingw_path = exe_path.join("mingw64_rsc").join("bin");

            if mingw_path.exists() {
                match std::env::var("PATH") {
                    Ok(existing_path) => {
                        std::env::set_var("PATH", format!("{};{}", existing_path, mingw_path.to_str().unwrap()));
                    }

                    Err(_) => {
                        std::env::set_var("PATH", mingw_path.as_os_str());
                    }
                }
            }
        }

        Err(_) => ()
    };
}
