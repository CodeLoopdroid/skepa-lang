mod cfg_simplify;
mod const_fold;
mod copy_prop;
mod dce;
mod inlining;
mod licm;
mod strength_reduce;

use crate::ir::IrProgram;

pub fn optimize_program(program: &mut IrProgram) {
    loop {
        let mut changed = false;
        changed |= const_fold::run(program);
        changed |= copy_prop::run(program);
        changed |= dce::run(program);
        changed |= cfg_simplify::run(program);
        changed |= inlining::run(program);
        changed |= licm::run(program);
        changed |= strength_reduce::run(program);
        if !changed {
            break;
        }
    }
}
