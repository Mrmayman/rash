//! Compile-time analysis of custom blocks for compiler optimization.
//!
//! Determines:
//! - Effects (what variables are modified)
//! - Data types of arguments
//! - Call-site information

use std::collections::{HashMap, HashSet};

use crate::{
    ProjectBuilder, Ptr, ScratchBlock,
    effects::{CallSiteInfo, Effects, VariableWrite, analyze},
    runtime::{CustomBlock, CustomBlockId, MaybeCompiled},
};

#[cfg(test)]
mod tests;

pub struct CustomBlockAnalysis {
    pub effects: Effects,
    pub call_site: CallSiteInfo,
}

pub struct AnalysisOutput {
    pub custom_blocks: HashMap<CustomBlockId, CustomBlockAnalysis>,
    pub var_buf_len: usize,
}

impl std::ops::Index<&CustomBlockId> for AnalysisOutput {
    type Output = CustomBlockAnalysis;

    fn index(&self, index: &CustomBlockId) -> &Self::Output {
        self.custom_blocks.index(index)
    }
}

type CallCheck<T> = HashMap<CustomBlockId, HashSet<T>>;

/// `parent_to_child` covers every block (the analysis needs it for ordering).
/// `child_to_parent` only counts callers reachable from a green flag, so a
/// block that was inlined away no longer counts as a caller of its callees.
struct CallsAnalyzed {
    child_to_parent: CallCheck<CustomBlockId>,
    parent_to_child: CallCheck<CustomBlockId>,
    child_to_flags: CallCheck<usize>,
    /// Blocks that can (transitively) call themselves. Never inlined.
    recursive: HashSet<CustomBlockId>,
}

impl ProjectBuilder {
    pub(crate) fn analyze_custom_blocks(&mut self) -> AnalysisOutput {
        self.analyze_custom_blocks_with(true)
    }

    /// `inline: false` analyzes the program as written (used by tests that
    /// inspect call sites, which inlining would otherwise remove).
    fn analyze_custom_blocks_with(&mut self, inline: bool) -> AnalysisOutput {
        // Inlining changes both the call graph and the effects, so everything
        // derived from them is rebuilt from scratch each round. Every round
        // that inlines removes at least one call to a non-recursive block, so
        // this converges. The final round inlines nothing, which means the
        // analysis it produced describes the final program.
        loop {
            let calls = self.collect_custom_block_calls();
            let mut accessed_vars = HashSet::new();
            let (effects_by_block, call_sites) =
                self.analyze_pass(&calls.parent_to_child, &mut accessed_vars);

            if inline && self.inline_functions(&calls) {
                continue;
            }

            let custom_blocks = combine_into_analysis(effects_by_block, call_sites);
            let var_buf_len = accessed_vars.iter().map(|ptr| ptr.0 + 1).max().unwrap_or(0);

            return AnalysisOutput {
                custom_blocks,
                var_buf_len,
            };
        }
    }

