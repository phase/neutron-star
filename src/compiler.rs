use generational_arena::Arena;
use crate::ast::Path;
use crate::ir::Module;
use crate::ir::translate::IrBuilder;
use crate::parser::Parser;

pub struct Compiler {
    pub modules: Arena<Module>,
    ir_builder: IrBuilder,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            modules: Default::default(),
            ir_builder: IrBuilder::new(),
        }
    }

    pub fn parse_module(&mut self, path: Path, file_name: String, code: String) {
        let mut parser = Parser::new();
        let parsed_program = parser.parse(path, file_name, code);
        parser.diagnostics.emit_errors();
        if let Some(program) = parsed_program {
            let module = self.ir_builder.convert(program);
            self.modules.insert(module);
        }
    }
}
