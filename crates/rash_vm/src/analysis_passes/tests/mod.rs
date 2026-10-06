use std::collections::HashMap;

use rash_core::SpriteId;

use crate::{
    MEMORY, ProjectBuilder, Ptr, ScratchBlock, SpriteBuilder,
    builder::set,
    compiler::VarTypeChecked,
    effects::Effects,
    runtime::{CustomBlockId, Script},
};

const X: Ptr = Ptr(0);
const Y: Ptr = Ptr(1);
const Z: Ptr = Ptr(2);

const A: CustomBlockId = CustomBlockId(0);
const B: CustomBlockId = CustomBlockId(1);
const C: CustomBlockId = CustomBlockId(2);

// Btw big warning: if you're defining custom blocks, don't jump
// straight from A to C (in definitions) with gap in the ID (B). Else it will panic.

mod args;

/// Adjust this helper to the actual custom-block-call variant in
/// ScratchBlock. Keeping it here means the tests below don't depend
/// on the exact AST spelling.
fn call(id: CustomBlockId) -> ScratchBlock {
    ScratchBlock::FunctionCallNoScreenRefresh(id, Vec::new())
}

fn ty_map(e: &Effects) -> HashMap<Ptr, VarTypeChecked> {
    e.writes.iter().map(|(p, w)| (*p, w.ty)).collect()
}

fn assert_writes(e: &Effects, expected: &[(Ptr, VarTypeChecked)]) {
    assert_eq!(
        e.writes.len(),
        expected.len(),
        "unexpected number of writes: {:?}",
        e.writes
    );

    let map = ty_map(e);
    assert!(!e.is_unknown);

    for (ptr, ty) in expected {
        assert_eq!(
            map.get(ptr).copied(),
            Some(*ty),
            "unexpected type for {:?}",
            ptr
        );
    }
}

fn custom_block(id: CustomBlockId, blocks: Vec<ScratchBlock>) -> Script {
    Script::new_custom_block(blocks, 0, id, false)
}

/// Build a ProjectBuilder containing custom blocks.
///
/// If your ProjectBuilder requires the scripts to be attached to a
/// particular sprite/runtime structure, this is the only helper that
/// should need adapting.
fn project(custom_blocks: Vec<Script>) -> ProjectBuilder {
    let memory = MEMORY.lock().unwrap();

    let mut sprite = SpriteBuilder::new(SpriteId(0));

    for script in custom_blocks {
        sprite.add_script(script, &memory);
    }

    let mut builder = ProjectBuilder::new();
    builder.add_sprite(sprite);

    builder
}

// ---------------------------------------------------------------------
// Direct effects
// ---------------------------------------------------------------------

#[test]
fn custom_block_empty() {
    let mut builder = project(vec![custom_block(A, vec![])]);

    let analysis = builder.analyze_custom_blocks();

    let a = &analysis[&A].0;

    assert!(!a.is_unknown);
    assert!(a.is_fully_direct());
    assert!(!a.yields);
    assert!(a.writes.is_empty());
    assert!(a.reads.is_empty());

    for (_, callsite) in analysis.values() {
        assert!(callsite.call_site.is_unknown);
        assert!(callsite.argument_types.is_empty());
    }
}

#[test]
fn custom_block_direct_write() {
    let mut builder = project(vec![custom_block(A, vec![set(X, 1.0)])]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Number)]);

    assert!(analysis[&A].0.is_fully_direct());
}

