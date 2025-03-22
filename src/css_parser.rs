#[derive(Debug)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    // Calculate the specificity of a selector.
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

impl Value {
    // Convert a length value to pixels.
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Unit {
    Px,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug)]
pub struct CSSParser {
    pos: usize,
    input: String,
}

impl CSSParser {
    // Parse rules.
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.is_eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    // Parse a rule.
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    // Parse selectors.
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character: {}", c),
            }
        }
        selectors.sort_by_key(|s| s.specificity());
        selectors
    }

    // Parse a simple selector.
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while !self.is_eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    // Parse declarations.
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        self.expect_char('{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    // Parse a declaration.
    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        self.expect_char(':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        self.expect_char(';');

        Declaration { name, value }
    }

    // Parse a value.
    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    // Parse a length value.
    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    // Parse a float value.
    fn parse_float(&mut self) -> f32 {
        self.consume_chars_while(|c| matches!(c, '0'..='9' | '.'))
            .parse()
            .unwrap()
    }

    // Parse a unit value.
    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("Unrecognized unit."),
        }
    }

    // Parse a color value.
    fn parse_color(&mut self) -> Value {
        self.expect_char('#');
        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    // Parse two hexadecimal digits.
    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    // Parse an identifier.
    fn parse_identifier(&mut self) -> String {
        self.consume_chars_while(valid_identifier_char)
    }

    // Consume characters while the condition is true.
    fn consume_chars_while(
        &mut self,
        condition: impl Fn(char) -> bool,
    ) -> String {
        let mut result = String::new();
        while !self.is_eof() && condition(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    // Consume whitespace.
    fn consume_whitespace(&mut self) {
        self.consume_chars_while(char::is_whitespace);
    }

    // Consume a character.
    #[inline]
    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    // Read the next character from the input.
    #[inline]
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // If the exact character is found, consume it.
    // Otherwise, panic.
    fn expect_char(&mut self, c: char) {
        if self.consume_char() != c {
            panic!(
                "Expected {:?} at byte {} but it was not found",
                c, self.pos
            );
        }
    }

    // Check if the parser is at the end of the input.
    #[inline]
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

// Check if a character is a valid identifier character.
fn valid_identifier_char(c: char) -> bool {
    // TODO: Include U+00A0 and higher.
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}
