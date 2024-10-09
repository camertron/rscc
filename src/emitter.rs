use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::FuncId;
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use target_lexicon::Triple;
use rand::Rng;
use std::io;
use std::io::Write;

pub struct RSCObjectModule {
    pub product: ObjectProduct
}

pub struct RSCJITModule {
    pub module: JITModule,
    pub main_id: FuncId
}

pub fn emit_object_module(triple: Triple, instructions: Vec<crate::parser::Instruction>) -> RSCObjectModule {
    let mut shared_builder = settings::builder();
    shared_builder.enable("is_pic").unwrap();

    let shared_flags = settings::Flags::new(shared_builder);
    let isa_builder = isa::lookup(triple).unwrap();
    let isa = isa_builder.finish(shared_flags).unwrap();
    let obj_builder = ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap();
    let mut module = ObjectModule::new(obj_builder);

    crate::compiler::compile(instructions, &mut module);

    RSCObjectModule { product: module.finish() }
}

pub fn emit_jit_module(triple: Triple, instructions: Vec<crate::parser::Instruction>) -> RSCJITModule {
    let mut shared_builder = settings::builder();

    // Disable PIC so code can run on aarch64.
    // See: https://github.com/bytecodealliance/wasmtime/issues/2735#issuecomment-801471323
    shared_builder.set("is_pic", "false").unwrap();

    let shared_flags = settings::Flags::new(shared_builder);
    let isa_builder = isa::lookup(triple).unwrap();
    let isa = isa_builder.finish(shared_flags).unwrap();
    let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    // fill in these helper functions with rust implementations so we can actually run the code
    builder.symbol("rsc_init", rsc_init as *const u8);
    builder.symbol("rsc_rand", rsc_rand as *const u8);
    builder.symbol("rsc_out", rsc_out as *const u8);
    builder.symbol("rsc_input", rsc_input as *const u8);

    let mut module = JITModule::new(builder);
    let main_id = crate::compiler::compile(instructions, &mut module);

    module.finalize_definitions().unwrap();

    RSCJITModule { module, main_id }
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
