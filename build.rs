extern crate lalrpop;

fn main() {
    // grammar is in src/parser/grammar.lalrpop
    lalrpop::process_root().unwrap();
}
