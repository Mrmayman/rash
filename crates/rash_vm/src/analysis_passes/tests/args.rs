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

    let analysis = builder.analyze_custom_blocks();

    assert_arg_types(
        &analysis[&B].1,
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

    let analysis = builder.analyze_custom_blocks();

    assert_arg_types(
        &analysis[&C].1,
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

    let analysis = builder.analyze_custom_blocks();

    assert_arg_types(&analysis[&C].1, &[VarTypeChecked::Object]);
}

#[test]
fn call_environment_with_no_arguments_has_empty_argument_types() {
    let mut builder = project(vec![
        custom_block(B, vec![]),
        custom_block(A, vec![call(B)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(
        analysis[&B].1.argument_types.is_empty(),
        "expected no argument types, got {:?}",
        analysis[&B].1.argument_types
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

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&C].1.call_site, &[(X, VarTypeChecked::Number)]);

    assert_arg_types(
        &analysis[&C].1,
        &[VarTypeChecked::Number, VarTypeChecked::Object],
    );
}

#[test]
fn call_environment_argument_types_are_not_polluted_by_parent_context() {
    // A writes X, then calls B. B calls C with one Bool argument.
    // C's argument_types should be exactly [Bool], not affected by
    // the X write propagating through the call-site context.
    let mut builder = project(vec![
        custom_block(C, vec![]),
        custom_block(B, vec![call_with_args(C, vec![true.into()])]),
        custom_block(A, vec![set(X, 1.0), call_with_args(B, vec![1.0.into()])]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // B's arg types come from A's call to B.
    assert_arg_types(&analysis[&B].1, &[VarTypeChecked::Number]);
    // C's arg types come from B's call to C. A's context must not leak in.
    assert_arg_types(&analysis[&C].1, &[VarTypeChecked::Bool]);
}

#[test]
fn call_environment_argument_types_merge_across_depths() {
    // Two callers at different depths call C: A directly, and B (which
    // is called by A). Their arg types must merge.
    let mut builder = project(vec![
        custom_block(C, vec![]),
        custom_block(B, vec![call_with_args(C, vec!["from_b".into()])]),
        custom_block(
            A,
            vec![
                call_with_args(B, Vec::new()),
                call_with_args(C, vec![1.0.into()]),
            ],
        ),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // Number from A's direct call merges with String from B's call.
    assert_arg_types(&analysis[&C].1, &[VarTypeChecked::Object]);
}

#[test]
fn recursive_block_has_no_argument_types_recorded() {
    let mut builder = project(vec![custom_block(
        A,
        vec![call_with_args(A, vec![1.0.into()])],
    )]);

    let analysis = builder.analyze_custom_blocks();

    assert!(!analysis.contains_key(&A));
}
