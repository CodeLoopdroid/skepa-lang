use std::collections::{HashMap, HashSet};

use crate::ir::{BlockId, IrProgram, Terminator};

pub fn run(program: &mut IrProgram) -> bool {
    let mut changed = false;

    for func in &mut program.functions {
        let redirect = collect_jump_only_redirects(func);
        if redirect.is_empty() {
            continue;
        }

        changed |= rewrite_targets(func, &redirect);
        changed |= remove_unreachable_blocks(func);
    }

    changed
}

fn collect_jump_only_redirects(func: &crate::ir::IrFunction) -> HashMap<BlockId, BlockId> {
    let mut redirect = HashMap::new();
    for block in &func.blocks {
        if block.id == func.entry {
            continue;
        }
        if block.instrs.is_empty()
            && let Terminator::Jump(target) = block.terminator
        {
            redirect.insert(block.id, target);
        }
    }

    let keys = redirect.keys().copied().collect::<Vec<_>>();
    for block in keys {
        let target = resolve_redirect(block, &redirect);
        redirect.insert(block, target);
    }

    redirect
}

fn resolve_redirect(start: BlockId, redirect: &HashMap<BlockId, BlockId>) -> BlockId {
    let mut seen = HashSet::new();
    let mut current = start;
    while let Some(next) = redirect.get(&current).copied() {
        if !seen.insert(current) {
            break;
        }
        current = next;
    }
    current
}

fn rewrite_targets(func: &mut crate::ir::IrFunction, redirect: &HashMap<BlockId, BlockId>) -> bool {
    let mut changed = false;
    for block in &mut func.blocks {
        match &mut block.terminator {
            Terminator::Jump(target) => {
                let final_target = resolve_redirect(*target, redirect);
                if final_target != *target {
                    *target = final_target;
                    changed = true;
                }
            }
            Terminator::Branch(branch) => {
                let then_target = resolve_redirect(branch.then_block, redirect);
                let else_target = resolve_redirect(branch.else_block, redirect);
                if then_target != branch.then_block {
                    branch.then_block = then_target;
                    changed = true;
                }
                if else_target != branch.else_block {
                    branch.else_block = else_target;
                    changed = true;
                }
            }
            Terminator::Return(_) | Terminator::Panic { .. } | Terminator::Unreachable => {}
        }
    }
    changed
}

fn remove_unreachable_blocks(func: &mut crate::ir::IrFunction) -> bool {
    let mut reachable = HashSet::new();
    let mut work = vec![func.entry];

    while let Some(block_id) = work.pop() {
        if !reachable.insert(block_id) {
            continue;
        }
        let Some(block) = func.blocks.iter().find(|block| block.id == block_id) else {
            continue;
        };
        match &block.terminator {
            Terminator::Jump(target) => work.push(*target),
            Terminator::Branch(branch) => {
                work.push(branch.then_block);
                work.push(branch.else_block);
            }
            Terminator::Return(_) | Terminator::Panic { .. } | Terminator::Unreachable => {}
        }
    }

    let before = func.blocks.len();
    func.blocks.retain(|block| reachable.contains(&block.id));
    before != func.blocks.len()
}
