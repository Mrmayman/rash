use crate::{ScratchBlock, Script, builder::set, compiler::VarTypeChecked};

use super::{A, B, X, Y, assert_writes, call, custom_block, project};

fn green_flag(blocks: Vec<ScratchBlock>) -> Script {
    Script::new_green_flag(blocks)
}

#[test]
fn green_flag_contributes_call_site_to_callee() {
    let mut builder = project(vec![
        green_flag(vec![set(X, 1.0), call(A)]),
        custom_block(A, vec![]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // A has no blocks of its own, but the green flag's pre-call state
    // should still be visible as A's call-site environment.
    assert_writes(
        &analysis.custom_blocks[&A].call_site.at_site,
        &[(X, VarTypeChecked::Number)],
    );
}

#[test]
fn green_flag_call_site_propagates_transitively() {
    let mut builder = project(vec![
        green_flag(vec![set(X, 1.0), call(A)]),
        custom_block(A, vec![set(Y, true), call(B)]),
        custom_block(B, vec![]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // B sees the green flag's X *plus* A's local Y write. Verifies the
    // fixed-point pass threads green-flag state through multiple hops.
    assert_writes(
        &analysis.custom_blocks[&B].call_site.at_site,
        &[(X, VarTypeChecked::Number), (Y, VarTypeChecked::Bool)],
    );
}

#[test]
fn green_flag_writes_after_call_are_excluded() {
    let mut builder = project(vec![
        green_flag(vec![call(A), set(X, 1.0)]),
        custom_block(A, vec![]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    assert!(
        analysis.custom_blocks[&A]
            .call_site
            .at_site
            .writes
            .is_empty(),
        "writes after the call site must not leak into the callee's env"
    );
}

#[test]
fn multiple_green_flags_merge_call_sites() {
    let mut builder = project(vec![
        green_flag(vec![set(X, 1.0), call(A)]),
        green_flag(vec![set(X, true), call(A)]),
        custom_block(A, vec![]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // Two independent entry points with conflicting types must both be
    // represented; the variable downgrades to Object.
    assert_writes(
        &analysis.custom_blocks[&A].call_site.at_site,
        &[(X, VarTypeChecked::Object)],
    );
}

#[test]
fn single_green_flag_multiple_calls_merge_states() {
    let mut builder = project(vec![
        green_flag(vec![set(X, 1.0), call(A), set(Y, true), call(A)]),
        custom_block(A, vec![]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // X is written before both calls (Number); Y only before the second,
    // so it becomes optional → Object.
    assert_writes(
        &analysis.custom_blocks[&A].call_site.at_site,
        &[(X, VarTypeChecked::Number), (Y, VarTypeChecked::Object)],
    );
}

#[test]
fn green_flag_contributes_call_site_to_recursive_block() {
    let mut builder = project(vec![
        green_flag(vec![set(X, 1.0), call(A)]),
        custom_block(A, vec![call(A)]),
    ]);

    let analysis = builder.analyze_custom_blocks();

    // A is self-recursive, so it never enters `effects_by_block`…
    assert!(analysis.custom_blocks[&A].effects.is_unknown);
    // …but the green flag still supplies its call-site state.
    assert_writes(
        &analysis.custom_blocks[&A].call_site.at_site,
        &[(X, VarTypeChecked::Number)],
    );
}
