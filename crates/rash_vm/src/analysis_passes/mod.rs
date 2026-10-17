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
        let mut effects_by_block: HashMap<CustomBlockId, Effects> = HashMap::new();
        let mut call_sites: HashMap<CustomBlockId, CallSiteInfo> = HashMap::new();

        #[allow(unused)]
        let (child_to_parent, parent_to_child) = self.collect_custom_block_calls();

        let fallback_write = &|_| VariableWrite::default();

        loop {
            let mut changed = false;

            // Start fresh on every cycle to achieve proper convergence (and prevent infinite looping)
            let previous_call_sites = std::mem::take(&mut call_sites);

            for (_, script) in &self.runtime.scripts.green_flags {
                let MaybeCompiled::ToCompile(script) = &script else {
                    continue;
                };

                analyze(
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
            var_buf_len: self.get_var_buf_len(),
        }
    }

    fn get_var_buf_len(&mut self) -> usize {
        let mut accessed_vars = HashSet::new();
        for s in self.runtime.scripts.iter() {
            let MaybeCompiled::ToCompile(script) = &s else {
                continue;
            };

            let eff = analyze(
                &script.blocks,
                &mut |_| Effects::unknown(),
                &|_| VariableWrite::default(),
                &mut |_, _| {},
            );
            accessed_vars.extend(eff.reads);
            accessed_vars.extend(eff.writes.into_keys());
        }
        let var_buf_len = if let Some(max) = accessed_vars.iter().max_by_key(|a| a.0) {
            max.0 + 1
        } else {
            accessed_vars.len()
        };
        var_buf_len
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
