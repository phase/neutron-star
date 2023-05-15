use std::borrow::Cow;

pub struct Module<'a> {
    kind: ModuleKind,
    name: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy)]
enum ModuleKind {
    CPU,
    GPU,
    SPIRV
}

impl ToString for ModuleKind {
    fn to_string(&self) -> String {
        match self {
            ModuleKind::CPU => "cpu".to_string(),
            ModuleKind::GPU => "gpu".to_string(),
            ModuleKind::SPIRV => "spirv".to_string()
        }
    }
}

enum BinOp {
    Plus,
    Minus,
    Times,
    Divide,
    Modulo,
    Power,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LogicalAnd,
    LogicalOr,
}

enum BuiltinType {
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
}

enum Ref {
    Index(u32),
    Type(BuiltinType),
}

enum Instruction {
    BinOp {
        op: BinOp,
        operad: Ref,
    }
}

type Index = i32;

enum Node {
    Container {
        kind: ContainerKind,
        nodes: Vec<Index>,
    },
    GlobalVariable {
        name: String,
    },
    Function {
        name: String,
        parameters: Vec<Index>,
    },
    Block {
        items: Vec<Index>,
    },
    Instruction(Instruction),
}

enum ContainerKind {
    Struct,
    Actor,
}

enum Expression {
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    ArrayLiteral {
        items: Vec<Expression>
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>
    },
}
