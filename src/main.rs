use melior::{
    Context,
    dialect::{arith, DialectRegistry, func, index},
    ir::{*, attribute::*, r#type::{FunctionType, MemRefType, IntegerType, RankedTensorType}, operation::*},
    utility::register_all_dialects,
    pass::{self, PassManager},
};
use mlir_sys::*;
use crate::{compiler::*};

mod ast;
mod parser;
mod ir;
mod compiler;
mod diagnostic;

fn main() {
    let test_file = "test.ns";
    let test_source = "\
    actor Foo {
        async fun foo(a: &iso Int32, b: &val Int32) {
        }

        async fun bar(a: *Int32, b: *Int32) {
            unsafe {
                let x: Int32 = unsafe {a.*};
            }
        }
    }

    interface CoolInterface {
        fun publicFunc(x: Int32, y: Int32);
    }

    public struct CoolApi {
        public let CONSTANT = 7;
    }

    struct X {
        let x = 2;

        fun test(): Int32 {
            return 7;
        }
    }

    fun testRefinement(a: (v: Int32 where v >= 0 and v <= 10 + 7), b): Int32 {
        let x = 1;
        let y = 0;
        if x < a {
            y = 7;
        } else if x >= 500 {
            y = 5;
        } else if x >= a and x < b {
            y = 9;
        } else {
            y = b;
        }
        return y;
    }

    fun testRow(x: {field1: Int32, field2: Int32}) {
        let y = test((x.field1), x.field2);
    }

    type Nat32 = (v: Int32 where v >= 0);
    struct Box {
        let x: Int32;
    }
    type PosBox = (b: Box where b.x >= 0);
    type PosBox2 = (b: {x: Int32} where b.x >= 0);
    type PosBox3 = Box where it.x >= 0;
    unique type Meters = Int32;

    public fun refTest(arena: ArenaAllocator): X {
        return new X in arena;
    }

    public fun buildX2(arena: &mut ArenaAllocator): X {
        return new X in arena;
    }

    public fun buildX3(arena: &Allocator): X {
        return new X in arena;
    }

    public fun derefX(refX: &X): X {
        return x.*;
    }

    public fun derefX2(refX: &?X): X {
        return x.*.?;
    }

    public fun derefX2(refX: &?&?X): X {
        let xRefCopy = refX.&.*.&.*;
        return x.*.?.*.?;
    }

    fun add(x, y) {
        return x + y;
    }

    enum Node {
        Point(x: Int32, y: Int32, next: &Node),
        Nil
    }

    fun max(x, y) {
        if x > y {
            return x;
        } else {
            return y;
        }
    }

    fun max2(x: Int32, y: Int32): (ret: Int32 where x <= v or y <= v) {
        if x > y { return x; }
        else { return y; }
    }

    fun sum(k) {
        if k < 0 {
            return 0;
        } else {
            let s = sum(k - 1);
            return s + k;
        }
    }

    fun sum2(k: Int32): (ret: Int32 where 0 <= ret and k <= ret) {
        if k < 0 {
            return 0;
        } else {
            let s = sum(k - 1);
            return s + k;
        }
    }

    fun loop(n, i, c, f) {
        if i < n {
            return loop(n, i + 1, f(i, c), f);
        } else {
            return c;
        }
    }

    fun foldn(n, b, f) {
        return loop(n, 0, b, f);
    }

    public fun foldn2[A](n: Int32, b: A, f: (Int32 where 0 <= it or it < n, A) -> A): A {
        return loop(n, 0, b, f);
    }

    ".to_string();

    let mut compiler = Compiler::new();
    compiler.parse_module(ast::Path::of("test"), test_file.to_string(), test_source);
    for (_index, module) in compiler.modules.iter() {
        println!("Parsed nodes: {}", module.module_arena.node_arena.len());
        println!("Parsed blocks: {}", module.module_arena.block_arena.len());
        println!("Parsed instructions: {}", module.module_arena.instruction_arena.len());
        println!("Parsed types: {}", module.module_arena.type_arena.len());
    }

    println!("parse complete!")
}

fn _main() {
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
    let vector2_float32_type = Type::vector(&[2], float32_type);
    let i64_type: Type = IntegerType::new(&context, 64).into();
    let tensor_type: Type = RankedTensorType::new(&[1], float32_type, None).into();
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

     // testing tensor ops
     module.body().append_operation(func::func(
        &context,
        StringAttribute::new(&context, "firstInTensor"),
        TypeAttribute::new(FunctionType::new(&context, &[tensor_type], &[float32_type]).into()),
        {
            // block arguments must match type attribute arguments
            let block = Block::new(&[(tensor_type, location)]);

            let constant_op = index::constant(&context, IntegerAttribute::new(0, index_type), location);
            let constant_op = block.append_operation(constant_op);

            let tensor_extract_op = OperationBuilder::new("tensor.extract", location)
                .add_operands(&[
                    // tensor: ranked tensor of any type values
                    block.argument(0).unwrap().into(),
                    // indices: index
                    constant_op.result(0).unwrap().into()
                ])
                .add_results(&[float32_type])
                .build();
            let tensor_extract_op = block.append_operation(tensor_extract_op);

            block.append_operation(func::r#return(&[tensor_extract_op.result(0).unwrap().into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        location
    ));

    let module_op = module.as_operation();
    module_op.dump();
    assert!(module_op.verify());

    if true {
        // llvm
        let pass_manager = PassManager::new(&context);
        pass_manager.add_pass(pass::conversion::create_arith_to_llvm());
        pass_manager.add_pass(pass::conversion::create_math_to_llvm());
        pass_manager.add_pass(pass::conversion::create_func_to_llvm());
        pass_manager.add_pass(pass::conversion::create_vector_to_llvm());
        pass_manager.add_pass(pass::conversion::create_tensor_to_linalg());
        pass_manager.add_pass(pass::conversion::create_linalg_to_llvm());
        pass_manager.add_pass(pass::conversion::create_index_to_llvm_pass());
        /* pass_manager.add_pass(pass::conversion::create_tensor_to_linalg());
        pass_manager.add_pass(pass::conversion::create_linalg_to_standard());
        pass_manager.add_pass(pass::conversion::create_linalg_to_llvm());
        pass_manager.add_pass(pass::conversion::create_mem_ref_to_llvm());
        pass_manager.add_pass(pass::conversion::create_gpu_to_llvm()); */
        pass_manager.run(&mut module).unwrap();
    } else {
        // spirv
        let pass_manager = PassManager::new(&context);
        pass_manager.add_pass(pass::conversion::create_arith_to_spirv());
        pass_manager.add_pass(pass::conversion::create_math_to_spirv());
        pass_manager.add_pass(pass::conversion::create_func_to_spirv());
        pass_manager.add_pass(pass::conversion::create_vector_to_spirv());
        //pass_manager.add_pass(pass::conversion::create_index_to_llvm_pass());
        /* pass_manager.add_pass(pass::conversion::create_tensor_to_linalg());
        pass_manager.add_pass(pass::conversion::create_linalg_to_standard());
        pass_manager.add_pass(pass::conversion::create_linalg_to_llvm());
        pass_manager.add_pass(pass::conversion::create_mem_ref_to_llvm());
        pass_manager.add_pass(pass::conversion::create_gpu_to_llvm()); */
        pass_manager.run(&mut module).unwrap();
    }

    let module_op = module.as_operation();
    module_op.dump();
    assert!(module_op.verify())
}
