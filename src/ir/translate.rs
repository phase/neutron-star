use crate::ast::{AstFunction, Expression, Node, Program, Statement, StatementIndex, Type, TypedName, TypeIndex};
use crate::ir::*;

pub struct IrBuilderContext<'ctx> {
    program: &'ctx Program,
    module_arena: ModuleArena,
    void_index: IrTypeIndex,
    unknown_index: IrTypeIndex,
}

impl<'ctx> IrBuilderContext<'ctx> {
    pub fn new(program: &'ctx Program) -> IrBuilderContext {
        let mut module_arena = ModuleArena::new();

        let void_index = module_arena.type_arena.insert(IrType::Void);
        let unknown_index = module_arena.type_arena.insert(IrType::Unknown);

        IrBuilderContext {
            program,
            module_arena: ModuleArena::new(),
            void_index,
            unknown_index,
        }
    }

    pub fn new_block(&mut self) -> IrBlockIndex {
        self.module_arena.block_arena.insert(IrBlock::new())
    }

    /// Insert an instruction into the instruction arena and add its index to the provided block.
    /// This ensures that all IrInstructions are allocated into some IrBlock.
    /// The returned index can be used in other instructions.
    pub fn ins(&mut self, block: IrBlockIndex, ins: IrInstruction) -> IrInstructionIndex {
        let index = self.module_arena.instruction_arena.insert(ins);
        self.module_arena.block_arena.get_mut(block).unwrap().instructions.push(index);
        index
    }
}

pub struct IrBuilder {}

impl IrBuilder {
    pub fn new() -> IrBuilder {
        IrBuilder {}
    }

    pub fn convert(&self, program: Program) -> Module {
        let mut ctx = IrBuilderContext::new(&program);
        for (_index, node) in program.program_arena.node_arena.iter() {
            match node {
                Node::TypeAlias { .. } => {}
                Node::Variable { .. } => {}
                Node::Function(ast_function) => {
                    let node = self.build_function(&mut ctx, ast_function);
                    ctx.module_arena.node_arena.insert(node);
                }
                Node::FunctionPrototype { .. } => {}
                Node::Struct { .. } => {}
                Node::Enum { .. } => {}
                Node::Interface { .. } => {}
                Node::Error => {}
            }
        }
        Module {
            path: program.path.clone(),
            name: program.file_name.clone(),
            imports: program.imports.clone(),
            module_arena: ctx.module_arena,
        }
    }

    fn build_type(&self, ctx: &mut IrBuilderContext, ast_type: &TypeIndex) -> IrTypeIndex {
        if let Some(ast_type) = ctx.program.program_arena.type_arena.get(*ast_type) {
            match ast_type {
                Type::Base(name) => {
                    if let Some(int_type) = IntTy::from(&name.name) {
                        ctx.module_arena.type_arena.insert(IrType::Int(int_type))
                    } else if let Some(int_type) = UIntTy::from(&name.name) {
                        ctx.module_arena.type_arena.insert(IrType::UInt(int_type))
                    } else if let Some(float_type) = FloatTy::from(&name.name) {
                        ctx.module_arena.type_arena.insert(IrType::Float(float_type))
                    } else if "Void" == name.name {
                        ctx.void_index
                    } else {
                        ctx.unknown_index
                    }
                }
                Type::Refinement(_, _, _) => ctx.unknown_index,
                Type::Row(_) => ctx.unknown_index,
                Type::Reference(_, _) => ctx.unknown_index,
                Type::Optional(_) => ctx.unknown_index,
                Type::Function(_, _) => ctx.unknown_index,
            }
        } else {
            ctx.unknown_index
        }
    }

    fn build_typed_name(&self, ctx: &mut IrBuilderContext, ast_typed_name: &TypedName) -> IrTypedName {
        IrTypedName {
            typ: ast_typed_name.typ.map_or(ctx.void_index, |ty| self.build_type(ctx, &ty)),
            name: ast_typed_name.name.clone(),
        }
    }

    fn build_function(&self, ctx: &mut IrBuilderContext, func: &AstFunction) -> IrNode {
        let mut blocks = vec![];
        let mut current_block = ctx.new_block();
        blocks.push(current_block.clone());

        for s_index in &func.statements {
            self.build_statement(ctx, func, s_index, &mut current_block);
        }
        IrNode::Function(IrFunction {
            access: Access::from(func.access),
            name: func.name.clone(),
            type_params: vec![],
            return_type: self.build_type(ctx, &func.return_type),
            blocks,
        })
    }