    /// Compute effects and call-site info for every custom block via a
    /// fixpoint iteration. Call sites are always drawn from the *previous*
    /// iteration so that block ordering within a pass doesn't matter.
    fn analyze_pass(
        &mut self,
        parent_to_child: &HashMap<CustomBlockId, HashSet<CustomBlockId>>,
        accessed_vars: &mut HashSet<Ptr>,
    ) -> (
        HashMap<CustomBlockId, Effects>,
        HashMap<CustomBlockId, CallSiteInfo>,
    ) {
        let mut effects_by_block: HashMap<CustomBlockId, Effects> = HashMap::new();
        let mut call_sites: HashMap<CustomBlockId, CallSiteInfo> = HashMap::new();
        let unknown = &|_| VariableWrite::default();
        let mut first_iteration = true;

        loop {
            let mut changed = false;

            // Start fresh each iteration; using the previous iteration's
            // values keeps the ordering of blocks within a pass irrelevant.
            let previous_call_sites = std::mem::take(&mut call_sites);

            // --- Green flag scripts (top-level entry points) ---
            for (_, script) in &self.runtime.scripts.green_flags {
                let MaybeCompiled::ToCompile(script) = &script else {
                    continue;
                };

                let eff = analyze(
                    &script.blocks,
                    &|child| lookup_effects(&effects_by_block, child),
                    unknown,
                    &mut |child, local_effects| {
                        record_call_site(&mut call_sites, child, local_effects);
                    },
                );
                if first_iteration {
                    accessed_vars.extend(eff.reads);
                    accessed_vars.extend(eff.writes.into_keys());
                }
            }

            // --- Custom blocks ---
            for (id, script) in self.runtime.scripts.custom_blocks.iter().enumerate() {
                let MaybeCompiled::ToCompile(script) = &script.script else {
                    continue;
                };
                let block_id = CustomBlockId(id);

                // Wait until every callee has resolved effects before
                // analyzing this block.
                let has_unresolved_children =
                    parent_to_child.get(&block_id).is_some_and(|children| {
                        children.iter().any(|n| !effects_by_block.contains_key(n))
                    });

                if has_unresolved_children {
                    if first_iteration {
                        // Still collect accessed vars, treating children as unknown.
                        let eff = analyze(
                            &script.blocks,
                            &|_| Effects::unknown(),
                            unknown,
                            &mut |_, _| {},
                        );
                        accessed_vars.extend(eff.reads);
                        accessed_vars.extend(eff.writes.into_keys());
                    }
                    continue;
                }

                let parent_call_site = previous_call_sites.get(&block_id).cloned();

                let effects = analyze(
                    &script.blocks,
                    &|child| lookup_effects(&effects_by_block, child),
                    &|v| {
                        previous_call_sites
                            .get(&block_id)
                            .filter(|n| !n.at_site.is_unknown)
                            .and_then(|n| n.at_site.writes.get(&v).copied())
                            .unwrap_or_default()
                    },
                    &mut |child, local_effects| {
                        let contribution = match parent_call_site.clone() {
                            Some(mut parent) => {
                                parent.sequence(&local_effects, unknown);
                                parent
                            }
                            None => local_effects.clone(),
                        };
                        record_call_site(&mut call_sites, child, contribution);
                    },
                );

                if effects_by_block
                    .get(&block_id)
                    .is_some_and(|n| *n == effects)
                {
                    continue;
                }

                if first_iteration {
                    accessed_vars.extend(effects.reads.iter().copied());
                    accessed_vars.extend(effects.writes.keys().copied());
                }
                effects_by_block.insert(block_id, effects);
                changed = true;
            }

            if call_sites != previous_call_sites {
                changed = true;
            }

            first_iteration = false;

            if !changed {
                break;
            }
        }
        (effects_by_block, call_sites)
    }

    /// Inline single-caller, non-recursive blocks. Returns whether anything
    /// changed; the caller must then rebuild `calls`, since it is stale.
    fn inline_functions(&mut self, calls: &CallsAnalyzed) -> bool {
        // Escape hatch for debugging.
        if std::env::var("RASH_SKIP_INLINE")
            .map(|value| value == "1" || value.to_lowercase() == "true")
            .unwrap_or(false)
        {
            return false;
        }

        let mut inlined_something = false;
        let scripts = &mut self.runtime.scripts;

        // Custom blocks called from other custom blocks.
        inline_inner(
            &inline_candidates(&calls.child_to_parent, &calls.recursive),
            &mut *scripts,
            |s, id| s.custom_blocks.get(id.0),
            |s, id| s.custom_blocks[id.0].is_screen_refresh,
            |s, id| &mut s.custom_blocks[id.0].script,
            &mut inlined_something,
        );

        // Custom blocks called from green flags.
        inline_inner(
            &inline_candidates(&calls.child_to_flags, &calls.recursive),
            &mut *scripts,
            |s, id| s.custom_blocks.get(id.0),
            |_, _| true,
            |s, flag| &mut s.green_flags[*flag].1,
            &mut inlined_something,
        );

        inlined_something
    }

