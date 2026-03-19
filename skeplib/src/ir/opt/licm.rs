use crate::ir::IrProgram;

pub fn run(program: &mut IrProgram) -> bool {
    let mut changed = false;

    for func in &mut program.functions {
        let ids: Vec<_> = func.blocks.iter().map(|block| block.id).collect();
        for header_id in ids {
            let Some(header_idx) = func.blocks.iter().position(|block| block.id == header_id)
            else {
                continue;
            };
            let header_name = func.blocks[header_idx].name.clone();
            if header_name != "while_cond" && header_name != "for_cond" {
                continue;
            }
            let Some(preheader_id) = find_preheader(func, header_id) else {
                continue;
            };
            let Some(preheader_idx) = func
                .blocks
                .iter()
                .position(|block| block.id == preheader_id)
            else {
                continue;
            };

            for block_name in related_loop_blocks(&header_name) {
                let Some(loop_idx) = func
                    .blocks
                    .iter()
                    .position(|block| block.name == *block_name)
                else {
                    continue;
                };
                let mut split_at = 0usize;
                for instr in &func.blocks[loop_idx].instrs {
                    if matches!(instr, crate::ir::Instr::Const { .. }) {
                        split_at += 1;
                    } else {
                        break;
                    }
                }
                if split_at == 0 {
                    continue;
                }
                let hoisted = func.blocks[loop_idx]
                    .instrs
                    .drain(..split_at)
                    .collect::<Vec<_>>();
                func.blocks[preheader_idx].instrs.extend(hoisted);
                changed = true;
            }
        }
    }

    changed
}

fn find_preheader(
    func: &crate::ir::IrFunction,
    header: crate::ir::BlockId,
) -> Option<crate::ir::BlockId> {
    let preds = predecessors(func, header);
    if preds.len() != 2 {
        return None;
    }
    preds.into_iter().find(|pred| {
        let Some(block) = func.blocks.iter().find(|block| block.id == *pred) else {
            return false;
        };
        block.name != "while_body" && block.name != "for_step"
    })
}

fn predecessors(
    func: &crate::ir::IrFunction,
    target: crate::ir::BlockId,
) -> Vec<crate::ir::BlockId> {
    let mut out = Vec::new();
    for block in &func.blocks {
        match &block.terminator {
            crate::ir::Terminator::Jump(next) if *next == target => out.push(block.id),
            crate::ir::Terminator::Branch(branch)
                if branch.then_block == target || branch.else_block == target =>
            {
                out.push(block.id);
            }
            _ => {}
        }
    }
    out
}

fn related_loop_blocks(header_name: &str) -> &'static [&'static str] {
    match header_name {
        "while_cond" => &["while_cond", "while_body"],
        "for_cond" => &["for_cond", "for_body", "for_step"],
        _ => &[],
    }
}
