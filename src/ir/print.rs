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

struct IrPrintManager {
    printer: PrintManager,
}

impl IrPrintManager {
    fn new() -> Self {
        Self {
            printer: PrintManager::new(),
        }
    }

    fn print(&mut self, module: &Module) {
        self.printer.write(&format!("module {}{}{}", module.path.to_string(), SEPARATOR, module.name));

        module.imports.iter().for_each(|path| {
            self.printer.write(&format!("import {}", path.to_string()));
        });

        for (_, node) in module.module_arena.node_arena.iter() {
            self.print_node(node);
        }
    }

    fn print_node(&mut self, node: &IrNode) {
        match node {
            IrNode::Function (func) => {
                self.printer.write(&format!("function {}(", func.name));
                for (i, arg) in func.params.iter().enumerate() {
                    if i > 0 {
                        self.printer.write(", ");
                    }
                    self.printer.write(&format!("{}: {}", arg.name, arg.typ));
                }
            }
            n => {
                self.printer.write(format!("unknown node: {:?}", n));
            }
        }
    }
}
