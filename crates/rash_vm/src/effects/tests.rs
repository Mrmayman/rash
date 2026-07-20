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
    let mut other_effs: Option<Effects> = None;
    let var_ty = &|_| VariableWrite::default();
    let mut main_eff = input.effects(&mut |_| Effects::unknown(), var_ty, &mut |e| {
        if let Some(other_effs) = &mut other_effs {
            other_effs.merge(e, var_ty);
        } else {
            other_effs = Some(e);
        }
    });
    if let Some(other_effs) = other_effs {
        main_eff.merge(other_effs, var_ty);
    }
    main_eff
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

#[test]
fn stop_this_script_disagreeing_return_types_become_object() {
    #[rustfmt::skip]
    let blocks = vec![
        c_if(true, vec![
            set(X, 1.0),
            ScratchBlock::ControlStopThisScript,
        ]),

        set(X, "final"),
    ];

    let e = eff(&blocks);

    // Early return says X is Float, final return says X is String.
    // Since both are possible outcomes, the resulting type must widen.
    assert_eq!(e.writes.len(), 1);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Object);
}

#[test]
fn stop_this_script_agreeing_return_types_keep_correct_type() {
    #[rustfmt::skip]
    let blocks = vec![
        c_if(true, vec![
            set(X, "same"),
            ScratchBlock::ControlStopThisScript,
        ]),

        set(X, "same"),
    ];

    let e = eff(&blocks);

    // Both possible exits agree.
    assert_eq!(e.writes.len(), 1);
    assert_eq!(e.writes[&X].ty, VarTypeChecked::String);
}

#[test]
fn stop_this_script_multiple_returns_stress() {
    #[rustfmt::skip]
    let blocks = vec![
        set(X, 123.0),
        set(Y, "initial"),

        c_if(true, vec![
            set(X, 1.0),

            c_if(true, vec![
                set(Y, "branch"),
                ScratchBlock::ControlStopThisScript,
            ]),

            set(Z, true),

            c_if(true, vec![
                set(X, false), // conflicts with every other X write
                ScratchBlock::ControlStopThisScript,
            ]),
        ]),

        c_if(true, vec![
            set(Y, "final"),

            c_if(true, vec![
                set(Z, "final conflict"),
                ScratchBlock::ControlStopThisScript,
            ]),
        ]),

        set(X, "final"),
        set(Y, "final"),
        set(Z, 999.0),
    ];

    let e = eff(&blocks);

    assert_eq!(e.writes.len(), 3);

    // X:
    // - initial Float
    // - nested early return Float
    // - nested early return Bool
    // - final String
    assert_eq!(e.writes[&X].ty, VarTypeChecked::Object);

    // Y:
    // - initial String
    // - early return String
    // - final String
    assert_eq!(e.writes[&Y].ty, VarTypeChecked::String);

    // Z:
    // - early path Bool
    // - another early path String
    // - final Float
    assert_eq!(e.writes[&Z].ty, VarTypeChecked::Object);
}
