use parser::Parser;

mod dom;
mod parser;

fn main() {
    Parser::parse("<html></html>".to_string());
}
