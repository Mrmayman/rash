use crate::{
    Input, ScratchBlock,
    analysis_passes::tests::{A, B, C, X, assert_writes, call, custom_block, project},
    builder::set,
    compiler::VarTypeChecked,
    effects::CallSiteInfo,
    runtime::CustomBlockId,
};

fn call_with_args(id: CustomBlockId, args: Vec<Input>) -> ScratchBlock {
    ScratchBlock::FunctionCallNoScreenRefresh(id, args)
}

fn assert_arg_types(info: &CallSiteInfo, expected: &[VarTypeChecked]) {
    let actual: Vec<_> = info.argument_types.iter().map(|w| w.ty).collect();
    assert_eq!(
        actual.as_slice(),
        expected,
        "unexpected argument types: {:?}",
        info.argument_types
    );
}

#[test]
fn call_environment_records_argument_types_for_single_call() {
    let mut builder = project(vec![
        custom_block(B, vec![]),
        custom_block(
            A,
            vec![call_with_args(
                B,
                vec![1.0.into(), "hello".into(), true.into()],
            )],
        ),
    ]);

    let (_, call_env) = builder.analyze_custom_blocks();

    assert_arg_types(
        &call_env[&B],
        &[
            VarTypeChecked::Number,
            VarTypeChecked::String,
            VarTypeChecked::Bool,
        ],
    );
}

#[test]
fn call_environment_merges_argument_types_across_call_sites() {
    let mut builder = project(vec![
        custom_block(C, vec![]),
        custom_block(
            A,
            vec![call_with_args(
                C,
                vec![1.0.into(), "hello".into(), true.into()],
            )],
        ),
        custom_block(
            B,
            vec![call_with_args(
                C,
                vec![2.0.into(), "world".into(), "text".into()],
            )],
        ),
    ]);

    let (_, call_env) = builder.analyze_custom_blocks();

    assert_arg_types(
        &call_env[&C],
        &[
            VarTypeChecked::Number,
            VarTypeChecked::String,
            VarTypeChecked::Object,
        ],
    );
}

#[test]
fn call_environment_argument_types_downgrade_on_mismatch() {
    let mut builder = project(vec![
        custom_block(C, vec![]),
        custom_block(A, vec![call_with_args(C, vec![1.0.into()])]),
        custom_block(B, vec![call_with_args(C, vec![true.into()])]),
    ]);

    let (_, call_env) = builder.analyze_custom_blocks();

    assert_arg_types(&call_env[&C], &[VarTypeChecked::Object]);
}

#[test]
fn call_environment_with_no_arguments_has_empty_argument_types() {
    let mut builder = project(vec![
        custom_block(B, vec![]),
        custom_block(A, vec![call(B)]),
    ]);

    let (_, call_env) = builder.analyze_custom_blocks();

    assert!(
        call_env[&B].argument_types.is_empty(),
        "expected no argument types, got {:?}",
        call_env[&B].argument_types
    );
}

#[test]
fn call_environment_merges_call_site_and_argument_types_independently() {
    let mut builder = project(vec![
        custom_block(C, vec![]),
        custom_block(
            A,
            vec![
                set(X, 1.0),
                call_with_args(C, vec![1.0.into(), true.into()]),
            ],
        ),
        custom_block(
            B,
            vec![
                set(X, 2.0),
                call_with_args(C, vec![2.0.into(), "text".into()]),
            ],
        ),
    ]);

    let (_, call_env) = builder.analyze_custom_blocks();

    assert_writes(&call_env[&C].call_site, &[(X, VarTypeChecked::Number)]);

    assert_arg_types(
        &call_env[&C],
        &[VarTypeChecked::Number, VarTypeChecked::Object],
    );
}
