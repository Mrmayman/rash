use std::collections::HashMap;

use crate::{
    Ptr, ScratchBlock,
    builder::{c_if, fadd, fmul, set},
    compiler::VarTypeChecked,
    effects::{CheckEffects, Effects, VariableWrite},
    runtime::CustomBlockId,
};

#[must_use]
fn eff<T: CheckEffects>(input: &T) -> Effects {
    input.effects(&mut |_| Effects::unknown(), &|_| VariableWrite::default())
}

const X: Ptr = Ptr(0);
const Y: Ptr = Ptr(1);
const Z: Ptr = Ptr(2);

fn ty_map(e: &Effects) -> HashMap<Ptr, VarTypeChecked> {
    e.writes.iter().map(|(p, w)| (*p, w.ty)).collect()
}

fn assert_writes(e: &Effects, expected: &[(Ptr, VarTypeChecked)]) {
    assert_eq!(e.writes.len(), expected.len());

    let map = ty_map(e);

    for (ptr, ty) in expected {
        assert_eq!(map.get(ptr).copied(), Some(*ty));
    }
}

#[test]
fn empty_blocks() {
    let blocks: Vec<ScratchBlock> = vec![];
    let e = eff(&blocks);

    assert!(!e.is_unknown);
    assert!(!e.yields);
    assert_eq!(e.writes.len(), 0);
    assert_eq!(e.reads.len(), 0);
}

#[test]
fn overwrite_same_var() {
    let blocks = vec![set(X, 1.0), set(X, 2.0), set(X, 3.0)];

    let e = eff(&blocks);

    assert!(!e.is_unknown);
    assert_eq!(e.writes.len(), 1);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Number);
}

#[test]
fn multi_write_types() {
    let blocks = vec![set(X, 1.0), set(Y, true), set(Z, "Hello")];

    let e = eff(&blocks);

    assert_writes(
        &e,
        &[
            (X, VarTypeChecked::Number),
            (Y, VarTypeChecked::Bool),
            (Z, VarTypeChecked::String),
        ],
    );
}

#[test]
fn type_overwrite_same_var() {
    let blocks = vec![
        set(X, 1.0),
        set(X, true),    // conflict
        set(X, "hello"), // conflict again
    ];

    let e = eff(&blocks);

    assert_eq!(e.writes.len(), 1);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::String);
}

#[test]
fn branching_join_intersection_behavior() {
    #[rustfmt::skip]
    let blocks = vec![
        set(X, 1.0),
        c_if(true, vec![
            set(X, true),
            set(Y, "hello")
        ])
    ];

    let e = eff(&blocks);

    assert_eq!(e.writes.len(), 2);

    // X exists in both branches -> type collapses
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Object);

    // Y only written in branch -> defaulted merge behavior
    assert_eq!(e.writes[&Y].ty, VarTypeChecked::Object);
}

#[test]
fn if_else_merge_intersection() {
    #[rustfmt::skip]
    let blocks = vec![
        set(X, 1.0),
        c_if(true, vec![
            set(Y, true)
        ]),
    ];

    let e = eff(&blocks);

    assert_eq!(e.writes.len(), 2);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Number);
    assert_eq!(e.writes[&Y].ty, VarTypeChecked::Object);
}

#[test]
fn arithmetic_propagates_reads() {
    let blocks = vec![set(X, 1.0), fadd(X, 2.0), fmul(X, Y)];

    let e = eff(&blocks);

    assert!(e.reads.contains(&X));
    assert!(e.reads.contains(&Y));
}

#[test]
fn control_if_merges_reads_and_writes() {
    let blocks = vec![c_if(true, vec![set(X, 1.0), fadd(X, Y)])];

    let e = eff(&blocks);

    assert!(e.reads.contains(&X));
    assert!(e.reads.contains(&Y));
    assert_eq!(e.writes.len(), 1);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Object);
}

#[test]
fn control_if_else_union_vs_intersection() {
    let blocks = vec![
        ScratchBlock::ControlIfElse(true.into(), vec![set(X, 1.0)], vec![set(X, true)]),
        ScratchBlock::ControlIfElse(true.into(), vec![set(Y, 1.0)], vec![set(Y, 2.0)]),
    ];

    let e = eff(&blocks);

    assert_eq!(e.writes.len(), 2);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Object);
    assert_eq!(e.writes[&Y].ty, VarTypeChecked::Number);
}

#[test]
fn repeat_loop_marks_may_not_happen_logic() {
    let blocks = vec![ScratchBlock::ControlRepeat(10.0.into(), vec![set(X, 1.0)])];

    let e = eff(&blocks);

    assert!(!e.is_unknown);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Object);
}

#[test]
fn forever_loop_basic() {
    let blocks = vec![ScratchBlock::ControlForever(vec![set(X, 1.0)])];

    let e = eff(&blocks);

    assert_eq!(e.writes[&X].ty, VarTypeChecked::Number);
}

#[test]
fn function_call_merges_effects() {
    let blocks = vec![ScratchBlock::FunctionCallNoScreenRefresh(
        CustomBlockId(0),
        vec![X.into(), Y.into()],
    )];

    let e = eff(&blocks);

    assert!(e.is_unknown); // Test suite doesn't contain inter-function analysis
    assert!(e.reads.contains(&X));
    assert!(e.reads.contains(&Y));
}

#[test]
fn screen_refresh_sets_unknown_and_yield() {
    let blocks = vec![ScratchBlock::ScreenRefresh];

    let e = eff(&blocks);

    assert!(e.is_unknown);
    assert!(e.yields);
}
