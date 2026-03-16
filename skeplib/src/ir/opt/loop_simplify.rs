use std::collections::HashSet;

use crate::ir::{IrProgram, Terminator};

pub fn run(program: &mut IrProgram) -> bool {
    let mut changed = false;

    for func in &mut program.functions {
        let mut redirects = Vec::new();
        for block in &func.blocks {
            let is_loop_body = block.name == "while_body" || block.name == "for_body";
            if !is_loop_body || !block.instrs.is_empty() {
                continue;
            }
            let Terminator::Jump(target) = block.terminator else {
                continue;
            };
            redirects.push((block.id, target));
        }

        if redirects.is_empty() {
            continue;
        }

        let redirect_map = redirects
            .iter()
            .copied()
            .collect::<std::collections::HashMap<_, _>>();

        for block in &mut func.blocks {
            match &mut block.terminator {
                Terminator::Jump(target) => {
                    if let Some(next) = redirect_map.get(target) {
                        *target = *next;
                        changed = true;
                    }
                }
                Terminator::Branch(branch) => {
                    if let Some(next) = redirect_map.get(&branch.then_block) {
                        branch.then_block = *next;
                        changed = true;
                    }
                    if let Some(next) = redirect_map.get(&branch.else_block) {
                        branch.else_block = *next;
                        changed = true;
                    }
                }
                Terminator::Return(_) | Terminator::Panic { .. } | Terminator::Unreachable => {}
            }
        }

        if changed {
            let mut reachable = HashSet::new();
            let mut stack = vec![func.entry];
            while let Some(id) = stack.pop() {
                if !reachable.insert(id) {
                    continue;
                }
                let Some(block) = func.blocks.iter().find(|block| block.id == id) else {
                    continue;
                };
                match &block.terminator {
                    Terminator::Jump(target) => stack.push(*target),
                    Terminator::Branch(branch) => {
                        stack.push(branch.then_block);
                        stack.push(branch.else_block);
                    }
                    Terminator::Return(_) | Terminator::Panic { .. } | Terminator::Unreachable => {}
                }
            }
            func.blocks.retain(|block| reachable.contains(&block.id));
        }
    }

    changed
}
