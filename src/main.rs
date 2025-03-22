use html_parser::HTMLParser;

mod css_parser;
mod dom;
mod html_parser;
mod style;

fn main() {
    HTMLParser::parse("<html></html>".to_string());
}
