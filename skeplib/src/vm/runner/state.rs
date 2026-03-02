use crate::bytecode::{FunctionChunk, Value};

pub(super) struct CallFrame<'a> {
    pub chunk: &'a FunctionChunk,
    pub function_name: &'a str,
    pub ip: usize,
    pub locals: Vec<Value>,
    pub stack: Vec<Value>,
}

pub(super) struct FrameStorage {
    pub locals: Vec<Value>,
    pub stack: Vec<Value>,
}

impl FrameStorage {
    pub(super) fn new(
        mut locals: Vec<Value>,
        mut stack: Vec<Value>,
        locals_len: usize,
        stack_capacity: usize,
    ) -> Self {
        locals.clear();
        if locals.capacity() < locals_len {
            locals.reserve(locals_len - locals.capacity());
        }
        stack.clear();
        if stack.capacity() < stack_capacity {
            stack.reserve(stack_capacity - stack.capacity());
        }
        Self { locals, stack }
    }
}

pub(super) fn acquire_storage(
    locals_len: usize,
    stack_capacity: usize,
    locals_pool: &mut Vec<Vec<Value>>,
    stack_pool: &mut Vec<Vec<Value>>,
) -> FrameStorage {
    let locals = locals_pool.pop().unwrap_or_default();
    let stack = stack_pool.pop().unwrap_or_default();
    FrameStorage::new(locals, stack, locals_len, stack_capacity.max(8))
}

impl<'a> CallFrame<'a> {
    pub(super) fn with_storage(
        chunk: &'a FunctionChunk,
        function_name: &'a str,
        storage: FrameStorage,
    ) -> Self {
        Self {
            chunk,
            function_name,
            ip: 0,
            locals: storage.locals,
            stack: storage.stack,
        }
    }

    pub(super) fn into_storage(mut self) -> FrameStorage {
        self.locals.clear();
        self.stack.clear();
        FrameStorage {
            locals: self.locals,
            stack: self.stack,
        }
    }

    #[inline(always)]
    pub(super) fn read_local_cloned(&self, slot: usize) -> Option<Value> {
        if slot < self.locals.len() {
            // Compiled bytecode keeps frame locals pre-sized, so this is the hot path.
            Some(unsafe { self.locals.get_unchecked(slot).clone() })
        } else {
            None
        }
    }

    #[inline(always)]
    pub(super) fn write_local_fast(&mut self, slot: usize, value: Value) -> bool {
        if slot < self.locals.len() {
            // Compiled bytecode keeps frame locals pre-sized, so this is the hot path.
            unsafe {
                *self.locals.get_unchecked_mut(slot) = value;
            }
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub(super) fn add_const_to_int_local(
        &mut self,
        slot: usize,
        rhs: i64,
    ) -> Option<Result<(), ()>> {
        if slot >= self.locals.len() {
            return None;
        }
        match unsafe { self.locals.get_unchecked_mut(slot) } {
            Value::Int(lhs) => {
                *lhs += rhs;
                Some(Ok(()))
            }
            _ => Some(Err(())),
        }
    }

    #[inline(always)]
    pub(super) fn add_int_local_to_local(
        &mut self,
        dst: usize,
        src: usize,
    ) -> Option<Result<(), ()>> {
        if dst >= self.locals.len() || src >= self.locals.len() {
            return None;
        }
        if dst == src {
            match unsafe { self.locals.get_unchecked_mut(dst) } {
                Value::Int(value) => {
                    *value += *value;
                    Some(Ok(()))
                }
                _ => Some(Err(())),
            }
        } else if dst < src {
            let (left, right) = self.locals.split_at_mut(src);
            match (&mut left[dst], &right[0]) {
                (Value::Int(dst_value), Value::Int(src_value)) => {
                    *dst_value += *src_value;
                    Some(Ok(()))
                }
                _ => Some(Err(())),
            }
        } else {
            let (left, right) = self.locals.split_at_mut(dst);
            match (&mut right[0], &left[src]) {
                (Value::Int(dst_value), Value::Int(src_value)) => {
                    *dst_value += *src_value;
                    Some(Ok(()))
                }
                _ => Some(Err(())),
            }
        }
    }

    #[inline(always)]
    pub(super) fn pop2(&mut self) -> Option<(Value, Value)> {
        let rhs = self.stack.pop()?;
        let lhs = self.stack.pop()?;
        Some((lhs, rhs))
    }
}
