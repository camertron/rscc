extern crate rscc;

use rand::Rng;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::mem;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::process::ExitCode;
use clap::{Parser, Subcommand};
use cranelift::prelude::*;
use cranelift_codegen::ir::{FuncRef, Function};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use rscc::Instruction;
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
    },

    #[command(
        about="Run an RSC program",
        arg_required_else_help = true,
    )]
    Run {
        #[arg(long, short, value_name="FILE", help="The file containing the program to run")]
        file: String,
    }
}

struct Program<'a, M: Module> {
    module: &'a mut M,
    rsc_init: FuncRef,
    rsc_out: FuncRef,
    rsc_rand: FuncRef,
    rsc_input: FuncRef,
    accum: Variable,
    location_vars: HashMap<u32, Variable>,
    var_index: usize,
}

extern "C" fn rsc_init() {
}

extern "C" fn rsc_out(number: f64) {
    println!("{:.2}", number);
}

extern "C" fn rsc_rand() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range((-0x10000 as f64)..(0x10000 as f64))
}

extern "C" fn rsc_input() -> f64 {
    loop {
        print!("Input: ");
        io::stdout().flush().unwrap();

        let mut buffer = String::new();

        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let trimed_buffer = buffer.trim();

                match trimed_buffer.parse::<f64>() {
                    Ok(float) => return float,
                    Err(_) => {}
                }
            }

            Err(_) => {}
        }

        println!("Invalid entry, try again.")
    }
}

impl<'a, M: Module> Program<'a, M> {
    fn new(main: &mut FunctionBuilder, module: &'a mut M) -> Self {
        main.func.signature.call_conv = module.isa().default_call_conv();
        main.func.signature.returns.push(AbiParam::new(types::I32));

        // Declare the rsc_init function
        let init_sig = module.make_signature();

        let init_func_id = module
            .declare_function("rsc_init", Linkage::Import, &init_sig)
            .unwrap();

        // Declare the rsc_out function
        let mut out_sig = module.make_signature();
        out_sig.params.push(AbiParam::new(types::F64));

        let out_func_id = module
            .declare_function("rsc_out", Linkage::Import, &out_sig)
            .unwrap();

        // Declare the rsc_rand function
        let mut rand_sig = module.make_signature();
        rand_sig.returns.push(AbiParam::new(types::F64));

        let rand_func_id = module
            .declare_function("rsc_rand", Linkage::Import, &rand_sig)
            .unwrap();

        // Declare the rsc_input function
        let mut input_sig = module.make_signature();
        input_sig.returns.push(AbiParam::new(types::F64));

        let input_func_id = module
            .declare_function("rsc_input", Linkage::Import, &input_sig)
            .unwrap();

        let accum = Variable::new(0);

        Self {
            rsc_init: module.declare_func_in_func(init_func_id, main.func),
            rsc_out: module.declare_func_in_func(out_func_id, main.func),
            rsc_rand: module.declare_func_in_func(rand_func_id, main.func),
            rsc_input: module.declare_func_in_func(input_func_id, main.func),
            module,
            accum: accum,
            location_vars: HashMap::new(),
            var_index: 1,
        }
    }
}

impl<'a, M: Module> Program<'a, M> {
    fn store(self: &mut Self, func: &mut FunctionBuilder, location: u32) {
        let accum = self.accum;
        let location = self.get_or_create_loc(func, location);
        let accum_val = func.use_var(accum);
        func.def_var(*location, accum_val);
    }

    fn load(self: &mut Self, func: &mut FunctionBuilder, location: u32) {
        let location = self.get_or_create_loc(func, location);
        let location_val = func.use_var(*location);
        func.def_var(self.accum, location_val);
    }

    fn input(self: &mut Self, func: &mut FunctionBuilder, location: u32) {
        let input_ref = self.rsc_input;
        let loc = self.get_or_create_loc(func, location);
        let input_inst = func.ins().call(input_ref, &[]);
        let input_val = func.inst_results(input_inst)[0];
        func.def_var(*loc, input_val);
    }

    fn get_or_create_loc(self: &mut Self, func: &mut FunctionBuilder, location: u32) -> &Variable {
        if !self.location_vars.contains_key(&location) {
            let loc = Variable::new(self.var_index);
            self.var_index += 1;
            func.declare_var(loc, types::F64);
            self.location_vars.insert(location, loc);
            let rand_val = self.rand(func);
            func.def_var(loc, rand_val);
        }

        self.location_vars.get(&location).unwrap()
    }

    fn rand(self: &Self, func: &mut FunctionBuilder) -> Value {
        let init_inst = func.ins().call(self.rsc_rand, &[]);
        func.inst_results(init_inst)[0]
    }
}

