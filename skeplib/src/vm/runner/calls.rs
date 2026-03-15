use crate::bytecode::{BytecodeModule, FunctionChunk, Instr, Value};
use crate::vm::{BuiltinHost, BuiltinRegistry, VmConfig, VmError, VmErrorKind};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use super::{err_at, frame, state};

pub(super) struct CallEnv<'a> {
    pub host: &'a mut dyn BuiltinHost,
    pub reg: &'a BuiltinRegistry,
}

#[derive(Clone, Copy)]
pub(super) struct CallOptions {
    pub depth: usize,
    pub config: VmConfig,
}

pub(super) struct Site<'a> {
    pub function_name: &'a str,
    pub ip: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ModuleCacheKey {
    ptr: usize,
    len: usize,
    name_fingerprint: u64,
}

type MethodMap = HashMap<String, HashMap<String, usize>>;
type MethodCache = HashMap<ModuleCacheKey, MethodMap>;
type MethodIdMap = HashMap<String, HashMap<usize, usize>>;
type MethodIdCache = HashMap<ModuleCacheKey, MethodIdMap>;

pub(super) fn take_call_args(stack: &mut Vec<Value>, argc: usize) -> Vec<Value> {
    let split = stack.len() - argc;
    stack.split_off(split)
}

fn module_cache_key(module: &BytecodeModule) -> ModuleCacheKey {
    let mut name_fingerprint = 0u64;
    for name in module.functions.keys() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        name_fingerprint ^= hasher.finish();
    }
    ModuleCacheKey {
        ptr: module as *const BytecodeModule as usize,
        len: module.functions.len(),
        name_fingerprint,
    }
}

pub(super) fn resolve_chunk<'a>(
    module: &'a BytecodeModule,
    callee_name: &str,
    site: Site<'_>,
) -> Result<&'a FunctionChunk, VmError> {
    module.functions.get(callee_name).ok_or_else(|| {
        super::err_at(
            VmErrorKind::UnknownFunction,
            format!("Unknown function `{callee_name}`"),
            site.function_name,
            site.ip,
        )
    })
}

pub(super) fn resolve_chunk_idx<'a>(
    fn_table: &'a [&'a FunctionChunk],
    callee_idx: usize,
    site: Site<'_>,
) -> Result<&'a FunctionChunk, VmError> {
    fn_table.get(callee_idx).copied().ok_or_else(|| {
        super::err_at(
            VmErrorKind::UnknownFunction,
            format!("Invalid function index `{callee_idx}`"),
            site.function_name,
            site.ip,
        )
    })
}

pub(super) fn resolve_function_value<'a>(
    module: &'a BytecodeModule,
    fn_table: &'a [&'a FunctionChunk],
    callee: Value,
    site: Site<'_>,
) -> Result<&'a FunctionChunk, VmError> {
    match callee {
        Value::FunctionIdx(callee_idx) => resolve_chunk_idx(fn_table, callee_idx, site),
        Value::Function(callee_name) => {
            module.functions.get(callee_name.as_ref()).ok_or_else(|| {
                super::err_at(
                    VmErrorKind::UnknownFunction,
                    format!("Unknown function `{callee_name}`"),
                    site.function_name,
                    site.ip,
                )
            })
        }
        _ => Err(super::err_at(
            VmErrorKind::TypeMismatch,
            "CallValue callee must be Function",
            site.function_name,
            site.ip,
        )),
    }
}

fn resolve_method_idx(
    module: &BytecodeModule,
    fn_table: &[&FunctionChunk],
    struct_name: &str,
    method_name: &str,
) -> Option<usize> {
    static METHOD_CACHE: OnceLock<Mutex<MethodCache>> = OnceLock::new();
    let cache = METHOD_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let module_key = module_cache_key(module);

    {
        let cache = cache.lock().expect("method cache poisoned");
        if let Some(idx) = cache
            .get(&module_key)
            .and_then(|methods| methods.get(struct_name))
            .and_then(|methods| methods.get(method_name))
            .copied()
        {
            return Some(idx);
        }
    }

    let mangled = format!("__impl_{struct_name}__{method_name}");
    let idx = fn_table.iter().position(|chunk| chunk.name == mangled)?;

    let mut cache = cache.lock().expect("method cache poisoned");
    cache
        .entry(module_key)
        .or_default()
        .entry(struct_name.to_string())
        .or_default()
        .insert(method_name.to_string(), idx);
    Some(idx)
}

