use cranelift::prelude::{FunctionBuilder, InstBuilder};

use crate::{
    compiler::Compiler,
    data_types::ScratchObject,
    input_primitives::{Input, Ptr, ScratchValue},
    variable_storage::VariableSlot,
};

impl Compiler<'_> {
    pub fn var_read(&mut self, builder: &mut FunctionBuilder<'_>, ptr: Ptr) -> ScratchValue {
        self.vars
            .get(ptr, builder, &mut self.constants)
            .val
            .clone_in_code(self, builder)
    }

    pub fn var_set(&mut self, obj: &Input, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
        match obj {
            Input::Obj(obj) => {
                self.ins_drop_obj(builder, ptr);
                match obj {
                    ScratchObject::Number(num) => {
                        self.vars
                            .store_f64(ptr, builder, *num, &mut self.constants);
                    }
                    ScratchObject::Bool(num) => {
                        self.vars
                            .store_bool(ptr, builder, *num, &mut self.constants);
                    }
                    ScratchObject::String(string) => {
                        let num = obj.convert_to_number();
                        if ScratchObject::Number(num).convert_to_string() == *string {
                            // TODO: This is a very opinionated optimization
                            // Fast for number-crunching but slow for string handling?
                            self.vars.store_f64(ptr, builder, num, &mut self.constants);
                        } else {
                            let to_drop =
                                self.vars
                                    .store_string(ptr, builder, string, &mut self.constants);
                            self.static_strings.push(to_drop);
                        }
                    }
                }
            }
            Input::Block(block) => {
                let val = self.compile_block(block, builder)
                    .expect("blocks inside other blocks (like an add operator in a set var block) should return something!");

                self.ins_drop_obj(builder, ptr);
                self.vars.store(
                    ptr,
                    VariableSlot {
                        val,
                        skip_nan: block
                            .return_type(|n| self.vars.get_type(n))
                            .is_some_and(|n| n.skip_nan),
                    },
                    builder,
                    &mut self.constants,
                );
            }
        }
    }

    pub fn var_change(&mut self, input: &Input, builder: &mut FunctionBuilder<'_>, ptr: Ptr) {
        let input = input.get_number(self, builder);
        let old_value = self.var_read(builder, ptr).get_number(self, builder);
        let new_value = builder.ins().fadd(old_value, input);

        self.vars.store(
            ptr,
            VariableSlot {
                val: ScratchValue::Num(new_value),
                skip_nan: true,
            },
            builder,
            &mut self.constants,
        );
    }
}