fn main() -> ExitCode {
    let options = CLI::parse();

    match options.command {
        Commands::Build { file } => {
            build(&file)
        }

        Commands::Run { file } => {
            run(&file)
        }
    }
}

fn build(file: &str) -> ExitCode {
    match parse(file) {
        Ok(parse_result) => {
            build_instrs(file, parse_result.instructions)
        }

        Err(_) => ExitCode::from(1)
    }
}

fn build_instrs(file: &str, instructions: Vec<rscc::Instruction>) -> ExitCode {
    let mut shared_builder = settings::builder();
    shared_builder.enable("is_pic").unwrap();

    let shared_flags = settings::Flags::new(shared_builder);
    let isa_builder = isa::lookup(Triple::from_str(built_info::TARGET).unwrap()).unwrap();
    let isa = isa_builder.finish(shared_flags).unwrap();
    let obj_builder = ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(obj_builder);

    compile(instructions, &mut module);

    let path = Path::new(file);
    let res = module.finish();
    let base_name = path.file_stem().unwrap().to_str().unwrap();
    let out_dir = Path::new("target").join(base_name);

    fs::create_dir_all(&out_dir).unwrap();

    let o_file = out_dir.join(&format!("{}.o", base_name));
    let mut file = File::create(o_file.as_os_str()).unwrap();
    res.object.write_stream(&mut file).unwrap();

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
        .arg(base_name)
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

fn parse(file: &str) -> Result<rscc::ParseResult, ()> {
    let path = Path::new(file);
    let contents = fs::read_to_string(path).unwrap();
    let parse_result = rscc::parse(&contents);

    if parse_result.diagnostics.len() > 0 {
        println!("{}", parse_result.diagnostics[0].annotate(&contents));
        Err(())
    } else {
        Ok(parse_result)
    }
}

fn run(file: &str) -> ExitCode {
    match parse(file) {
        Ok(parse_result) => {
            run_instrs(parse_result.instructions)
        }

        Err(_) => ExitCode::from(1)
    }
}

fn run_instrs(instructions: Vec<rscc::Instruction>) -> ExitCode {
    let mut shared_builder = settings::builder();

    // Disable PIC so code can run on aarch64.
    // See: https://github.com/bytecodealliance/wasmtime/issues/2735#issuecomment-801471323
    shared_builder.set("is_pic", "false").unwrap();

    let shared_flags = settings::Flags::new(shared_builder);
    let isa_builder = isa::lookup(Triple::from_str(built_info::TARGET).unwrap()).unwrap();
    let isa = isa_builder.finish(shared_flags).unwrap();
    let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    // fill in these helper functions with rust implementations so we can actually run the code
    builder.symbol("rsc_init", rsc_init as *const u8);
    builder.symbol("rsc_rand", rsc_rand as *const u8);
    builder.symbol("rsc_out", rsc_out as *const u8);
    builder.symbol("rsc_input", rsc_input as *const u8);

    let mut module = JITModule::new(builder);

    let main_id = compile(instructions, &mut module);

    module.finalize_definitions().unwrap();

    let code_ptr = module.get_finalized_function(main_id);
    let code_fn = unsafe { mem::transmute::<_, fn() -> ()>(code_ptr) };

    code_fn();

    ExitCode::from(0)
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

fn compile<M: Module>(instructions: Vec<rscc::Instruction>, module: &mut M) -> FuncId {
    let mut builder_context = FunctionBuilderContext::new();
    let main_func = Function::new();
    let mut ctx = Context::for_function(main_func);
    let mut main = FunctionBuilder::new(
        &mut ctx.func,
        &mut builder_context
    );

    let mut program: Program<M> = Program::new(&mut main, module);

    // Create the entry block, to start emitting code in.
    let entry_block = main.create_block();

    // Since this is the entry block, add block parameters corresponding to
    // the function's parameters.
    //
    // TODO: Streamline the API here.
    main.append_block_params_for_function_params(entry_block);

    // Tell the builder to emit code in this block.
    main.switch_to_block(entry_block);

    // And, tell the builder that this block will have no further
    // predecessors. Since it's the entry block, it won't have any
    // predecessors.
    main.seal_block(entry_block);

    let rand_val = program.rand(&mut main);

    main.declare_var(program.accum, types::F64);
    main.def_var(program.accum, rand_val);

    main.ins().call(program.rsc_init, &[]);

    emit(&instructions, 0, &mut program, &mut main);

    // Tell the builder we're done with this function.
    main.finalize();

    // Next, declare the function to jit. Functions must be declared
    // before they can be called, or defined.
    //
    // TODO: This may be an area where the API should be streamlined; should
    // we have a version of `declare_function` that automatically declares
    // the function?
    let main_id = program
        .module
        .declare_function("main", Linkage::Export, &ctx.func.signature)
        .map_err(|e| e.to_string())
        .unwrap();

    program.module
        .define_function(main_id, &mut ctx)
        .map_err(|e| e.to_string())
        .unwrap();

    // Now that compilation is finished, we can clear out the context state.
    program.module.clear_context(&mut ctx);

    main_id
}

fn emit<M: Module>(instructions: &Vec<rscc::Instruction>, start_line: usize, program: &mut Program<M>, main: &mut FunctionBuilder) {
    for instr in instructions {
        if instr.lineno() < start_line {
            continue
        }

        match instr {
            // LoaD Accumulator
            Instruction::LDA(lda) => {
                program.load(main, lda.location);
            },

            // LoaD Constant
            Instruction::LDC(ldc) => {
                let value = main.ins().f64const(ldc.value);
                main.def_var(program.accum, value);
            },

            // STore Accumulator
            Instruction::STA(sta) => {
                program.store(main, sta.location);
            },

            // INPut
            Instruction::INP(inp) => {
                program.input(main, inp.location);
            },

            // OUTput
            Instruction::OUT(out) => {
                program.load(main, out.location);
                let accum_val = main.use_var(program.accum);
                main.ins().call(program.rsc_out, &[accum_val]);
            },

            // ADd Constant
            Instruction::ADC(adc) => {
                let value = main.ins().f64const(adc.value);
                let accum_val = main.use_var(program.accum);
                let new_accum = main.ins().iadd(accum_val, value);
                main.def_var(program.accum, new_accum);
            },

            // ADD
            Instruction::ADD(add) => {
                let location = program.get_or_create_loc(main, add.location);
                let location_val = main.use_var(*location);
                let accum_val = main.use_var(program.accum);
                let new_accum = main.ins().fadd(accum_val, location_val);
                main.def_var(program.accum, new_accum);
            },

            // SUBtract
            Instruction::SUB(sub) => {
                let location = program.get_or_create_loc(main, sub.location);
                let location_val = main.use_var(*location);
                let accum_val = main.use_var(program.accum);
                let new_accum = main.ins().fsub(accum_val, location_val);
                main.def_var(program.accum, new_accum);
            },

            // MULtiply
            Instruction::MUL(mul) => {
                let location = program.get_or_create_loc(main, mul.location);
                let location_val = main.use_var(*location);
                let accum_val = main.use_var(program.accum);
                let new_accum = main.ins().fmul(accum_val, location_val);
                main.def_var(program.accum, new_accum);
            },

            // DIVide
            Instruction::DIV(div) => {
                let location = program.get_or_create_loc(main, div.location);
                let location_val = main.use_var(*location);
                let accum_val = main.use_var(program.accum);
                let new_accum = main.ins().fdiv(accum_val, location_val);
                main.def_var(program.accum, new_accum);
            },

            // BRanch Unconditional
            Instruction::BRU(bru) => {
                let then_block = main.create_block();

                main.ins().jump(then_block, &[]);
                main.switch_to_block(then_block);
                main.seal_block(then_block);

                emit(instructions, bru.location as usize, program, main);

                break;
            },

            // Branch Positive Accumulator
            Instruction::BPA(bpa) => {
                emit_branch(
                    FloatCC::GreaterThan,  // condition
                    bpa.location,          // jump here if condition holds
                    instructions,          // list of instructions
                    program,
                    main
                );
            },

            // Branch Negative Accumulator
            Instruction::BNA(bna) => {
                emit_branch(
                    FloatCC::LessThan,     // condition
                    bna.location,          // jump here if condition holds
                    instructions,          // list of instructions
                    program,
                    main
                );
            },

            // Branch Zero Accumulator
            Instruction::BZA(bza) => {
                emit_branch(
                    FloatCC::Equal,        // condition
                    bza.location,          // jump here if condition holds
                    instructions,          // list of instructions
                    program,
                    main
                );
            },

            // Stop
            Instruction::STP(_) => {
                let izero = main.ins().iconst(types::I32, 0);
                main.ins().return_(&[izero]);
                break;
            },
        }
    }
}

fn emit_branch<M: Module>(condition: FloatCC, then_location: u32, instructions: &Vec<rscc::Instruction>, program: &mut Program<M>, main: &mut FunctionBuilder) {
    let accum_val = main.use_var(program.accum);
    let fzero = main.ins().f64const(0.0);
    let condition = main.ins().fcmp(condition, accum_val, fzero);

    let then_block = main.create_block();
    let else_block = main.create_block();
    let merge_block = main.create_block();

    main.ins().brif(condition, then_block, &[], else_block, &[]);

    main.switch_to_block(then_block);
    main.seal_block(then_block);
    emit(instructions, then_location as usize, program, main);

    main.switch_to_block(else_block);
    main.seal_block(else_block);
    main.ins().jump(merge_block, &[]);

    main.switch_to_block(merge_block);
    main.seal_block(merge_block);
}
