use std::{ops::Deref, collections::HashMap};

use super::Module;
use crate::{lang::*, ir::*};

struct PrintManager {
    buffer: String,
    current_indent: u32,
}

impl PrintManager {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            current_indent: 0,
        }
    }

    fn indent(&mut self) {
        self.current_indent += 1;
    }

    fn dedent(&mut self) {
        self.current_indent -= 1;
    }

    fn whitespace(&self) -> String {
        let mut s = String::new();
        for _ in 0..self.current_indent {
            s.push_str("  ");
        }
        s
    }

    fn write<T:AsRef<str>>(&mut self, s: T) {
        let whitespace = self.whitespace();
        self.buffer.push_str(&format!("{}{}", whitespace, s.as_ref()));
    }
}

pub struct IrPrintManager {
    printer: PrintManager,
}

impl IrPrintManager {
    pub fn new() -> Self {
        Self {
            printer: PrintManager::new(),
        }
    }

    pub fn print(&mut self, module: &Module) {
        self.printer.write(&format!("module {}{}{}\n\n", module.path.to_string(), SEPARATOR, module.name));

        module.imports.iter().for_each(|path| {
            self.printer.write(&format!("import {}\n", path.to_string()));
        });

        for (_, node) in module.module_arena.node_arena.iter() {
            self.print_node(&module.module_arena, node);
        }
    }

    fn print_node(&mut self, arena: &ModuleArena, node: &IrNode) {
        match node {
            IrNode::Function (func) => {
                // function signature
                self.printer.write(&format!("function {}(", func.name));
                for (i, arg) in func.params.iter().enumerate() {
                    if i > 0 {
                        self.printer.write(", ");
                    }
                    self.print_typed_name(arena, arg);
                }
                self.printer.write(") -> ");
                // return type
                let return_type = arena.type_arena.get(func.return_type).map(|typ| {
                    self.print_type(arena, typ)
                }).unwrap_or("unknown_type".to_string());
                self.printer.write(return_type);
                self.printer.write(":\n");

                // function body
                self.printer.indent();
                let mut instruction_names: HashMap<Index, String> = HashMap::new();
                for (i, block_index) in func.blocks.iter().enumerate() {
                    let block = arena.block_arena.get(*block_index).expect(format!("where did block {:?} go??", block_index).as_str());
                    self.printer.write(format!("block#{}:\n", i));
                    self.printer.indent();
                    for (j, instruction_index) in block.instructions.iter().enumerate() {
                        let instruction = arena.instruction_arena.get(*instruction_index).expect(format!("where did instruction {:?} go??", instruction_index).as_str());
                        self.printer.write(format!("%{} = {}\n", j, self.print_instruction(&mut instruction_names, arena, instruction)));
                    }
                    self.printer.dedent();
                }
                self.printer.dedent();
                self.printer.write("\n");
            }
            n => {
                self.printer.write(format!("unknown node: {:?}", n));
            }
        }
    }

    fn print_typed_name(&mut self, arena: &ModuleArena, typed_name: &IrTypedName) {
        let type_name = arena.type_arena.get(typed_name.typ).map(|typ| {
            self.print_type(arena, typ)
        }).unwrap_or("unknown_type".to_string());
        self.printer.write(&format!("{}: {}", typed_name.name, type_name));
    }

    fn print_type(&self, arena: &ModuleArena, typ: &IrType) -> String {
        use IrType::*;
        match typ {
            Bool => "Bool".to_string(),
            UInt(uint) => uint.to_string(),
            Int(int) => int.to_string(),
            Float(float) => float.to_string(),
            Void => "Void".to_string(),
            Unknown => "Unknown".to_string(),
            Reference(inner, ptr_kind, refcap) => {
                let inner_type = arena.type_arena.get(*inner).map(|typ| {
                    self.print_type(arena, typ)
                }).unwrap_or("unknown_type".to_string());

                let ptr_kind = match ptr_kind {
                    PointerKind::Raw => "*",
                    PointerKind::Tracked => "&",
                };

                format!("{}{} {}", ptr_kind, refcap.to_string(), inner_type)
            },
            x => format!("bad_type[{:?}]", x),
        }
    }

    fn print_instruction(&self, instruction_map: &mut HashMap<Index, String>, arena: &ModuleArena, ins: &IrInstruction) -> String {
        let mut to_string = |i: &Index| {
            if let Some(x) = instruction_map.get(i) {
                return x.clone();
            }
            let name = format!("%{}", instruction_map.len());
            instruction_map.insert(*i, name.clone());
            return name;
        };

        use IrInstruction::*;
        match ins {
            BoolLiteral(b) => format!("{}", b),
            NatLiteral(n) => format!("{}", n),
            Branch { condition, true_branch, false_branch } => format!("branch {} {} {}", to_string(condition), to_string(true_branch), to_string(false_branch)),
            Return { value } => format!("return {}", to_string(value)),
            BinOp(a, op, b) => format!("binop.`{}` {} {}", op, to_string(a), to_string(b)),
            Ref(a) => format!("ref %{}", a),
            FunctionCall { function, args } => format!("call {} ({})", to_string(function), args.iter().map(|i| to_string(i)).collect::<Vec<String>>().join(", ")),
            New { typ, allocator } => format!("new {} {}", to_string(typ), to_string(allocator)),
            Dereference { pointer } => format!("deref.`&` {}", to_string(pointer)),
            Denull { optional } => format!("denull.`!!` {}", to_string(optional)),
            x => format!("bad_ins[{:?}]", x),
        }
    }
}

impl ToString for IrPrintManager {
    fn to_string(&self) -> String {
        self.printer.buffer.clone()
    }
}
