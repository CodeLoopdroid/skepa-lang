use crate::bytecode::{FunctionChunk, Value};
use crate::vm::{VmError, VmErrorKind};

use super::{RunOptions, err_at, stack_capacity_hint, state};

pub(super) struct IndexedCallCtx<'a, 'b> {
    pub fn_table: &'a [&'a FunctionChunk],
    pub idx: usize,
    pub argc: usize,
    pub current_depth: usize,
    pub opts: RunOptions,
    pub function_name: &'b str,
    pub ip: usize,
    pub locals_pool: &'b mut Vec<Vec<Value>>,
    pub stack_pool: &'b mut Vec<Vec<Value>>,
}

#[inline(always)]
pub(super) fn call_idx_fast<'a>(
    frame: &mut state::CallFrame<'_>,
    ctx: IndexedCallCtx<'a, '_>,
) -> Result<state::CallFrame<'a>, VmError> {
    if frame.stack.len() < ctx.argc {
        return Err(err_at(
            VmErrorKind::StackUnderflow,
            "Stack underflow on CallIdx",
            ctx.function_name,
            ctx.ip,
        ));
    }
    let Some(callee_chunk) = ctx.fn_table.get(ctx.idx).copied() else {
        return Err(err_at(
            VmErrorKind::UnknownFunction,
            format!("Invalid function index `{}`", ctx.idx),
            ctx.function_name,
            ctx.ip,
        ));
    };
    if ctx.current_depth + ctx.opts.depth >= ctx.opts.config.max_call_depth {
        return Err(VmError::new(
            VmErrorKind::StackOverflow,
            format!(
                "Call stack limit exceeded ({})",
                ctx.opts.config.max_call_depth
            ),
        ));
    }
    if ctx.argc != callee_chunk.param_count {
        return Err(VmError::new(
            VmErrorKind::ArityMismatch,
            format!(
                "Function `{}` arity mismatch: expected {}, got {}",
                callee_chunk.name, callee_chunk.param_count, ctx.argc
            ),
        ));
    }
    frame.ip += 1;
    Ok(push_call_frame(
        frame,
        callee_chunk,
        ctx.argc,
        None,
        ctx.locals_pool,
        ctx.stack_pool,
    ))
}

pub(super) fn push_call_frame<'a>(
    caller: &mut state::CallFrame<'_>,
    callee_chunk: &'a FunctionChunk,
    argc: usize,
    receiver: Option<Value>,
    locals_pool: &mut Vec<Vec<Value>>,
    stack_pool: &mut Vec<Vec<Value>>,
) -> state::CallFrame<'a> {
    let mut new_frame = state::CallFrame::with_storage(
        callee_chunk,
        &callee_chunk.name,
        state::acquire_storage(
            callee_chunk.locals_count,
            stack_capacity_hint(callee_chunk),
            locals_pool,
            stack_pool,
        ),
    );
    if let Some(receiver) = receiver {
        new_frame.locals.push(receiver);
    }
    let split = caller.stack.len() - argc;
    new_frame.locals.extend(caller.stack.split_off(split));
    if new_frame.locals.len() < callee_chunk.locals_count {
        new_frame
            .locals
            .resize(callee_chunk.locals_count, Value::Unit);
    }
    new_frame
}
