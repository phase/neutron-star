use melior::{
    Context,
    dialect::{arith, DialectRegistry, func},
    ir::{*, attribute::*, r#type::{FunctionType, MemRefType, IntegerType}, operation::*},
    utility::register_all_dialects,
    pass::{self, PassManager},
};
use mlir_sys::*;

fn main() {
    println!("Hello, world!");

    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let location = Location::unknown(&context);
    let mut module = Module::new(location);

    let _index_type = Type::index(&context);
    let float32_type = Type::float32(&context);
    let vector2_float32_type = Type::vector(&[2], float32_type);
    let i64_type: Type = IntegerType::new(&context, 64).into();
    let _memref_load_type: Type = MemRefType::new(vector2_float32_type, &[1], None, None).into();

    // see https://github.com/raviqqe/melior/issues/180
    let array_attr: Attribute = unsafe {
        let raw_attr = mlirArrayAttrGet(context.to_raw(), 1, &IntegerAttribute::new(0, i64_type).to_raw());
        Attribute::from_raw(raw_attr)
    };

    // add two floats together
    module.body().append_operation(func::func(
        &context,
        StringAttribute::new(&context, "add"),
        TypeAttribute::new(FunctionType::new(&context, &[float32_type, float32_type], &[float32_type]).into()),
        {
            let block = Block::new(&[(float32_type, location), (float32_type, location)]);
            let sum = block.append_operation(arith::addf(
                block.argument(0).unwrap().into(),
                block.argument(1).unwrap().into(),
                location
            ));

            block.append_operation(func::r#return(&[sum.result(0).unwrap().into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        location
    ));

    // testing vector ops
    module.body().append_operation(func::func(
        &context,
        StringAttribute::new(&context, "firstInVector"),
        TypeAttribute::new(FunctionType::new(&context, &[vector2_float32_type], &[float32_type]).into()),
        {
            // block arguments must match type attribute arguments
            let block = Block::new(&[(vector2_float32_type, location)]);

            let vector_extract_op = OperationBuilder::new("vector.extract", location)
                .add_attributes(&[(
                    Identifier::new(&context, "position"),
                    array_attr
                )])
                .add_operands(&[block.argument(0).unwrap().into()])
                .add_results(&[float32_type])
                .build();
            let vector_extract_op = block.append_operation(vector_extract_op);

            block.append_operation(func::r#return(&[vector_extract_op.result(0).unwrap().into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        location
    ));

    let module_op = module.as_operation();
    module_op.dump();
    assert!(module_op.verify());

    let pass_manager = PassManager::new(&context);
    pass_manager.add_pass(pass::conversion::create_arith_to_llvm());
    pass_manager.add_pass(pass::conversion::create_math_to_llvm());
    pass_manager.add_pass(pass::conversion::create_func_to_llvm());
    pass_manager.add_pass(pass::conversion::create_vector_to_llvm());
    pass_manager.add_pass(pass::conversion::create_index_to_llvm_pass());
   /* pass_manager.add_pass(pass::conversion::create_tensor_to_linalg());
    pass_manager.add_pass(pass::conversion::create_linalg_to_standard());
    pass_manager.add_pass(pass::conversion::create_linalg_to_llvm());
    pass_manager.add_pass(pass::conversion::create_mem_ref_to_llvm());
    pass_manager.add_pass(pass::conversion::create_gpu_to_llvm()); */
    pass_manager.run(&mut module).unwrap();

    let module_op = module.as_operation();
    module_op.dump();
    assert!(module_op.verify());
}
