mod block;
mod calls;
mod compare;
mod context;
mod runtime;
mod runtime_boxing;
mod runtime_decls;
mod strings;
mod types;
mod value;

use crate::codegen::CodegenError;
use crate::ir::IrProgram;

pub fn compile_program(program: &IrProgram) -> Result<String, CodegenError> {
    context::LlvmEmitter::new(program).emit_program()
}
