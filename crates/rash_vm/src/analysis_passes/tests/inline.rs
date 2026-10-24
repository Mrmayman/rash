use super::*;
use crate::{
    Input,
    analysis_passes::inline_inner,
    runtime::{CustomBlock, MaybeCompiled, Script},
};
use std::collections::HashSet;

fn arg(n: usize) -> ScratchBlock {
    ScratchBlock::FunctionGetArg(n)
}
fn local(n: usize) -> ScratchBlock {
    ScratchBlock::LocalsGet(n)
}
fn call_nr<T: Into<Input>>(id: CustomBlockId, args: Vec<T>) -> ScratchBlock {
    ScratchBlock::FunctionCallNoScreenRefresh(id, args.into_iter().map(|n| n.into()).collect())
}
fn call_sr<T: Into<Input>>(id: CustomBlockId, args: Vec<T>) -> ScratchBlock {
    ScratchBlock::FunctionCallScreenRefresh(id, args.into_iter().map(|n| n.into()).collect())
}
fn refresh() -> ScratchBlock {
    ScratchBlock::ScreenRefresh
}
fn lw<T: Into<Input>>(args: Vec<T>, body: Vec<ScratchBlock>) -> ScratchBlock {
    ScratchBlock::LocalsWith(args.into_iter().map(|n| n.into()).collect(), body)
}

struct Fixture {
    blocks: Vec<CustomBlock>,
}

impl Fixture {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    fn add(
        &mut self,
        body: Vec<ScratchBlock>,
        is_screen_refresh: bool,
        num_args: usize,
    ) -> CustomBlockId {
        let id = CustomBlockId(self.blocks.len());
        self.blocks.push(CustomBlock {
            script: MaybeCompiled::ToCompile(Script::new_custom_block(
                body,
                num_args,
                id,
                is_screen_refresh,
            )),
            is_screen_refresh,
            num_args,
        });
        id
    }

    fn body(&self, id: CustomBlockId) -> &[ScratchBlock] {
        match &self.blocks[id.0].script {
            MaybeCompiled::ToCompile(s) => &s.blocks,
            MaybeCompiled::Compiled(_) => panic!("expected ToCompile"),
        }
    }
}

/// Drive `inline_inner` over the given `(callee, parents)` edges.
///
/// Edges are processed in order, this lets tests pin down multi-level ordering.
fn inline(fx: &mut Fixture, child_to_parents: &[(CustomBlockId, Vec<CustomBlockId>)]) -> bool {
    let parent_sets: Vec<HashSet<CustomBlockId>> = child_to_parents
        .iter()
        .map(|(_, parents)| parents.iter().copied().collect())
        .collect();
    let calls: Vec<(CustomBlockId, &HashSet<CustomBlockId>)> = child_to_parents
        .iter()
        .zip(&parent_sets)
        .map(|((id, _), parents)| (*id, parents))
        .collect();

    let mut inlined = false;
    inline_inner(
        &calls,
        &mut fx.blocks,
        |w, id| &w[id.0],
        |w, id| w[id.0].is_screen_refresh,
        |w: &mut Vec<CustomBlock>, id: &CustomBlockId| &mut w[id.0].script,
        &mut inlined,
    );
    inlined
}

