use crate::lang::{Path, refcap::ReferenceCapability, ptr::PointerKind};
use std::fmt;
use std::fmt::Formatter;
use generational_arena::{Arena, Index};

pub type TypeIndex = Index;
pub type NodeIndex = Index;
pub type StatementIndex = Index;
pub type ExpressionIndex = Index;

#[derive(Clone, Debug)]
pub struct ProgramArena {
    pub type_arena: Arena<Type>,
    pub node_arena: Arena<Node>,
    pub statement_arena: Arena<Statement>,
    pub expression_arena: Arena<Expression>,
}

impl ProgramArena {
    pub fn new() -> ProgramArena {
        ProgramArena {
            type_arena: Arena::new(),
            node_arena: Arena::new(),
            statement_arena: Arena::new(),
            expression_arena: Arena::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub path: Path,
    pub file_name: String,
    pub imports: Vec<Path>,
    pub program_arena: ProgramArena,
}

impl Program {
    pub fn statement(&self, index: StatementIndex) -> &Statement {
        self.program_arena.statement_arena.get(index).unwrap()
    }

    pub fn expression(&self, index: ExpressionIndex) -> &Expression {
        self.program_arena.expression_arena.get(index).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct TypedName {
    pub name: String,
    pub typ: Option<TypeIndex>,
}

#[derive(Clone, Debug)]
pub enum Type {
    Base(TypeName),
    Refinement(String, TypeIndex, ExpressionIndex),
    Row(Vec<TypedName>),
    Reference(TypeIndex, PointerKind, ReferenceCapability),
    Optional(TypeIndex),
    Function(Vec<TypeIndex>, TypeIndex),
}

#[derive(Clone, Debug)]
pub struct TypeName {
    pub path: Path,
    pub name: String,
    pub arguments: Vec<TypeIndex>,
}

impl TypeName {
    pub fn to_string(&self) -> String {
        let mut name = format!("{}::{}", self.path.to_string(), self.name);
        if self.arguments.len() > 0 {
            name.push_str("[");
            for typ in self.arguments.iter() {
                name.push_str(&format!("{}", typ.into_raw_parts().0));
            }
            name.push_str("]");
        }
        name
    }
}

impl From<(Path, String)> for TypeName {
    fn from(pair: (Path, String)) -> Self {
        Self {
            path: pair.0,
            name: pair.1,
            arguments: vec![],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Access {
    Public,
    Internal,
}

#[derive(Clone, Debug)]
pub struct AstFunction {
    pub access: Access,
    pub kind: FunctionKind,
    pub name: String,
    pub type_params: Vec<TypedName>,
    pub params: Vec<TypedName>,
    pub return_type: TypeIndex,
    pub statements: Vec<StatementIndex>,
}

#[derive(Clone, Copy, Debug)]
pub enum StructKind {
    Struct,
    Actor,
}

#[derive(Clone, Copy, Debug)]
pub enum FunctionKind {
    Function,
    Behaviour,
}

#[derive(Clone, Debug)]
pub enum Node {
    TypeAlias {
        access: Access,
        unique: bool,
        name: String,
        value: TypeIndex,
    },
    Variable {
        access: Access,
        name: TypedName,
        value: Option<ExpressionIndex>,
    },
    Function(AstFunction),
    FunctionPrototype {
        name: String,
        kind: FunctionKind,
        type_params: Vec<TypedName>,
        params: Vec<TypedName>,
        return_type: TypeIndex,
    },
    Struct {
        access: Access,
        kind: StructKind,
        name: String,
        params: Vec<TypedName>,
        children: Vec<NodeIndex>,
    },
    Enum {
        access: Access,
        name: String,
        params: Vec<TypedName>,
        variants: Vec<EnumVariant>,
    },
    Interface {
        access: Access,
        name: String,
        params: Vec<TypedName>,
        children: Vec<NodeIndex>,
    },
    Error,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub params: Vec<TypedName>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    If {
        condition: ExpressionIndex,
        body: Vec<StatementIndex>,
        else_if: Option<StatementIndex>,
    },
    Call {
        function: ExpressionIndex,
        args: Vec<ExpressionIndex>,
    },
    Let {
        name: TypedName,
        value: ExpressionIndex,
    },
    Assign {
        name: String,
        value: ExpressionIndex,
    },
    Return {
        value: ExpressionIndex,
    },
    Unsafe {
        body: Vec<StatementIndex>,
    },
}

#[derive(Clone, Debug)]
pub enum Expression {
    Ref(String),
    NatLiteral(i64),
    BoolLiteral(bool),
    BinOp(ExpressionIndex, BinOpType, ExpressionIndex),
    FieldAccessor {
        aggregate: ExpressionIndex,
        value: ExpressionIndex,
    },
    FunctionCall {
        function: ExpressionIndex,
        args: Vec<ExpressionIndex>,
    },
    New {
        typ: TypeIndex,
        allocator: ExpressionIndex,
    },
    Dereference {
        pointer: ExpressionIndex,
    },
    Denull {
        optional: ExpressionIndex,
    },
    Borrow {
        value: ExpressionIndex,
    },
    Unsafe {
        value: ExpressionIndex,
    },
}

impl Expression {
    pub fn to_string(&self, program_arena: &ProgramArena) -> String {
        use Expression::*;
        match self {
            BinOp(a, o, b) => {
                let a_opt = program_arena.expression_arena.get(*a);
                let b_opt = program_arena.expression_arena.get(*b);
                if let (Some(a_exp), Some(b_exp)) = (a_opt, b_opt) {
                    format!("({} {} {})", a_exp.to_string(program_arena), o, b_exp.to_string(program_arena))
                } else {
                    format!("{}", self)
                }
            }
            FieldAccessor { aggregate, value } => {
                let agg_opt = program_arena.expression_arena.get(*aggregate);
                let value_opt = program_arena.expression_arena.get(*value);
                if let (Some(agg_exp), Some(value_exp)) = (agg_opt, value_opt) {
                    format!("{}.{}", agg_exp.to_string(program_arena), value_exp.to_string(program_arena))
                } else {
                    format!("{}", self)
                }
            }
            New { typ, allocator } => {
                let typ_opt = program_arena.type_arena.get(*typ);
                let allocator_opt = program_arena.expression_arena.get(*allocator);
                if let (Some(allocator_exp), Some(typ)) = (allocator_opt, typ_opt) {
                    format!("new {:?} in {}", typ, allocator_exp.to_string(program_arena))
                } else {
                    format!("{}", self)
                }
            }
            e => format!("{}", e)
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Expression::*;
        match self {
            Ref(r) => {
                write!(f, "{}", r)
            }
            NatLiteral(n) => {
                write!(f, "{}", n)
            }
            BoolLiteral(b) => {
                write!(f, "{}", b)
            }
            BinOp(a, o, b) => {
                let (a_index, _) = a.into_raw_parts();
                let (b_index, _) = b.into_raw_parts();
                write!(f, "#{} {} #{}", a_index, o, b_index)
            }
            FieldAccessor { aggregate, value } => {
                let (agg_index, _) = aggregate.into_raw_parts();
                let (value_index, _) = value.into_raw_parts();
                write!(f, "#{}.{}", agg_index, value_index)
            }
            FunctionCall { function, args } => {
                let (function_index, _) = function.into_raw_parts();
                let arg_indices: Vec<usize> = args.iter().map(|arg| arg.into_raw_parts().0).collect();
                write!(f, "#{}({:?})", function_index, arg_indices)
            }
            New { typ, allocator } => {
                let (type_index, _) = typ.into_raw_parts();
                let (allocator_index, _) = allocator.into_raw_parts();
                write!(f, "new #{} in #{}", type_index, allocator_index)
            }
            Dereference { pointer } => {
                let (pointer_index, _) = pointer.into_raw_parts();
                write!(f, "{}.*", pointer_index)
            }
            Denull { optional } => {
                let (optional_index, _) = optional.into_raw_parts();
                write!(f, "{}.?", optional_index)
            }
            Borrow { value } => {
                let (value_index, _) = value.into_raw_parts();
                write!(f, "{}.&", value_index)
            }
            Unsafe { value } => {
                let (value_index, _) = value.into_raw_parts();
                write!(f, "unsafe {}", value_index)
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BinOpType {
    Plus,
    Minus,
    Star,
    ForwardSlash,
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
    And,
    Or,
}

impl fmt::Display for BinOpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use BinOpType::*;
        write!(f, "{}", match self {
            Plus => "+",
            Minus => "-",
            Star => "*",
            ForwardSlash => "/",
            LessThan => "<",
            GreaterThan => ">",
            LessThanEqualTo => "<=",
            GreaterThanEqualTo => ">=",
            And => "and",
            Or => "or",
        })
    }
}
