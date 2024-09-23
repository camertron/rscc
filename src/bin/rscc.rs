extern crate rscc;

use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use clap::Parser;
use cranelift::prelude::*;
use cranelift_codegen::ir::{FuncRef, Function};
use cranelift_codegen::Context;
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use rscc::{Instruction, ParseError};
use target_lexicon::triple;
use target_lexicon;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser, Debug)]
#[command(author="Cameron C. Dutro")]
#[command(version="0.1.0")]
#[command(about="Compile an RSC program into an executable.")]
struct CLI {
    #[arg(long, short, value_name="FILE", help="The file to compile.")]
    file: String,
}

struct Program {
    module: ObjectModule,
    rsc_out: FuncRef,
    rsc_rand: FuncRef,
    accum: Variable,
    zero: Value,
    location_vars: HashMap<i32, Variable>,
    var_index: usize,
}

impl Program {
    fn new(main: &mut FunctionBuilder) -> Self {
        let mut shared_builder = settings::builder();
        shared_builder.enable("is_pic").unwrap();
        let shared_flags = settings::Flags::new(shared_builder);

        let isa_builder = isa::lookup(triple!("aarch64-unknown-unknown-macho")).unwrap();
        let isa = isa_builder.finish(shared_flags).unwrap();
        let obj_builder = ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap();
        let mut module = ObjectModule::new(obj_builder);

        main.func.signature.call_conv = module.isa().default_call_conv();
        main.func.signature.returns.push(AbiParam::new(types::I32));

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

        let mut out_sig = module.make_signature();
        out_sig.params.push(AbiParam::new(types::I32));

        // Declare the function in the module
        let out_func_id = module
            .declare_function("rsc_out", Linkage::Import, &out_sig)
            .unwrap();

        let mut rand_sig = module.make_signature();

        rand_sig.returns.push(AbiParam::new(types::I32));

        // Declare the function in the module
        let rand_func_id = module
            .declare_function("rsc_rand", Linkage::Import, &rand_sig)
            .unwrap();

        let zero = main.ins().iconst(types::I32, 0);
        let accum = Variable::new(0);

        main.declare_var(accum, types::I32);
        main.def_var(accum, zero);

        Self {
            rsc_out: module.declare_func_in_func(out_func_id, main.func),
            rsc_rand: module.declare_func_in_func(rand_func_id, main.func),
            module: module,
            accum: accum,
            zero: zero,
            location_vars: HashMap::new(),
            var_index: 1,
        }
    }
}

impl Program {
    fn store(self: &mut Self, func: &mut FunctionBuilder, location: i32) {
        let accum = self.accum;
        let location = self.get_or_create_loc(func, location);
        let accum_val = func.use_var(accum);
        func.def_var(*location, accum_val);
    }

    fn load(self: &mut Self, func: &mut FunctionBuilder, location: i32) {
        let location = self.get_or_create_loc(func, location);
        let location_val = func.use_var(*location);
        func.def_var(self.accum, location_val);
    }

    fn get_or_create_loc<'a>(self: &'a mut Self, func: &mut FunctionBuilder, location: i32) -> &'a Variable {
        self.location_vars.entry(location).or_insert_with(|| {
            let loc = Variable::new(self.var_index);
            let init_inst = func.ins().call(self.rsc_rand, &[]);
            let init_val = func.inst_results(init_inst)[0];
            self.var_index += 1;
            func.declare_var(loc, types::I32);
            func.def_var(loc, init_val);
            loc
        })
    }
}

fn main() -> std::io::Result<()> {
    let options = CLI::parse();
    let path = Path::new(&options.file);
    let contents = fs::read_to_string(path);
    let instructions = rscc::parse(&contents.unwrap());

    match instructions {
        Ok(instructions) => {
            let res = compile(instructions);
            let base_name = path.file_stem().unwrap().to_str().unwrap();
            let o_file = &format!("{}.o", base_name);
            let mut file = File::create(o_file).unwrap();
            cc::Build::new()
                .file("rsc.c")
                .file(o_file)
                .target(built_info::TARGET)
                .opt_level(built_info::OPT_LEVEL.parse().unwrap())
                .host(built_info::HOST)
                .out_dir(".")
                .compile(base_name);
            res.object.write_stream(&mut file).unwrap();
        },

        Err(err) => {
            match err {
                ParseError::InvalidInstruction(invalid_instruction) => {
                    eprintln!("Invalid instruction: {}", invalid_instruction.instruction);
                },
            }
        }
    }

    Ok(())
}

fn compile(instructions: Vec<rscc::Instruction>) -> ObjectProduct {
    let mut builder_context = FunctionBuilderContext::new();
    let main_func = Function::new();
    let mut ctx = Context::for_function(main_func);
    let mut main = FunctionBuilder::new(
        &mut ctx.func,
        &mut builder_context
    );

    let mut program: Program = Program::new(&mut main);

    for instr in instructions {
        match instr {
            Instruction::LDC(ldc) => {
                let value = main.ins().iconst(types::I32, ldc.operand as i64);
                main.def_var(program.accum, value);
            },

            Instruction::STA(sta) => {
                program.store(&mut main, sta.operand);
            },

            Instruction::OUT(out) => {
                program.load(&mut main, out.operand);
                let accum_val = main.use_var(program.accum);
                main.ins().call(program.rsc_out, &[accum_val]);
            },

            Instruction::STP(_) => {
                break;
            },
        }
    }

    main.ins().return_(&[program.zero]);

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

    program.module.finish()
}

// pub fn create_data(program: &mut Program, name: &str, contents: Vec<u8>) -> Result<DataId, String> {
//     // The steps here are analogous to `compile`, except that data is much
//     // simpler than functions.
//     program.data_description.define(contents.into_boxed_slice());
//     let id = program
//         .module
//         .declare_data(name, Linkage::Export, true, false)
//         .map_err(|e| e.to_string())?;

//     program.module
//         .define_data(id, &program.data_description)
//         .map_err(|e| e.to_string())?;
//     program.data_description.clear();
//     Ok(id)
// }