fn assert_blocks(actual: &[ScratchBlock], expected: &[ScratchBlock]) {
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

// ---------------------------------------------------------------------------
// Basic inlining + argument rewriting
// ---------------------------------------------------------------------------

#[test]
fn inlines_single_caller_and_rewrites_args() {
    let mut fx = Fixture::new();
    let b = fx.add(vec![arg(0), arg(1)], false, 2);
    let a = fx.add(vec![call_nr(b, vec![arg(0), arg(0)])], false, 0);

    assert!(inline(&mut fx, &[(b, vec![a])]));

    assert_blocks(
        fx.body(a),
        &[lw(vec![arg(0), arg(0)], vec![local(0), local(1)])],
    );
    assert_blocks(fx.body(b), &[arg(0), arg(1)]); // callee untouched
}

#[test]
fn inline_multiple_callers() {
    let mut fx = Fixture::new();
    let b = fx.add(vec![arg(0)], false, 1);
    let a1 = fx.add(vec![call_nr(b, vec![arg(0)])], false, 0);
    let a2 = fx.add(vec![call_nr(b, vec![arg(0)])], false, 0);

    assert!(inline(&mut fx, &[(b, vec![a1, a2])]));

    assert_blocks(fx.body(a1), &[lw(vec![arg(0)], vec![local(0)])]);
    assert_blocks(fx.body(a2), &[lw(vec![arg(0)], vec![local(0)])]);
}

// TODO: I am kinda iffy about this one
#[test]
fn inlines_all_call_sites_from_the_single_parent() {
    // One caller, but two call sites with different args, both inline.
    let mut fx = Fixture::new();
    let b = fx.add(vec![arg(0)], false, 1);
    let a = fx.add(
        vec![call_nr(b, vec![arg(0)]), call_nr(b, vec![arg(1)])],
        false,
        2,
    );

    assert!(inline(&mut fx, &[(b, vec![a])]));

    assert_blocks(
        fx.body(a),
        &[
            lw(vec![arg(0)], vec![local(0)]),
            lw(vec![arg(1)], vec![local(0)]),
        ],
    );
}

// ---------------------------------------------------------------------------
// Screen-refresh interactions
// ---------------------------------------------------------------------------

#[test]
fn skips_refresh_block_with_args_and_refresh_body() {
    // is_screen_refresh && performs_refresh && num_args > 0 → never inlined.
    let mut fx = Fixture::new();
    let b = fx.add(vec![refresh(), arg(0)], true, 1);
    let a = fx.add(vec![call_sr(b, vec![arg(0)])], true, 0);

    assert!(!inline(&mut fx, &[(b, vec![a])]));
    assert_blocks(fx.body(a), &[call_sr(b, vec![arg(0)])]);
}

#[test]
fn inlines_refresh_block_without_args() {
    // is_screen_refresh && performs_refresh && num_args == 0 → inline.
    let mut fx = Fixture::new();
    let b = fx.add(vec![refresh()], true, 0);
    let a = fx.add(vec![call_sr::<ScratchBlock>(b, vec![])], true, 0);

    assert!(inline(&mut fx, &[(b, vec![a])]));
    assert_blocks(fx.body(a), &[lw::<ScratchBlock>(vec![], vec![refresh()])]);
}

#[test]
fn inlines_refresh_block_without_refresh_content() {
    // is_screen_refresh but body doesn't refresh → inline.
    let mut fx = Fixture::new();
    let b = fx.add(vec![arg(0)], true, 1);
    let a = fx.add(vec![call_sr(b, vec![arg(0)])], true, 0);

    assert!(inline(&mut fx, &[(b, vec![a])]));
    assert_blocks(fx.body(a), &[lw(vec![arg(0)], vec![local(0)])]);
}

#[test]
fn inlines_refreshing_body_into_non_refresh_parent() {
    // Non-refresh callee whose body refreshes, non-refresh parent → inline.
    let mut fx = Fixture::new();
    let b = fx.add(vec![refresh()], false, 0);
    let a = fx.add(vec![call_nr::<ScratchBlock>(b, vec![])], false, 0);

    assert!(inline(&mut fx, &[(b, vec![a])]));
    assert_blocks(fx.body(a), &[lw::<ScratchBlock>(vec![], vec![refresh()])]);
}

#[test]
fn does_not_inline_refreshing_body_into_refresh_parent() {
    // Non-refresh callee whose body refreshes, refresh parent → would change
    // refresh semantics, so skip.
    let mut fx = Fixture::new();
    let b = fx.add(vec![refresh()], false, 0);
    let a = fx.add(vec![call_nr::<ScratchBlock>(b, vec![])], true, 0);

    assert!(!inline(&mut fx, &[(b, vec![a])]));
    assert_blocks(fx.body(a), &[call_nr::<ScratchBlock>(b, vec![])]);
}

// ---------------------------------------------------------------------------
// Multi-level inlining
// ---------------------------------------------------------------------------

#[test]
fn multi_level_inline_nests_when_callee_is_processed_first() {
    // A → B → C, all single-caller. C is inlined into B first, so when B is
    // inlined into A, A receives B's body with C already inlined.
    let mut fx = Fixture::new();
    let c = fx.add(vec![arg(0)], false, 1);
    let b = fx.add(vec![call_nr(c, vec![arg(0)])], false, 1);
    let a = fx.add(vec![call_nr(b, vec![arg(0)])], false, 1);

    assert!(inline(&mut fx, &[(c, vec![b]), (b, vec![a])]));

    assert_blocks(fx.body(b), &[lw(vec![arg(0)], vec![local(0)])]);
    assert_blocks(
        fx.body(a),
        &[lw(vec![arg(0)], vec![lw(vec![local(0)], vec![local(0)])])],
    );
}

#[test]
fn multi_level_inline_in_reverse_order_leaves_call_for_next_round() {
    // Same graph, B processed before C: A's copy of B still calls C at that
    // point. That's fine within one round; the outer loop rebuilds the call
    // graph and inlines C next (see `chain_is_fully_inlined_across_rounds`).
    let mut fx = Fixture::new();
    let c = fx.add(vec![arg(0)], false, 1);
    let b = fx.add(vec![call_nr(c, vec![arg(0)])], false, 1);
    let a = fx.add(vec![call_nr(b, vec![arg(0)])], false, 1);

    assert!(inline(&mut fx, &[(b, vec![a]), (c, vec![b])]));

    assert_blocks(fx.body(b), &[lw(vec![arg(0)], vec![local(0)])]);
    assert_blocks(
        fx.body(a),
        &[lw(vec![arg(0)], vec![call_nr(c, vec![local(0)])])],
    );
}

// ---------------------------------------------------------------------------
// End to end: the analyze/inline loop
// ---------------------------------------------------------------------------

fn flag_has_call(builder: &ProjectBuilder) -> bool {
    let MaybeCompiled::ToCompile(flag) = &builder.runtime.scripts.green_flags[0].1 else {
        panic!("expected ToCompile");
    };
    let mut found = false;
    for b in &flag.blocks {
        b.walk(
            &mut |b| {
                if let ScratchBlock::FunctionCallNoScreenRefresh(..)
                | ScratchBlock::FunctionCallScreenRefresh(..) = b
                {
                    found = true;
                }
            },
            false,
        );
    }
    found
}

#[test]
fn chain_is_fully_inlined_across_rounds() {
    // flag → A → B → C. One round only inlines one level deep per call
    // site, so full inlining needs the call graph rebuilt between rounds.
    let mut builder = project(vec![
        Script::new_green_flag(vec![call(A)]),
        custom_block(A, vec![call(B)]),
        custom_block(B, vec![call(C)]),
        custom_block(C, vec![set(X, 1.0)]),
    ]);

    builder.analyze_custom_blocks();

    assert!(!flag_has_call(&builder));
}

#[test]
fn inlined_away_caller_does_not_block_inlining_its_callee() {
    // flag → A → B, and B is only called from A. Once A is inlined into the
    // flag, B's only live caller is the flag, so B must be inlined too.
    let mut builder = project(vec![
        Script::new_green_flag(vec![call(A)]),
        custom_block(A, vec![call(B)]),
        custom_block(B, vec![set(X, 1.0)]),
    ]);

    builder.analyze_custom_blocks();

    assert!(!flag_has_call(&builder));
}

#[test]
fn recursive_block_is_not_inlined() {
    let mut builder = project(vec![
        Script::new_green_flag(vec![call(A)]),
        custom_block(A, vec![set(X, 1.0), call(A)]),
    ]);

    builder.analyze_custom_blocks();

    assert!(flag_has_call(&builder));
}

#[test]
fn leaf_called_from_recursive_block_is_inlined_into_it() {
    // A is recursive and stays a call, but its single-caller leaf B can
    // still be inlined into A.
    let mut builder = project(vec![
        Script::new_green_flag(vec![call(A)]),
        custom_block(A, vec![call(B), call(A)]),
        custom_block(B, vec![set(X, 1.0)]),
    ]);

    builder.analyze_custom_blocks();

    assert!(flag_has_call(&builder)); // A is still called
    let MaybeCompiled::ToCompile(a) = &builder.runtime.scripts.custom_blocks[A.0].1.script else {
        panic!("expected ToCompile");
    };
    assert!(
        !a.blocks
            .iter()
            .any(|b| matches!(b, ScratchBlock::FunctionCallNoScreenRefresh(id, _) if *id == B)),
        "B should have been inlined into A: {:?}",
        a.blocks
    );
}
