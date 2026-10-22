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
    runtime::{CustomBlockId, MaybeCompiled},
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

impl ProjectBuilder {
    pub(crate) fn analyze_custom_blocks(&mut self) -> AnalysisOutput {
        let mut accessed_vars = HashSet::new();
        let (child_to_parent, parent_to_child) = self.collect_custom_block_calls();

        let (mut effects_by_block, mut call_sites) =
            self.analyze_pass(&parent_to_child, &mut accessed_vars);

        let has_changed = self.inline_functions(child_to_parent);

        if has_changed {
            (effects_by_block, call_sites) =
                self.analyze_pass(&parent_to_child, &mut accessed_vars);
        }

        // Recursive/cyclic blocks remain intentionally unresolved
        // and resort to runtime checks rather than compile time.

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

        AnalysisOutput {
            custom_blocks: combined,
            var_buf_len: if let Some(max) = accessed_vars.iter().max_by_key(|a| a.0) {
                max.0 + 1
            } else {
                accessed_vars.len()
            },
        }
    }

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

        let fallback_write = &|_| VariableWrite::default();
        let mut first_iteration = true;

        loop {
            let mut changed = false;

            // Start fresh on every cycle to achieve proper convergence (and prevent infinite looping)
            let previous_call_sites = std::mem::take(&mut call_sites);

            for (_, script) in &self.runtime.scripts.green_flags {
                let MaybeCompiled::ToCompile(script) = &script else {
                    continue;
                };

                let eff = analyze(
                    &script.blocks,
                    &|child| {
                        effects_by_block
                            .get(&child)
                            .cloned()
                            .unwrap_or(Effects::unknown())
                    },
                    &|_| VariableWrite::default(),
                    &mut |child, local_effects| {
                        // Fresh entry this pass ⇒ this is a genuine replace, not accumulate.
                        call_sites
                            .entry(child)
                            .and_modify(|existing| existing.merge(&local_effects, fallback_write))
                            .or_insert(local_effects);
                    },
                );
                if first_iteration {
                    accessed_vars.extend(eff.reads);
                    accessed_vars.extend(eff.writes.into_keys());
                }
            }

            for (id, script) in self.runtime.scripts.custom_blocks.iter().enumerate() {
                let MaybeCompiled::ToCompile(script) = &script.script else {
                    continue;
                };

                let block_id = CustomBlockId(id);

                let has_unresolved_children =
                    parent_to_child.get(&block_id).is_some_and(|children| {
                        children.iter().any(|n| !effects_by_block.contains_key(n))
                    });

                if has_unresolved_children {
                    if first_iteration {
                        let eff = analyze(
                            &script.blocks,
                            &|_| Effects::unknown(),
                            &|_| VariableWrite::default(),
                            &mut |_, _| {},
                        );
                        accessed_vars.extend(eff.reads);
                        accessed_vars.extend(eff.writes.into_keys());
                    }
                    continue;
                }

                // Use the previous pass's call-site info so intra-pass ordering doesn't matter.
                let parent_call_site = previous_call_sites.get(&block_id).cloned();

                let effects = analyze(
                    &script.blocks,
                    &|child| {
                        effects_by_block
                            .get(&child)
                            .cloned()
                            .unwrap_or(Effects::unknown())
                    },
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
                                parent.sequence(&local_effects, fallback_write);
                                parent
                            }
                            None => local_effects.clone(),
                        };
                        // Fresh entry this pass ⇒ this is a genuine replace, not accumulate.
                        call_sites
                            .entry(child)
                            .and_modify(|existing| existing.merge(&contribution, fallback_write))
                            .or_insert(contribution);
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

    fn inline_functions(
        &mut self,
        child_to_parent: HashMap<CustomBlockId, HashSet<CustomBlockId>>,
    ) -> bool {
        let skip_inline = std::env::var("RASH_SKIP_INLINE").ok();
        if skip_inline.as_deref() == Some("1")
            || skip_inline
                .as_deref()
                .is_some_and(|n| n.to_lowercase() == "true")
        {
            return false;
        }

        let mut to_inline: HashMap<CustomBlockId, HashSet<CustomBlockId>> = HashMap::new();
        let mut inlined_something = false;

        for (child, parents) in child_to_parent {
            let info = &self.runtime.scripts.custom_blocks[child.0];

            // ========================================================
            // 1. Can we inline?
            // ========================================================

            if info.is_screen_refresh && info.num_args > 0 {
                // The compiler doesn't support persistent storage
                // of inlined arguments for screen refresh blocks, so if any, just bail.
                // I don't plan on adding that either (too much complexity).
                continue;
            }

            let MaybeCompiled::ToCompile(s) = &info.script else {
                continue;
            };

            // Case where child is non-refresh, parent is refresh,
            // but when child inlined into parent,
            // child could become refresh like parent (incorrect behaviour)
            let can_turn_into_refresh =
                !info.is_screen_refresh && s.blocks.iter().any(|n| n.could_trigger_refresh());
            let parents_are_refresh = parents
                .iter()
                .filter_map(|n| self.runtime.scripts.custom_blocks.get(n.0))
                .any(|n| n.is_screen_refresh);

            if can_turn_into_refresh && parents_are_refresh {
                continue;
            }

            // ========================================================
            // 2. Should we inline?
            // ========================================================

            // TODO: more heuristics
            let should_inline = parents.len() == 1;

            if should_inline {
                to_inline.insert(child, parents);
                continue;
            }

            inlined_something = true;

            // TODO: do inlining
        }
        inlined_something
    }

    fn collect_custom_block_calls(
        &mut self,
    ) -> (
        HashMap<CustomBlockId, HashSet<CustomBlockId>>,
        HashMap<CustomBlockId, HashSet<CustomBlockId>>,
    ) {
        // In this terminology we call callers as parent and callees as child.
        let mut callers_by_callee: HashMap<CustomBlockId, HashSet<CustomBlockId>> = HashMap::new();
        let mut callees_by_caller: HashMap<CustomBlockId, HashSet<CustomBlockId>> = HashMap::new();

        for (id, script) in self.runtime.scripts.custom_blocks.iter().enumerate() {
            let MaybeCompiled::ToCompile(script) = &script.script else {
                continue;
            };

            let mut calls = HashSet::new();
            for block in &script.blocks {
                block.walk(
                    &mut |b| {
                        if let ScratchBlock::FunctionCallNoScreenRefresh(child_id, _)
                        | ScratchBlock::FunctionCallScreenRefresh(child_id, _) = b
                        {
                            calls.insert(*child_id);
                            callers_by_callee
                                .entry(*child_id)
                                .or_default()
                                .insert(CustomBlockId(id));
                        }
                    },
                    false,
                );
            }
            callees_by_caller.insert(CustomBlockId(id), calls);
        }
        (callers_by_callee, callees_by_caller)
    }
}
