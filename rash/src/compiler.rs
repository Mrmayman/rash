use std::{collections::HashMap, sync::Mutex};

use codegen::control::ControlPlane;
use codegen::ir::{Function, UserFuncName};
use cranelift::prelude::*;
use isa::CallConv;
use lazy_static::lazy_static;
use target_lexicon::Triple;
use types::{F64, I64};

use crate::{
    block_test, blocks, callbacks,
    constant_set::ConstantMap,
    data_types::ScratchObject,
    input_primitives::{Input, Ptr, ReturnValue},
    stack_cache::StackCache,
};

lazy_static! {
    pub static ref MEMORY: Mutex<Box<[ScratchObject]>> =
        Mutex::new(vec![ScratchObject::Number(0.0); 1024].into_boxed_slice());
}

#[allow(unused)]
#[derive(Debug)]
pub enum ScratchBlock {
    WhenFlagClicked,
    VarSet(Ptr, Input),
    VarChange(Ptr, Input),
    VarRead(Ptr),
    OpAdd(Input, Input),
    OpSub(Input, Input),
    OpMul(Input, Input),
    OpDiv(Input, Input),
    OpRound(Input),
    OpStrJoin(Input, Input),
    OpMod(Input, Input),
    OpStrLen(Input),
    OpBAnd(Input, Input),
    OpBNot(Input),
    OpBOr(Input, Input),
    OpMFloor(Input),
    OpMAbs(Input),
    OpMSqrt(Input),
    OpMSin(Input),
    OpMCos(Input),
    OpMTan(Input),
    OpCmpGreater(Input, Input),
    OpCmpLesser(Input, Input),
    OpRandom(Input, Input),
    OpStrLetterOf(Input, Input),
    OpStrContains(Input, Input),
    ControlIf(Input, Vec<ScratchBlock>),
    ControlIfElse(Input, Vec<ScratchBlock>, Vec<ScratchBlock>),
    ControlRepeat(Input, Vec<ScratchBlock>),
    ControlRepeatUntil(Input, Vec<ScratchBlock>),
    ScreenRefresh,
}

#[derive(PartialEq, Eq, Debug)]
pub enum VarTypeChecked {
    Number,
    Bool,
    String,
    Unknown,
}

impl From<VarType> for VarTypeChecked {
    fn from(value: VarType) -> Self {
        match value {
            VarType::Number => Self::Number,
            VarType::Bool => Self::Bool,
            VarType::String => Self::String,
        }
    }
}