pub(super) fn resolve_method<'a>(
    module: &'a BytecodeModule,
    fn_table: &'a [&'a FunctionChunk],
    receiver: &Value,
    method_name: &str,
    site: Site<'_>,
) -> Result<&'a FunctionChunk, VmError> {
    let Value::Struct { shape, .. } = receiver else {
        return Err(super::err_at(
            VmErrorKind::TypeMismatch,
            "CallMethod receiver must be Struct",
            site.function_name,
            site.ip,
        ));
    };
    let Some(callee_idx) = resolve_method_idx(module, fn_table, &shape.name, method_name) else {
        return Err(super::err_at(
            VmErrorKind::UnknownFunction,
            format!(
                "Unknown method `{}` on struct `{}`",
                method_name, shape.name
            ),
            site.function_name,
            site.ip,
        ));
    };
    Ok(fn_table[callee_idx])
}

pub(super) fn resolve_method_id<'a>(
    module: &'a BytecodeModule,
    fn_table: &'a [&'a FunctionChunk],
    receiver: &Value,
    method_id: usize,
    site: Site<'_>,
) -> Result<&'a FunctionChunk, VmError> {
    static METHOD_ID_CACHE: OnceLock<Mutex<MethodIdCache>> = OnceLock::new();
    let cache = METHOD_ID_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let module_key = module_cache_key(module);
    let Value::Struct { shape, .. } = receiver else {
        return Err(super::err_at(
            VmErrorKind::TypeMismatch,
            "CallMethod receiver must be Struct",
            site.function_name,
            site.ip,
        ));
    };

    {
        let cache = cache.lock().expect("method id cache poisoned");
        if let Some(idx) = cache
            .get(&module_key)
            .and_then(|methods| methods.get(&shape.name))
            .and_then(|methods| methods.get(&method_id))
            .copied()
        {
            return Ok(fn_table[idx]);
        }
    }

    let Some(method_name) = module.method_names.get(method_id) else {
        return Err(super::err_at(
            VmErrorKind::UnknownFunction,
            format!("Unknown method id `{method_id}`"),
            site.function_name,
            site.ip,
        ));
    };
    let Some(callee_idx) = resolve_method_idx(module, fn_table, &shape.name, method_name) else {
        return Err(super::err_at(
            VmErrorKind::UnknownFunction,
            format!(
                "Unknown method `{}` on struct `{}`",
                method_name, shape.name
            ),
            site.function_name,
            site.ip,
        ));
    };

    let mut cache = cache.lock().expect("method id cache poisoned");
    cache
        .entry(module_key)
        .or_default()
        .entry(shape.name.clone())
        .or_default()
        .insert(method_id, callee_idx);
    Ok(fn_table[callee_idx])
}

pub(super) fn call_builtin(
    stack: &mut Vec<Value>,
    package: &str,
    name: &str,
    argc: usize,
    env: &mut CallEnv<'_>,
    site: Site<'_>,
) -> Result<(), VmError> {
    if stack.len() < argc {
        return Err(super::err_at(
            VmErrorKind::StackUnderflow,
            "Stack underflow on CallBuiltin",
            site.function_name,
            site.ip,
        ));
    }
    let call_args = take_call_args(stack, argc);
    let ret = env.reg.call(env.host, package, name, call_args)?;
    stack.push(ret);
    Ok(())
}

pub(super) fn call_builtin_id(
    stack: &mut Vec<Value>,
    id: u16,
    argc: usize,
    env: &mut CallEnv<'_>,
    site: Site<'_>,
) -> Result<(), VmError> {
    if stack.len() < argc {
        return Err(super::err_at(
            VmErrorKind::StackUnderflow,
            "Stack underflow on CallBuiltinId",
            site.function_name,
            site.ip,
        ));
    }
    let call_args = take_call_args(stack, argc);
    let ret = env.reg.call_by_id(env.host, id, call_args)?;
    stack.push(ret);
    Ok(())
}

pub(super) struct DispatchCtx<'a, 'b> {
    pub module: &'a BytecodeModule,
    pub fn_table: &'a [&'a FunctionChunk],
    pub host: &'b mut dyn BuiltinHost,
    pub reg: &'b BuiltinRegistry,
    pub current_depth: usize,
    pub opts: CallOptions,
    pub function_name: &'b str,
    pub ip: usize,
    pub locals_pool: &'b mut Vec<Vec<Value>>,
    pub stack_pool: &'b mut Vec<Vec<Value>>,
}

