use std::collections::HashMap;

use crate::{
    ProjectBuilder,
    effects::{CallSiteInfo, Effects, VariableWrite, analyze},
    runtime::{CustomBlockFunc, CustomBlockId},
};

#[cfg(test)]
mod tests;

impl ProjectBuilder {
    pub(crate) fn analyze_custom_blocks(
        &mut self,
    ) -> (
        HashMap<CustomBlockId, Effects>,
        HashMap<CustomBlockId, CallSiteInfo>,
    ) {
        let mut resolved: HashMap<CustomBlockId, Effects> = HashMap::new();

        let mut call_sites: HashMap<CustomBlockId, CallSiteInfo> = HashMap::new();

        let mut changed = true;
        while changed {
            changed = false;
            for (id, script) in self.runtime.scripts.custom_blocks.iter().enumerate() {
                let CustomBlockFunc::ToCompile(script) = &script.script else {
                    continue;
                };
                if resolved.contains_key(&CustomBlockId(id)) {
                    // Already resolved
                    continue;
                }

                let mut calls = Vec::new();
                for block in &script.blocks {
                    block.check_calls_custom_blocks(&mut |called| {
                        calls.push(called);
                    });
                }
                if calls.iter().any(|n| !resolved.contains_key(n)) {
                    // Unresolved, resolve child functions first
                    continue;
                }

                let effects = analyze(
                    &script.blocks,
                    &mut |id| resolved.get(&id).cloned().unwrap_or(Effects::unknown()),
                    &mut |id, effects| {
                        call_sites
                            .entry(id)
                            // TODO: eliminate the default VariableWrite once we get more
                            // info about program in future (further optimizations)
                            .and_modify(|site| site.merge(&effects, &|_| VariableWrite::default()))
                            .or_insert_with(|| effects);
                    },
                );
                resolved.insert(CustomBlockId(id), effects);
                changed = true;
            }
        }

        // By now, recursive functions will not have been analysed
        // (will still use dynamic typing, no major optimizations).
        //
        // This is an intentional trade-off because analyzing them
        // adds significant complexity to the compiler and
        // is not worth it for the current use cases.

        (resolved, call_sites)
    }
}