#[test]
fn custom_block_multiple_direct_writes() {
    let mut builder = project(vec![custom_block(
        A,
        vec![set(X, 1.0), set(Y, true), set(Z, "hello")],
    )]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(
        &analysis[&A].0,
        &[
            (X, VarTypeChecked::Number),
            (Y, VarTypeChecked::Bool),
            (Z, VarTypeChecked::String),
        ],
    );
}

// ---------------------------------------------------------------------
// Propagation through calls
// ---------------------------------------------------------------------

#[test]
fn caller_inherits_called_block_effects() {
    let mut builder = project(vec![
        // A writes X.
        custom_block(A, vec![set(X, 1.0)]),
        // B calls A.
        custom_block(B, vec![call(A)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Number)]);

    assert_writes(&analysis[&B].0, &[(X, VarTypeChecked::Number)]);
}

#[test]
fn transitive_effects_propagate() {
    let mut builder = project(vec![
        // C writes X.
        custom_block(C, vec![set(X, 1.0)]),
        // B calls C.
        custom_block(B, vec![call(C)]),
        // A calls B.
        custom_block(A, vec![call(B)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&C].0, &[(X, VarTypeChecked::Number)]);

    assert_writes(&analysis[&B].0, &[(X, VarTypeChecked::Number)]);

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Number)]);
}

#[test]
fn direct_and_called_effects_are_merged() {
    let mut builder = project(vec![
        // B writes X.
        custom_block(B, vec![set(X, 1.0)]),
        // A writes Y and calls B.
        custom_block(A, vec![set(Y, true), call(B)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(
        &analysis[&A].0,
        &[(X, VarTypeChecked::Number), (Y, VarTypeChecked::Bool)],
    );
}

// ---------------------------------------------------------------------

/// Multiple Callers
#[test]
fn called_block_has_multiple_callers() {
    let mut builder = project(vec![
        // C writes X.
        custom_block(C, vec![set(X, 1.0)]),
        // A calls C.
        custom_block(A, vec![call(C)]),
        // B also calls C.
        custom_block(B, vec![call(C)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Number)]);

    assert_writes(&analysis[&B].0, &[(X, VarTypeChecked::Number)]);

    assert_writes(&analysis[&C].0, &[(X, VarTypeChecked::Number)]);
}

// ---------------------------------------------------------------------
// Fixed-point / recursive call graph
// Disabled due to lack of recursion support
// ---------------------------------------------------------------------

/*#[test]
fn mutually_recursive_custom_blocks_converge() {
    let mut builder = project(vec![
        // A writes X and calls B.
        custom_block(A, vec![set(X, 1.0), call(B)]),
        // B writes Y and calls A.
        custom_block(B, vec![set(Y, true), call(A)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // The analysis must reach a fixed point instead of looping forever.
    assert_writes(
        &analysis[&A].0,
        &[(X, VarTypeChecked::Number), (Y, VarTypeChecked::Bool)],
    );

    assert_writes(
        &analysis[&B].0,
        &[(X, VarTypeChecked::Number), (Y, VarTypeChecked::Bool)],
    );
}*/

/*#[test]
fn self_recursive_custom_block_converges() {
    let mut builder = project(vec![custom_block(A, vec![set(X, 1.0), call(A)])]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Number)]);
}*/

// ---------------------------------------------------------------------
// Call environments
// ---------------------------------------------------------------------

#[test]
fn call_environment_contains_effects_at_call_site() {
    let mut builder = project(vec![
        custom_block(B, vec![set(X, 1.0)]),
        custom_block(A, vec![set(Y, 1.0), call(B)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(
        analysis.get(&B).is_some_and(|n| !n.1.call_site.is_unknown),
        "B should have a call environment"
    );

    assert_writes(&analysis[&B].1.call_site, &[(Y, VarTypeChecked::Number)]);
}

#[test]
fn call_environment_merges_multiple_call_sites() {
    let mut builder = project(vec![
        custom_block(C, vec![]),
        custom_block(A, vec![set(X, 1.0), set(Y, 0.0), call(C)]),
        custom_block(B, vec![set(X, 2.0), set(Y, true), set(Z, true), call(C)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(
        analysis.get(&C).is_some_and(|n| !n.1.call_site.is_unknown),
        "C should have a call environment"
    );

    assert_writes(
        &analysis[&C].1.call_site,
        &[
            (X, VarTypeChecked::Number),
            (Y, VarTypeChecked::Object),
            (Z, VarTypeChecked::Object),
        ],
    );
}

/// Call environment with a local write before the call
#[test]
fn call_environment_represents_state_before_call() {
    let mut builder = project(vec![
        custom_block(B, vec![]),
        custom_block(A, vec![set(X, 123.0), set(Y, true), call(B)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(
        &analysis[&B].1.call_site,
        &[(X, VarTypeChecked::Number), (Y, VarTypeChecked::Bool)],
    );
}

// ---------------------------------------------------------------------
// Regression test: called block's result must become known after
// the fixed-point iteration, rather than permanently remaining
// Effects::unknown().
// ---------------------------------------------------------------------

#[test]
fn called_block_is_reanalyzed_after_callee_changes() {
    let mut builder = project(vec![
        // B has the information that eventually needs to propagate.
        custom_block(B, vec![set(Z, "hello")]),
        // A depends on B.
        custom_block(A, vec![call(B)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(Z, VarTypeChecked::String)]);
}

// ---------------------------------------------------------------------
// Additional analyze_custom_blocks coverage
// ---------------------------------------------------------------------

#[test]
fn called_effects_are_indirect_in_caller() {
    let mut builder = project(vec![
        custom_block(A, vec![set(X, 1.0)]),
        custom_block(B, vec![call(A)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(analysis[&A].0.writes[&X].direct);
    assert!(!analysis[&B].0.writes[&X].direct);
    assert!(!analysis[&B].0.is_fully_direct());
}

#[test]
fn yielding_callee_makes_caller_yield() {
    let mut builder = project(vec![
        custom_block(A, vec![ScratchBlock::ScreenRefresh]),
        custom_block(B, vec![call(A)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(analysis[&A].0.yields);
    assert!(analysis[&A].0.is_unknown);

    assert!(analysis[&B].0.yields);
    assert!(analysis[&B].0.is_unknown);
}

#[test]
fn call_environment_excludes_writes_after_call() {
    let mut builder = project(vec![
        custom_block(B, vec![]),
        custom_block(A, vec![call(B), set(X, 1.0)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(analysis[&B].1.call_site.writes.is_empty());
}

#[test]
fn if_else_writes_same_type_preserved() {
    let mut builder = project(vec![custom_block(
        A,
        vec![ScratchBlock::ControlIfElse(
            true.into(),
            vec![set(X, 1.0)],
            vec![set(X, 2.0)],
        )],
    )]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Number)]);
}

#[test]
fn if_without_else_makes_write_optional() {
    let mut builder = project(vec![custom_block(
        A,
        vec![ScratchBlock::ControlIf(true.into(), vec![set(X, 1.0)])],
    )]);

    let analysis = builder.analyze_custom_blocks();

    // X may be written as Number, or it may keep its previous value (Object).
    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Object)]);
}

#[test]
fn if_else_writes_different_types_downgrade_to_object() {
    let mut builder = project(vec![custom_block(
        A,
        vec![ScratchBlock::ControlIfElse(
            true.into(),
            vec![set(X, 1.0)],
            vec![set(X, true)],
        )],
    )]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(&analysis[&A].0, &[(X, VarTypeChecked::Object)]);
}

#[test]
fn if_else_merges_distinct_writes_as_optional() {
    let mut builder = project(vec![custom_block(
        A,
        vec![ScratchBlock::ControlIfElse(
            true.into(),
            vec![set(X, 1.0)],
            vec![set(Y, true)],
        )],
    )]);

    let analysis = builder.analyze_custom_blocks();

    assert_writes(
        &analysis[&A].0,
        &[(X, VarTypeChecked::Object), (Y, VarTypeChecked::Object)],
    );
}
