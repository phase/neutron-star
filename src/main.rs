use melior::{
    Context,
    dialect::{arith, DialectRegistry, func},
    ir::{*, attribute::{StringAttribute, TypeAttribute}, r#type::FunctionType},
    utility::register_all_dialects,
    pass::{self, PassManager},
};

fn main() {
    println!("Hello, world!");

    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let location = Location::unknown(&context);
    let mut module = Module::new(location);

    let index_type = Type::index(&context);
    let float32_type = Type::float32(&context);

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

    let pass_manager = PassManager::new(&context);
    pass_manager.add_pass(pass::conversion::create_arith_to_llvm());
    pass_manager.add_pass(pass::conversion::create_func_to_llvm());
    pass_manager.run(&mut module).unwrap();

    let module_op = module.as_operation();
    module_op.dump();
    assert!(module_op.verify());
}