pub(super) fn handle_call_instr<'a>(
    frame: &mut state::CallFrame<'_>,
    instr: &Instr,
    ctx: DispatchCtx<'a, '_>,
) -> Result<Option<state::CallFrame<'a>>, VmError> {
    match instr {
        Instr::Call {
            name: callee_name,
            argc,
        } => {
            if frame.stack.len() < *argc {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on Call",
                    ctx.function_name,
                    ctx.ip,
                ));
            }
            let callee_chunk = resolve_chunk(
                ctx.module,
                callee_name,
                Site {
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                },
            )?;
            if ctx.current_depth + ctx.opts.depth >= ctx.opts.config.max_call_depth {
                return Err(VmError::new(
                    VmErrorKind::StackOverflow,
                    format!(
                        "Call stack limit exceeded ({})",
                        ctx.opts.config.max_call_depth
                    ),
                ));
            }
            if *argc != callee_chunk.param_count {
                return Err(VmError::new(
                    VmErrorKind::ArityMismatch,
                    format!(
                        "Function `{}` arity mismatch: expected {}, got {}",
                        callee_name, callee_chunk.param_count, argc
                    ),
                ));
            }
            frame.ip += 1;
            Ok(Some(frame::push_call_frame(
                frame,
                callee_chunk,
                *argc,
                None,
                ctx.locals_pool,
                ctx.stack_pool,
            )))
        }
        Instr::CallIdx { idx, argc } => {
            let new_frame = frame::call_idx_fast(
                frame,
                frame::IndexedCallCtx {
                    fn_table: ctx.fn_table,
                    idx: *idx,
                    argc: *argc,
                    current_depth: ctx.current_depth,
                    opts: super::RunOptions {
                        depth: ctx.opts.depth,
                        config: ctx.opts.config,
                    },
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                    locals_pool: ctx.locals_pool,
                    stack_pool: ctx.stack_pool,
                },
            )?;
            Ok(Some(new_frame))
        }
        Instr::CallIdxAddConst(rhs) => {
            let Some(value) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on CallIdxAddConst",
                    ctx.function_name,
                    ctx.ip,
                ));
            };
            match value {
                Value::Int(lhs) => frame.stack.push(Value::Int(lhs + rhs)),
                _ => {
                    return Err(err_at(
                        VmErrorKind::TypeMismatch,
                        "CallIdxAddConst expects Int argument",
                        ctx.function_name,
                        ctx.ip,
                    ));
                }
            }
            Ok(None)
        }
        Instr::CallIdxStructFieldAdd(field_slot) => {
            let Some(arg) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on CallIdxStructFieldAdd arg",
                    ctx.function_name,
                    ctx.ip,
                ));
            };
            let Some(receiver) = frame.stack.pop() else {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on CallIdxStructFieldAdd receiver",
                    ctx.function_name,
                    ctx.ip,
                ));
            };
            let Value::Struct { fields, .. } = receiver else {
                return Err(err_at(
                    VmErrorKind::TypeMismatch,
                    "CallIdxStructFieldAdd expects Struct receiver",
                    ctx.function_name,
                    ctx.ip,
                ));
            };
            let Some(field_value) = fields.get(*field_slot) else {
                return Err(err_at(
                    VmErrorKind::TypeMismatch,
                    format!("Unknown struct field slot `{field_slot}`"),
                    ctx.function_name,
                    ctx.ip,
                ));
            };
            match (field_value, arg) {
                (Value::Int(lhs), Value::Int(rhs)) => frame.stack.push(Value::Int(*lhs + rhs)),
                _ => {
                    return Err(err_at(
                        VmErrorKind::TypeMismatch,
                        "CallIdxStructFieldAdd expects Int field and Int argument",
                        ctx.function_name,
                        ctx.ip,
                    ));
                }
            }
            Ok(None)
        }
        Instr::CallValue { argc } => {
            if frame.stack.len() < *argc + 1 {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on CallValue",
                    ctx.function_name,
                    ctx.ip,
                ));
            }
            let callee_index = frame.stack.len() - *argc - 1;
            let callee = frame.stack.remove(callee_index);
            let callee_chunk = resolve_function_value(
                ctx.module,
                ctx.fn_table,
                callee,
                Site {
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                },
            )?;
            if ctx.current_depth + ctx.opts.depth >= ctx.opts.config.max_call_depth {
                return Err(VmError::new(
                    VmErrorKind::StackOverflow,
                    format!(
                        "Call stack limit exceeded ({})",
                        ctx.opts.config.max_call_depth
                    ),
                ));
            }
            if *argc != callee_chunk.param_count {
                return Err(VmError::new(
                    VmErrorKind::ArityMismatch,
                    format!(
                        "Function `{}` arity mismatch: expected {}, got {}",
                        callee_chunk.name, callee_chunk.param_count, argc
                    ),
                ));
            }
            frame.ip += 1;
            Ok(Some(frame::push_call_frame(
                frame,
                callee_chunk,
                *argc,
                None,
                ctx.locals_pool,
                ctx.stack_pool,
            )))
        }
        Instr::CallMethod {
            name: method_name,
            argc,
        } => {
            if frame.stack.len() < *argc + 1 {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on CallMethod",
                    ctx.function_name,
                    ctx.ip,
                ));
            }
            let receiver_index = frame.stack.len() - *argc - 1;
            let receiver = frame.stack.remove(receiver_index);
            let callee_chunk = resolve_method(
                ctx.module,
                ctx.fn_table,
                &receiver,
                method_name,
                Site {
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                },
            )?;
            if ctx.current_depth + ctx.opts.depth >= ctx.opts.config.max_call_depth {
                return Err(VmError::new(
                    VmErrorKind::StackOverflow,
                    format!(
                        "Call stack limit exceeded ({})",
                        ctx.opts.config.max_call_depth
                    ),
                ));
            }
            if *argc + 1 != callee_chunk.param_count {
                return Err(VmError::new(
                    VmErrorKind::ArityMismatch,
                    format!(
                        "Function `{}` arity mismatch: expected {}, got {}",
                        callee_chunk.name,
                        callee_chunk.param_count,
                        argc + 1
                    ),
                ));
            }
            frame.ip += 1;
            Ok(Some(frame::push_call_frame(
                frame,
                callee_chunk,
                *argc,
                Some(receiver),
                ctx.locals_pool,
                ctx.stack_pool,
            )))
        }
        Instr::CallMethodId { id, argc } => {
            if frame.stack.len() < *argc + 1 {
                return Err(err_at(
                    VmErrorKind::StackUnderflow,
                    "Stack underflow on CallMethodId",
                    ctx.function_name,
                    ctx.ip,
                ));
            }
            let receiver_index = frame.stack.len() - *argc - 1;
            let receiver = frame.stack.remove(receiver_index);
            let callee_chunk = resolve_method_id(
                ctx.module,
                ctx.fn_table,
                &receiver,
                *id,
                Site {
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                },
            )?;
            if ctx.current_depth + ctx.opts.depth >= ctx.opts.config.max_call_depth {
                return Err(VmError::new(
                    VmErrorKind::StackOverflow,
                    format!(
                        "Call stack limit exceeded ({})",
                        ctx.opts.config.max_call_depth
                    ),
                ));
            }
            if *argc + 1 != callee_chunk.param_count {
                return Err(VmError::new(
                    VmErrorKind::ArityMismatch,
                    format!(
                        "Function `{}` arity mismatch: expected {}, got {}",
                        callee_chunk.name,
                        callee_chunk.param_count,
                        argc + 1
                    ),
                ));
            }
            frame.ip += 1;
            Ok(Some(frame::push_call_frame(
                frame,
                callee_chunk,
                *argc,
                Some(receiver),
                ctx.locals_pool,
                ctx.stack_pool,
            )))
        }
        Instr::CallBuiltin {
            package,
            name,
            argc,
        } => {
            call_builtin(
                &mut frame.stack,
                package,
                name,
                *argc,
                &mut CallEnv {
                    host: ctx.host,
                    reg: ctx.reg,
                },
                Site {
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                },
            )?;
            Ok(None)
        }
        Instr::CallBuiltinId { id, argc } => {
            call_builtin_id(
                &mut frame.stack,
                *id,
                *argc,
                &mut CallEnv {
                    host: ctx.host,
                    reg: ctx.reg,
                },
                Site {
                    function_name: ctx.function_name,
                    ip: ctx.ip,
                },
            )?;
            Ok(None)
        }
        _ => Ok(None),
    }
}
