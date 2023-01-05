use crate::Block;
use inkwell::values::GlobalValue;
use inkwell::{builder::*, context::*, module::*, AddressSpace};

pub fn generate_llvm<'ctx>(
    code: Vec<Block>,
    ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
) {
    let memory = create_memory_u8(ctx, module);
    let function_type = ctx.i32_type().fn_type(&[], false);
    let fun = module.add_function("entry", function_type, None);
    let getch = module.add_function(
        "getchar",
        ctx.i8_type().fn_type(&[], false),
        Some(Linkage::External),
    );
    for code_block in code {
        match code_block {
            Block::Block { .. } => {}
            Block::Input => {}
            Block::Output => {}
            Block::LoopOpen(value) => {}
            Block::LoopClose(_) => {}
            Block::Left => {}
            Block::Right => {}
        }
    }
}

fn create_memory_u8<'ctx>(ctx: &'ctx Context, module: Module<'ctx>) -> GlobalValue<'ctx> {
    let i8_type = ctx.i8_type();
    let i8_ptr = i8_type.get_pointer_type();
    module.add_global(i8_ptr, None, "memory")
}