impl ScratchBlock {
    pub fn return_type(
        &self,
        variable_type_data: &HashMap<Ptr, VarType>,
    ) -> Option<VarTypeChecked> {
        match self {
            ScratchBlock::VarRead(ptr) => match variable_type_data.get(ptr) {
                Some(vartype) => Some((*vartype).into()),
                None => Some(VarTypeChecked::Unknown),
            },
            ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpRandom(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::OpStrLen(_) => Some(VarTypeChecked::Number),
            ScratchBlock::OpStrLetterOf(_, _) | ScratchBlock::OpStrJoin(_, _) => {
                Some(VarTypeChecked::String)
            }
            ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpCmpGreater(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::OpCmpLesser(_, _) => Some(VarTypeChecked::Bool),
            ScratchBlock::WhenFlagClicked
            | ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::ControlRepeatUntil(_, _) => None,
        }
    }

    pub fn affects_var(
        &self,
        var_ptr: Ptr,
        variable_type_data: &HashMap<Ptr, VarType>,
    ) -> Option<VarTypeChecked> {
        match self {
            ScratchBlock::VarSet(ptr, input) => {
                if var_ptr == *ptr {
                    match input {
                        Input::Obj(scratch_object) => Some(scratch_object.get_type().into()),
                        Input::Block(scratch_block) => {
                            scratch_block.return_type(variable_type_data)
                        }
                    }
                } else {
                    None
                }
            }
            ScratchBlock::VarChange(ptr, _) => {
                if var_ptr == *ptr {
                    Some(VarTypeChecked::Number)
                } else {
                    None
                }
            }
            ScratchBlock::ControlIf(_, vec)
            | ScratchBlock::ControlRepeat(_, vec)
            | ScratchBlock::ControlRepeatUntil(_, vec) => vec
                .iter()
                .filter_map(|n| n.affects_var(var_ptr, variable_type_data))
                .last(),
            ScratchBlock::ControlIfElse(_, then, else_block) => {
                let then = then
                    .iter()
                    .map(|n| n.affects_var(var_ptr, variable_type_data))
                    .last()
                    .flatten();
                let else_block = else_block
                    .iter()
                    .map(|n| n.affects_var(var_ptr, variable_type_data))
                    .last()
                    .flatten();

                match (then, else_block) {
                    (None, None) => None,
                    (None, Some(n)) | (Some(n), None) => Some(n),
                    (Some(a), Some(b)) => (a == b).then_some(a),
                }
            }
            _ => None,
        }
    }

    pub fn could_be_nan(&self) -> bool {
        match self {
            ScratchBlock::WhenFlagClicked
            | ScratchBlock::VarSet(_, _)
            | ScratchBlock::VarChange(_, _)
            | ScratchBlock::ControlIf(_, _)
            | ScratchBlock::ControlIfElse(_, _, _)
            | ScratchBlock::ControlRepeat(_, _)
            | ScratchBlock::ControlRepeatUntil(_, _)
            | ScratchBlock::OpAdd(_, _)
            | ScratchBlock::OpSub(_, _)
            | ScratchBlock::OpMul(_, _)
            | ScratchBlock::OpStrJoin(_, _)
            | ScratchBlock::OpStrLen(_)
            | ScratchBlock::OpCmpGreater(_, _)
            | ScratchBlock::OpBAnd(_, _)
            | ScratchBlock::OpBNot(_)
            | ScratchBlock::OpBOr(_, _)
            | ScratchBlock::OpMFloor(_)
            | ScratchBlock::OpMAbs(_)
            | ScratchBlock::OpStrLetterOf(_, _)
            | ScratchBlock::OpStrContains(_, _)
            | ScratchBlock::OpRound(_)
            | ScratchBlock::OpMSin(_)
            | ScratchBlock::OpMCos(_)
            | ScratchBlock::OpMTan(_)
            | ScratchBlock::ScreenRefresh
            | ScratchBlock::OpCmpLesser(_, _) => false,
            ScratchBlock::VarRead(_)
            | ScratchBlock::OpDiv(_, _)
            | ScratchBlock::OpMod(_, _)
            | ScratchBlock::OpMSqrt(_)
            | ScratchBlock::OpRandom(_, _) => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Number,
    Bool,
    String,
}

pub struct CodeSprite {
    pub scripts: Vec<Vec<ScratchBlock>>,
}

pub fn compile(/*&self*/) {
    let mut builder = settings::builder();
    builder.set("opt_level", "speed").unwrap();
    // for setting in builder.iter() {
    //     println!("{setting:?}");
    // }
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {err}"),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };

    // let code_sprites = self.get_block_code();
    let code_sprites = vec![CodeSprite {
        scripts: vec![block_test::pi()],
    }];
    for sprite in &code_sprites {
        for script in &sprite.scripts {
            let mut sig = Signature::new(CallConv::SystemV);
            sig.params.push(AbiParam::new(I64));
            sig.params.push(AbiParam::new(I64));
            sig.returns.push(AbiParam::new(I64));
            let mut func = Function::with_name_signature(UserFuncName::default(), sig);

            let mut func_ctx = FunctionBuilderContext::new();
            // let func_ptr: *mut Function = &mut func;
            // let func_ptr = unsafe { &mut *func_ptr };
            let func_ptr = &mut func;
            let mut builder = FunctionBuilder::new(func_ptr, &mut func_ctx);

            let mut jmp1_block = builder.create_block();
            builder.append_block_param(jmp1_block, I64);

            let jmp2_block = builder.create_block();
            builder.append_block_params_for_function_params(jmp2_block);
            builder.switch_to_block(jmp2_block);
            let param = builder.block_params(jmp2_block)[0];
            let repeat_stack_ptr = builder.block_params(jmp2_block)[1];
            builder.ins().jump(jmp1_block, &[param]);

            let code_block = builder.create_block();
            builder.switch_to_block(code_block);

            let lock = MEMORY.lock().unwrap();

            let mut compiler =
                Compiler::new(code_block, &mut builder, script, &lock, repeat_stack_ptr);

            compiler
                .cache
                .init(&mut builder, &lock, &mut compiler.constants);

            compiler.break_points.push(code_block);

            for block in script {
                compiler.compile_block(block, &mut builder);
            }

            compiler
                .cache
                .save(&mut builder, &mut compiler.constants, &lock);

            let return_value = compiler.constants.get_int(-1, &mut builder);
            builder.ins().return_(&[return_value]);

            for (i, point) in compiler.break_points.iter().enumerate() {
                builder.switch_to_block(jmp1_block);
                let param = builder.block_params(jmp1_block)[0];
                let cmp = builder.ins().icmp_imm(IntCC::Equal, param, i as i64);
                jmp1_block = builder.create_block();
                builder.append_block_param(jmp1_block, I64);
                builder.ins().brif(cmp, *point, &[], jmp1_block, &[param]);
            }

            builder.switch_to_block(jmp1_block);
            let return_value = builder.ins().iconst(I64, -1);
            builder.ins().return_(&[return_value]);

            builder.seal_all_blocks();

            builder.finalize();

            println!("{}", func.display());

            let mut ctx = codegen::Context::for_function(func);
            let mut plane = ControlPlane::default();
            let isa = isa.as_ref();
            ctx.optimize(isa, &mut plane).unwrap();

            let code = ctx.compile(&*isa, &mut plane).unwrap();

            let mut buffer = memmap2::MmapOptions::new()
                .len(code.code_buffer().len())
                .map_anon()
                .unwrap();

            buffer.copy_from_slice(code.code_buffer());

            // Machine code dump
            // let ptr = buffer.as_ptr();
            // let bytes = unsafe { std::slice::from_raw_parts(ptr, code.code_buffer().len()) };
            // for (_i, byte) in bytes.iter().enumerate() {
            //     print!("{:#04x} ", byte);
            // }
            // println!();
            // std::fs::write("func.bin", bytes).unwrap();

            let buffer = buffer.make_exec().unwrap();

            unsafe {
                let code_fn: unsafe extern "sysv64" fn(i64, *mut Vec<i64>) -> i64 =
                    std::mem::transmute(buffer.as_ptr());

                let instant = std::time::Instant::now();
                let mut stack: Vec<i64> = Vec::new();

                let mut result = 0;
                while result != -1 {
                    result = code_fn(result, &mut stack);
                    println!("Iteration");
                }
                println!("Time: {:?}", instant.elapsed());
                // println!("Types: {:?}", compiler.variable_type_data);
                // println!("Memory ptr {:X}", lock.as_ptr() as usize);
            }
        }
    }
}

pub struct Compiler<'compiler> {
    pub variable_type_data: HashMap<Ptr, VarType>,
    pub constants: ConstantMap,
    pub code_block: Block,
    pub cache: StackCache,
    pub break_counter: usize,
    pub break_points: Vec<Block>,
    pub memory: &'compiler [ScratchObject],
    pub vec_ptr: Value,
}

impl<'a> Compiler<'a> {
    pub fn new(
        block: Block,
        builder: &mut FunctionBuilder<'_>,
        code: &[ScratchBlock],
        memory: &'a [ScratchObject],
        vec_ptr: Value,
    ) -> Self {
        Self {
            variable_type_data: HashMap::new(),
            constants: ConstantMap::new(),
            code_block: block,
            cache: StackCache::new(builder, code),
            break_points: Vec::new(),
            break_counter: 0,
            memory,
            vec_ptr,
        }
    }

    pub fn compile_block(
        &mut self,
        block: &ScratchBlock,
        builder: &mut FunctionBuilder<'_>,
    ) -> Option<ReturnValue> {
        match block {
            ScratchBlock::WhenFlagClicked => {}
            ScratchBlock::VarSet(ptr, obj) => {
                blocks::var::set(self, obj, builder, *ptr);
            }
            ScratchBlock::OpAdd(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fadd(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpSub(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fsub(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpMul(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fmul(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpDiv(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fdiv(a, b);
                return Some(ReturnValue::Num(res));
            }
            ScratchBlock::OpMod(a, b) => {
                let modulo = blocks::op::modulo(self, a, b, builder);
                return Some(ReturnValue::Num(modulo));
            }
            ScratchBlock::VarRead(ptr) => {
                return Some(blocks::var::read(self, builder, *ptr));
            }
            ScratchBlock::OpStrJoin(a, b) => {
                let obj = blocks::op::str_join(self, a, b, builder);
                return Some(ReturnValue::Object(obj));
            }
            ScratchBlock::ControlRepeat(input, vec) => {
                blocks::control::repeat(self, builder, input, vec);
            }
            ScratchBlock::VarChange(ptr, input) => {
                blocks::var::change(self, input, builder, *ptr);
            }
            ScratchBlock::ControlIf(input, vec) => {
                blocks::control::if_statement(self, input, builder, vec);
            }
            ScratchBlock::ControlIfElse(condition, then, r#else) => {
                blocks::control::if_else(self, condition, builder, then, r#else);
            }
            ScratchBlock::ControlRepeatUntil(input, vec) => {
                blocks::control::repeat_until(self, builder, input, vec);
            }
            ScratchBlock::OpCmpGreater(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fcmp(FloatCC::GreaterThan, a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpCmpLesser(a, b) => {
                let a = a.get_number(self, builder);
                let b = b.get_number(self, builder);
                let res = builder.ins().fcmp(FloatCC::LessThan, a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpStrLen(input) => {
                return Some(blocks::op::str_len(self, input, builder));
            }
            ScratchBlock::OpRandom(a, b) => return Some(blocks::op::random(self, a, b, builder)),
            ScratchBlock::OpBAnd(a, b) => {
                let a = a.get_bool(self, builder);
                let b = b.get_bool(self, builder);
                let res = builder.ins().band(a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpBNot(a) => {
                let a = a.get_bool(self, builder);
                let res = builder.ins().bnot(a);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpBOr(a, b) => {
                let a = a.get_bool(self, builder);
                let b = b.get_bool(self, builder);
                let res = builder.ins().bor(a, b);
                return Some(ReturnValue::Bool(res));
            }
            ScratchBlock::OpMFloor(n) => return Some(blocks::op::m_floor(self, n, builder)),
            ScratchBlock::OpStrLetterOf(letter, string) => {
                return Some(ReturnValue::Object(blocks::op::str_letter(
                    self, letter, string, builder,
                )))
            }
            ScratchBlock::OpStrContains(string, pattern) => {
                return Some(ReturnValue::Bool(blocks::op::str_contains(
                    self, string, pattern, builder,
                )))
            }
            ScratchBlock::OpRound(num) => {
                return Some(ReturnValue::Num(blocks::op::round(self, num, builder)))
            }
            ScratchBlock::OpMAbs(num) => {
                let num = num.get_number(self, builder);
                let abs = builder.ins().fabs(num);
                return Some(ReturnValue::Num(abs));
            }
            ScratchBlock::OpMSqrt(num) => {
                let num = num.get_number(self, builder);
                let sqrt = builder.ins().sqrt(num);
                return Some(ReturnValue::Num(sqrt));
            }
            ScratchBlock::OpMSin(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op_sin as usize, &[F64], &[F64], &[num]);
                let result = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(result));
            }
            ScratchBlock::OpMCos(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op_cos as usize, &[F64], &[F64], &[num]);
                let result = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(result));
            }
            ScratchBlock::OpMTan(num) => {
                let num = num.get_number(self, builder);
                let inst =
                    self.call_function(builder, callbacks::op_tan as usize, &[F64], &[F64], &[num]);
                let result = builder.inst_results(inst)[0];
                return Some(ReturnValue::Num(result));
            }
            ScratchBlock::ScreenRefresh => {
                self.break_counter += 1;
                self.cache.save(builder, &mut self.constants, self.memory);
                let break_counter = self.constants.get_int(self.break_counter as i64, builder);

                builder.ins().return_(&[break_counter]);
                self.constants.clear();
                self.code_block = builder.create_block();
                self.break_points.push(self.code_block);
                builder.switch_to_block(self.code_block);

                self.cache.init(builder, self.memory, &mut self.constants);
            }
        }
        None
    }
}

pub fn print_func_addresses() {
    println!("var_read: {:X}", callbacks::var_read as usize);
    println!("op_str_join: {:X}", callbacks::op_str_join as usize);
    println!("f64::floor: {:X}", f64::floor as usize);
    println!(
        "to_string_from_num: {:X}",
        callbacks::types::to_string_from_num as usize
    );
    println!("to_string: {:X}", callbacks::types::to_string as usize);
    println!(
        "to_string_from_bool: {:X}",
        callbacks::types::to_string_from_bool as usize
    );
    println!("to_number: {:X}", callbacks::types::to_number as usize);
    println!("drop_obj: {:X}", callbacks::types::drop_obj as usize);
}