    /// Build the caller/callee maps used by the analysis and inliner.
    fn collect_custom_block_calls(&self) -> CallsAnalyzed {
        let scripts = &self.runtime.scripts;

        let mut parent_to_child: CallCheck<CustomBlockId> = HashMap::new();
        for (id, script) in scripts.custom_blocks.iter().enumerate() {
            if let MaybeCompiled::ToCompile(script) = &script.script {
                parent_to_child.insert(CustomBlockId(id), called_blocks(&script.blocks));
            }
        }

        // Blocks reachable from a green flag.
        let mut child_to_flags: CallCheck<usize> = HashMap::new();
        let mut reachable = HashSet::new();
        let mut stack = Vec::new();
        for (flag, (_, script)) in scripts.green_flags.iter().enumerate() {
            let MaybeCompiled::ToCompile(script) = script else {
                continue;
            };
            for child in called_blocks(&script.blocks) {
                child_to_flags.entry(child).or_default().insert(flag);
                stack.push(child);
            }
        }
        while let Some(id) = stack.pop() {
            if reachable.insert(id) {
                stack.extend(parent_to_child.get(&id).into_iter().flatten().copied());
            }
        }

        let mut child_to_parent: CallCheck<CustomBlockId> = HashMap::new();
        for (&parent, children) in &parent_to_child {
            if reachable.contains(&parent) {
                for &child in children {
                    child_to_parent.entry(child).or_default().insert(parent);
                }
            }
        }

        let recursive = recursive_blocks(&parent_to_child);

        CallsAnalyzed {
            child_to_parent,
            parent_to_child,
            child_to_flags,
            recursive,
        }
    }
}

/// All custom blocks called anywhere in `blocks`.
fn called_blocks<'a>(blocks: impl IntoIterator<Item = &'a ScratchBlock>) -> HashSet<CustomBlockId> {
    let mut called = HashSet::new();
    for block in blocks {
        block.walk(
            &mut |b| {
                if let ScratchBlock::FunctionCallNoScreenRefresh(child_id, _)
                | ScratchBlock::FunctionCallScreenRefresh(child_id, _) = b
                {
                    called.insert(*child_id);
                }
            },
            false,
        );
    }
    called
}

/// Blocks that can reach themselves through the call graph.
fn recursive_blocks(
    parent_to_child: &HashMap<CustomBlockId, HashSet<CustomBlockId>>,
) -> HashSet<CustomBlockId> {
    parent_to_child
        .iter()
        .filter(|(id, children)| {
            let mut seen = HashSet::new();
            let mut stack: Vec<_> = children.iter().copied().collect();
            while let Some(n) = stack.pop() {
                if n == **id {
                    return true;
                }
                if seen.insert(n) {
                    stack.extend(parent_to_child.get(&n).into_iter().flatten().copied());
                }
            }
            false
        })
        .map(|(&id, _)| id)
        .collect()
}

/// Retrieve a custom block's already-computed effects, defaulting to
/// "unknown" when it hasn't been resolved yet in this pass.
fn lookup_effects(
    effects_by_block: &HashMap<CustomBlockId, Effects>,
    child: CustomBlockId,
) -> Effects {
    effects_by_block
        .get(&child)
        .cloned()
        .unwrap_or(Effects::unknown())
}

/// Merge `contribution` into the call site already recorded for `child`
/// during this pass, or insert it if this is the first contribution.
fn record_call_site(
    call_sites: &mut HashMap<CustomBlockId, CallSiteInfo>,
    child: CustomBlockId,
    local_effects: CallSiteInfo,
) {
    call_sites
        .entry(child)
        .and_modify(|existing| existing.merge(&local_effects, &|_| VariableWrite::default()))
        .or_insert(local_effects);
}

