//! Callbacks for JIT code to call into Rust code.
//!
//! These functions are called by JIT code to perform
//! operations that are not possible in JIT code.

use std::{collections::HashMap, sync::LazyLock};

use cranelift::codegen::ir::UserExternalName;

macro_rules! print_func {
    ($module:expr, $($fn:ident),+ $(,)?) => {
        pub fn print_function_addresses() {
            println!("\n========");
            println!("{}", $module);
            println!("========");

            paste::paste! {
                $(
                    // println!(
                    //     "{:20} {:20} = {:#018x}",
                    //     [<$fn:upper>],
                    //     stringify!($fn),
                    //     $fn as *const () as usize,
                    // );
                    println!(
                        "{:<6} {}",
                        format!("{}", [<$fn:upper>]),
                        stringify!($fn)
                    );
                )+
            }
        }
    };
}

macro_rules! declare_module {
    ($file:literal, $namespace:expr, $($func:ident),+ $(,)?) => {
        paste::paste! {
            print_func!(
                $file,
                $($func),*,
            );

            const fn name(n: u32) -> cranelift::codegen::ir::UserExternalName {
                cranelift::codegen::ir::UserExternalName {
                    namespace: $namespace,
                    index: n,
                }
            }

            pub const FUNCS: &[(cranelift::codegen::ir::UserExternalName, *const ())] = &[
                $(
                    ([<$func:upper>], $func as *const ()),
                )*
            ];

            declare_module!(@consts 0; $($func),*);
        }
    };

    (@consts $n:expr; ) => {};

    (@consts $n:expr; $func:ident $(, $rest:ident)*) => {
        paste::paste! {
            pub const [<$func:upper>]: cranelift::codegen::ir::UserExternalName = name($n);
        }

        declare_module!(@consts ($n + 1); $($rest),*);
    };
}

pub mod custom_block;
pub mod env;
pub mod op;
pub mod repeat_stack;
pub mod types;

pub const FUNCS: LazyLock<HashMap<UserExternalName, *const ()>> = LazyLock::new(|| {
    let mut funcs = HashMap::new();
    funcs.extend(custom_block::FUNCS.iter().cloned());
    funcs.extend(env::FUNCS.iter().cloned());
    funcs.extend(op::FUNCS.iter().cloned());
    funcs.extend(repeat_stack::FUNCS.iter().cloned());
    funcs.extend(types::FUNCS.iter().cloned());
    funcs
});

pub fn print_function_addresses() {
    custom_block::print_function_addresses();
    env::print_function_addresses();
    op::print_function_addresses();
    repeat_stack::print_function_addresses();
    types::print_function_addresses();
}
