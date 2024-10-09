use std::collections::HashMap;
use cranelift::prelude::*;
use cranelift_codegen::ir::{FuncRef, Function};
use cranelift_codegen::Context;
use cranelift_module::{FuncId, Linkage, Module};
use crate::parser::Instruction;

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

pub fn compile<M: Module>(instructions: Vec<crate::parser::Instruction>, module: &mut M) -> FuncId {
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

    compile_instructions(&instructions, 0, &mut program, &mut main);

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

fn compile_instructions<M: Module>(instructions: &Vec<crate::parser::Instruction>, start_line: usize, program: &mut Program<M>, main: &mut FunctionBuilder) {
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

                compile_instructions(instructions, bru.location as usize, program, main);

                break;
            },

            // Branch Positive Accumulator
            Instruction::BPA(bpa) => {
                compile_branch(
                    FloatCC::GreaterThan,  // condition
                    bpa.location,          // jump here if condition holds
                    instructions,          // list of instructions
                    program,
                    main
                );
            },

            // Branch Negative Accumulator
            Instruction::BNA(bna) => {
                compile_branch(
                    FloatCC::LessThan,     // condition
                    bna.location,          // jump here if condition holds
                    instructions,          // list of instructions
                    program,
                    main
                );
            },

            // Branch Zero Accumulator
            Instruction::BZA(bza) => {
                compile_branch(
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

fn compile_branch<M: Module>(condition: FloatCC, then_location: u32, instructions: &Vec<crate::parser::Instruction>, program: &mut Program<M>, main: &mut FunctionBuilder) {
    let accum_val = main.use_var(program.accum);
    let fzero = main.ins().f64const(0.0);
    let condition = main.ins().fcmp(condition, accum_val, fzero);

    let then_block = main.create_block();
    let else_block = main.create_block();
    let merge_block = main.create_block();

    main.ins().brif(condition, then_block, &[], else_block, &[]);

    main.switch_to_block(then_block);
    main.seal_block(then_block);
    compile_instructions(instructions, then_location as usize, program, main);

    main.switch_to_block(else_block);
    main.seal_block(else_block);
    main.ins().jump(merge_block, &[]);

    main.switch_to_block(merge_block);
    main.seal_block(merge_block);
}
