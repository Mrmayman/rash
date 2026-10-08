//! Compile-time analysis of custom blocks for compiler optimization.
//!
//! Determines:
//! - Effects (what variables are modified)
//! - Data types of arguments
//! - Call-site information

use std::collections::{HashMap, HashSet};

use crate::{
    ProjectBuilder, ScratchBlock,
    effects::{CallSiteInfo, Effects, VariableWrite, analyze},
    runtime::{CustomBlockId, MaybeCompiled},
};

#[cfg(test)]
mod tests;

impl ProjectBuilder {
    pub(crate) fn analyze_custom_blocks(
        &mut self,
    ) -> HashMap<CustomBlockId, (Effects, CallSiteInfo)> {
        let mut effects_by_block: HashMap<CustomBlockId, Effects> = HashMap::new();
        let mut call_sites: HashMap<CustomBlockId, CallSiteInfo> = HashMap::new();

        #[allow(unused)]
        let (child_to_parent, parent_to_child) = self.collect_custom_block_calls();

        let fallback_write = &|_| VariableWrite::default();

        loop {
            let mut changed = false;

            // Start fresh on every cycle to achieve proper convergence (and prevent infinite looping)
            let previous_call_sites = std::mem::take(&mut call_sites);

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
                    continue;
                }

                // Use the previous pass's call-site info so intra-pass ordering doesn't matter.
                let parent_call_site = previous_call_sites.get(&block_id).cloned();

                let effects = analyze(
                    &script.blocks,
                    &mut |child| {
                        effects_by_block
                            .get(&child)
                            .cloned()
                            .unwrap_or(Effects::unknown())
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
                effects_by_block.insert(block_id, effects);
                changed = true;
            }

            if call_sites != previous_call_sites {
                changed = true;
            }

            if !changed {
                break;
            }
        }

        // let mut to_inline: HashMap<CustomBlockId, HashSet<CustomBlockId>> = HashMap::new();
        // for (child, parents) in child_to_parent {
        // if parents.len() == 1 {
        // TODO: add PushArgSlot and PopArgSlot
        // to_inline.insert(child, parents);
        // continue;
        // }
        // TODO: more heuristics
        // TODO: implement inlining
        // }

        // Recursive/cyclic blocks remain intentionally unresolved
        // and resort to runtime checks rather than compile time.

        let mut combined: HashMap<CustomBlockId, (Effects, CallSiteInfo)> = HashMap::new();

        for (id, effects) in effects_by_block {
            combined.insert(
                id,
                (
                    effects,
                    CallSiteInfo {
                        call_site: Effects::unknown(),
                        argument_types: Vec::new(),
                    },
                ),
            );
        }
        for (id, call_site_info) in call_sites {
            combined
                .entry(id)
                .and_modify(|(_, existing)| *existing = call_site_info.clone())
                .or_insert_with(|| (Effects::unknown(), call_site_info));
        }

        combined
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