    fn build_statement(&self, ctx: &mut IrBuilderContext, func: &AstFunction, s_index: &StatementIndex, current_block: &mut IrBlockIndex) {
        let stmt = ctx.program.statement(s_index.clone());
        match stmt {
            Statement::If { condition, body, else_if } => {
                let cond_ins = self.build_expression(ctx, func, stmt, condition, current_block);
                // make the blocks we can branch to
                let true_branch = ctx.new_block();
                let false_branch = ctx.new_block();

                // add the branch ins to the current block
                let branch = IrInstruction::Branch {
                    condition: cond_ins,
                    true_branch,
                    false_branch,
                };
                ctx.ins(*current_block, branch);

                // build the true block
                *current_block = true_branch;
                for stmt in body {
                    self.build_statement(ctx, func, stmt, current_block);
                }

                // build the false block
                *current_block = false_branch;
                if let Some(stmt) = else_if {
                    self.build_statement(ctx, func, stmt, current_block);
                }
            }
            Statement::Call { function, args } => {
                let fun_ins = self.build_expression(ctx, func, stmt, function, current_block);
                let mut arg_insx = Vec::with_capacity(args.len());
                for arg in args {
                    let arg_ins = self.build_expression(ctx, func, stmt, arg, current_block);
                    arg_insx.push(arg_ins);
                }
                ctx.ins(*current_block, IrInstruction::FunctionCall {
                    function: fun_ins,
                    args: arg_insx,
                });
            }
            Statement::Let { .. } => {}
            Statement::Assign { .. } => {}
            Statement::Return { value } => {
                let value_ins = self.build_expression(ctx, func, stmt, value, current_block);
                ctx.ins(*current_block, IrInstruction::Return {
                    value: value_ins
                });
            }
        }
    }

    fn build_expression(&self, ctx: &mut IrBuilderContext, func: &AstFunction,
                        stmt: &Statement, exp: &ExpressionIndex, current_block: &mut IrBlockIndex) -> IrInstructionIndex {
        let exp = ctx.program.expression(*exp);
        // let todo = IrInstruction::Ref("TODO".to_string());
        let ins = match exp {
            Expression::Ref(s) => IrInstruction::Ref(s.clone()),
            Expression::NatLiteral(i) => IrInstruction::NatLiteral(i.clone()),
            Expression::BoolLiteral(b) => IrInstruction::BoolLiteral(b.clone()),
            Expression::BinOp(lhs, op, rhs) => {
                let lhs_ins = self.build_expression(ctx, func, stmt, lhs, current_block);
                let rhs_ins = self.build_expression(ctx, func, stmt, rhs, current_block);
                IrInstruction::BinOp(lhs_ins, op.clone(), rhs_ins)
            }
            Expression::FieldAccessor { aggregate, value } => {
                let agg_ins = self.build_expression(ctx, func, stmt, aggregate, current_block);
                let value_ins = self.build_expression(ctx, func, stmt, value, current_block);
                IrInstruction::FieldAccessor {
                    aggregate: agg_ins,
                    value: value_ins,
                }
            }
            Expression::FunctionCall { function, args } => {
                let fun_ins = self.build_expression(ctx, func, stmt, function, current_block);
                let mut arg_insx = Vec::with_capacity(args.len());
                for arg in args {
                    let arg_ins = self.build_expression(ctx, func, stmt, arg, current_block);
                    arg_insx.push(arg_ins);
                }
                IrInstruction::FunctionCall {
                    function: fun_ins,
                    args: arg_insx,
                }
            }
            Expression::New { typ, allocator } => {
                let alloc_ins = self.build_expression(ctx, func, stmt, allocator, current_block);
                IrInstruction::New {
                    typ: self.build_type(ctx, typ),
                    allocator: alloc_ins,
                }
            }
            Expression::Dereference { pointer } => {
                let pointer_ins = self.build_expression(ctx, func, stmt, pointer, current_block);
                IrInstruction::Dereference { pointer: pointer_ins }
            }
            Expression::Denull { optional } => {
                let optional_ins = self.build_expression(ctx, func, stmt, optional, current_block);
                IrInstruction::Denull { optional: optional_ins }
            }
            Expression::Borrow { value } => {
                let value_ins = self.build_expression(ctx, func, stmt, value, current_block);
                IrInstruction::Borrow { value: value_ins }
            }
        };
        ctx.ins(*current_block, ins)
    }
}
