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
                self.printer.write("\n");

                // function body
                self.printer.indent();
                for _block in func.blocks.iter() {
                    //self.print_block(arena, block);
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
        match typ {
            IrType::Bool => "Bool".to_string(),
            IrType::UInt(uint) => uint.to_string(),
            IrType::Int(int) => int.to_string(),
            IrType::Float(float) => float.to_string(),
            IrType::Reference(inner, ptr_kind, refcap) => {
                let inner_type = arena.type_arena.get(*inner).map(|typ| {
                    self.print_type(arena, typ)
                }).unwrap_or("unknown_type".to_string());

                let ptr_kind = match ptr_kind {
                    PointerKind::Raw => "*",
                    PointerKind::Tracked => "&",
                };

                format!("{}{} {}", ptr_kind, refcap.to_string(), inner_type)
            },
            x => format!("unknown_type[{:?}]", x),
        }
    }
}

impl ToString for IrPrintManager {
    fn to_string(&self) -> String {
        self.printer.buffer.clone()
    }
}