fn combine_into_analysis(
    effects_by_block: HashMap<CustomBlockId, Effects>,
    call_sites: HashMap<CustomBlockId, CallSiteInfo>,
) -> HashMap<CustomBlockId, CustomBlockAnalysis> {
    let mut combined = HashMap::new();

    for (id, effects) in effects_by_block {
        combined.insert(
            id,
            CustomBlockAnalysis {
                effects,
                call_site: CallSiteInfo {
                    at_site: Effects::unknown(),
                    arguments: Vec::new(),
                },
            },
        );
    }
    for (id, call_site) in call_sites {
        combined
            .entry(id)
            .and_modify(|c| c.call_site = call_site.clone())
            .or_insert_with(|| CustomBlockAnalysis {
                effects: Effects::unknown(),
                call_site,
            });
    }
    combined
}

/// Non-recursive callees with their callers.
fn inline_candidates<'a, T>(
    callers: &'a CallCheck<T>,
    recursive: &HashSet<CustomBlockId>,
) -> Vec<(CustomBlockId, &'a HashSet<T>)> {
    callers
        .iter()
        .filter(|(id, _)| !recursive.contains(id))
        .map(|(id, parents)| (*id, parents))
        .collect()
}

fn inline_inner<W, T>(
    calls: &[(CustomBlockId, &HashSet<T>)], // callee, caller pair
    world: &mut W,
    get_callee: impl Fn(&W, CustomBlockId) -> Option<&CustomBlock>,
    is_refresh: impl Fn(&W, &T) -> bool,
    get_script: impl for<'a, 'b> Fn(&'a mut W, &'b T) -> &'a mut MaybeCompiled,
    inlined_something: &mut bool,
) {
    for (child, parents) in calls {
        let Some(info) = get_callee(world, *child).cloned() else {
            continue;
        };
        let MaybeCompiled::ToCompile(s) = &info.script else {
            continue;
        };

        // --- Can we inline? ---
        // Inlining a refreshing block into a refreshing context would change
        // refresh semantics, so those stay as calls.
        let performs_refresh = s.blocks.iter().any(|n| n.could_trigger_refresh());

        if info.is_screen_refresh && performs_refresh && info.num_args > 0 {
            continue;
        }

        let can_turn_into_refresh = !info.is_screen_refresh && performs_refresh;
        if can_turn_into_refresh && parents.iter().any(|p| is_refresh(&*world, p)) {
            continue;
        }

        // --- Should we inline? ---
        // TODO: more heuristics
        let should_inline = parents.len() == 1;
        if !should_inline {
            continue;
        }

        // --- Do the inline ---
        // Rewrite the callee's body: `FunctionGetArg` becomes `LocalsGet`.
        let mut blocks_to_inline = s.blocks.clone();
        for b in &mut blocks_to_inline {
            b.walk_mut(
                &mut |b| {
                    if let ScratchBlock::FunctionGetArg(n) = b {
                        *b = ScratchBlock::LocalsGet(*n);
                    }
                },
                true,
            );
        }

        let mut replaced = false;
        for par in *parents {
            let MaybeCompiled::ToCompile(scr) = get_script(&mut *world, par) else {
                continue;
            };
            for b in &mut scr.blocks {
                b.walk_mut(
                    &mut |b| {
                        if let ScratchBlock::FunctionCallNoScreenRefresh(id, args)
                        | ScratchBlock::FunctionCallScreenRefresh(id, args) = b
                            && id == child
                        {
                            replaced = true;
                            *b = ScratchBlock::LocalsWith(
                                std::mem::take(args),
                                blocks_to_inline.clone(),
                            )
                        }
                    },
                    true,
                );
            }
        }

        // Only report progress if a call was actually replaced, so the outer
        // loop can't spin on a no-op.
        *inlined_something |= replaced;
    }
}
