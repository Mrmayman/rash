use super::{X, suite};
use crate::{
    Ptr, ScratchBlock, Script,
    builder::{change, fadd, set},
    runtime::CustomBlockId,
};

#[test]
fn single_numeric_argument() {
    suite(
        vec![
            Script::new_custom_block(
                vec![set(X, ScratchBlock::FunctionGetArg(0))],
                1,
                CustomBlockId(0),
                false,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![42.0.into()]),
            ]),
        ],
        |m| assert_eq!(m[0].convert_to_number(), 42.0),
        1,
    );
}

#[test]
fn multiple_numeric_arguments() {
    suite(
        vec![
            Script::new_custom_block(
                vec![
                    set(X, ScratchBlock::FunctionGetArg(0)),
                    change(X, ScratchBlock::FunctionGetArg(1)),
                ],
                2,
                CustomBlockId(0),
                false,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(
                    CustomBlockId(0),
                    vec![2.0.into(), 3.0.into()],
                ),
            ]),
        ],
        |m| assert_eq!(m[0].convert_to_number(), 5.0),
        1,
    );
}

#[test]
fn mixed_type_single_argument() {
    suite(
        vec![
            Script::new_custom_block(
                vec![change(X, ScratchBlock::FunctionGetArg(0))],
                1,
                CustomBlockId(0),
                false,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![1.0.into()]),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec!["2".into()]),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![true.into()]),
            ]),
        ],
        |m| assert_eq!(m[0].convert_to_number(), 4.0), // 1 + 2 + 1
        1,
    );
}

#[test]
fn same_arguments_repeated() {
    suite(
        vec![
            Script::new_custom_block(
                vec![change(X, ScratchBlock::FunctionGetArg(0))],
                1,
                CustomBlockId(0),
                false,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![5.0.into()]),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![5.0.into()]),
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![5.0.into()]),
            ]),
        ],
        |m| assert_eq!(m[0].convert_to_number(), 15.0),
        1,
    );
}

#[test]
fn multiple_arguments_mixed_types() {
    suite(
        vec![
            Script::new_custom_block(
                vec![
                    set(X, ScratchBlock::FunctionGetArg(0)),
                    change(X, ScratchBlock::FunctionGetArg(1)),
                    ScratchBlock::Log(ScratchBlock::VarRead(X).into()),
                    change(X, ScratchBlock::FunctionGetArg(2)),
                    ScratchBlock::Log(ScratchBlock::VarRead(X).into()),
                ],
                3,
                CustomBlockId(0),
                false,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                ScratchBlock::FunctionCallScreenRefresh(
                    CustomBlockId(0),
                    vec![1.0.into(), "2".into(), true.into()],
                ),
            ]),
        ],
        |m| assert_eq!(m[0].convert_to_number(), 4.0), // 1 + 2 + 1
        1,
    );
}

#[test]
fn argument_sources_literals_arithmetic_string_and_var() {
    suite(
        vec![
            // Custom block: X += arg0
            Script::new_custom_block(
                vec![change(X, ScratchBlock::FunctionGetArg(0))],
                1,
                CustomBlockId(0),
                false,
            ),
            Script::new_green_flag(vec![
                set(X, 0.0),
                set(Ptr(1), 100.0), // Y = 100
                // 1. Literal
                ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), vec![10.0.into()]),
                // 2. Arithmetic op: 3 + 4
                ScratchBlock::FunctionCallScreenRefresh(
                    CustomBlockId(0),
                    vec![fadd(3.0, 4.0).into()],
                ),
                // 3. String op: join "5" and "6" -> "56"
                ScratchBlock::FunctionCallScreenRefresh(
                    CustomBlockId(0),
                    vec![ScratchBlock::OpStrJoin("5".into(), "6".into()).into()],
                ),
                // 4. Reading a variable: Y = 100
                ScratchBlock::FunctionCallScreenRefresh(
                    CustomBlockId(0),
                    vec![ScratchBlock::VarRead(Ptr(1)).into()],
                ),
            ]),
        ],
        |m| {
            // 10 + (3+4) + 56 + 100 = 173
            assert_eq!(m[0].convert_to_number(), 173.0);
        },
        1,
    );
}
