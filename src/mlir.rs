use melior::{
    Context,
    dialect::{arith, DialectRegistry, func, index},
    ir::{*, attribute::*, r#type::{FunctionType, MemRefType, IntegerType, RankedTensorType}, operation::*},
    utility::register_all_dialects,
    pass::{self, PassManager},
};
use mlir_sys::*;

use generational_arena::Index;
use crate::compiler::Compiler;

impl Compiler {
    pub fn create_mlir_module(&mut self, module_index: Index) {
        if let Some(module) = self.modules.get(module_index) {
            s
        }
    }

    fn mlir_module(&mut self) {
        let registry = DialectRegistry::new();
        register_all_dialects(&registry);

        let context = Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();

        let location = Location::unknown(&context);
        let mut module = Module::new(location);

        let index_type = Type::index(&context);
        let float32_type = Type::float32(&context);
        let vector2_float32_type = Type::vector(&[2], float32_type);
        let i64_type: Type = IntegerType::new(&context, 64).into();
        let tensor_type: Type = RankedTensorType::new(&[1], float32_type, None).into();
        let _memref_load_type: Type = MemRefType::new(vector2_float32_type, &[1], None, None).into();

        // see https://github.com/raviqqe/melior/issues/180
    }
}
