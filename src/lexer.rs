#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Class, Is, Done, Local, Print, Get, At, Assign, Dot, New,
    If, Then, While, Do, Greater, Less, Equal,
    Plus, Minus, Star, Slash, Comma, Rest,
    Quest, Percent, LeftBracket, RightBracket,
    Identifier(String), Number(f64), StringLit(String), EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self { Self { input: input.chars().collect(), pos: 0 } }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Token::EOF; }
        let ch = self.input[self.pos];
        match ch {
            '?' => { self.pos += 1; Token::Quest },
            '%' => { self.pos += 1; Token::Percent },
            '@' => { self.pos += 1; Token::At },
            ',' => { self.pos += 1; Token::Comma },
            '.' => { self.pos += 1; Token::Dot },
            '[' => { self.pos += 1; Token::LeftBracket },
            ']' => { self.pos += 1; Token::RightBracket },
            '>' => { self.pos += 1; Token::Greater },
            '<' => { self.pos += 1; Token::Less },
            '+' => { self.pos += 1; Token::Plus },
            '-' => { self.pos += 1; Token::Minus },
            '*' => { self.pos += 1; Token::Star },
            '/' => { self.pos += 1; Token::Slash },
            '=' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == '=' {
                    self.pos += 1; Token::Equal
                } else { Token::Assign }
            },
            '"' => self.lex_string(),
            '0'..='9' => self.lex_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(),
            _ => { self.pos += 1; self.next_token() }
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while self.pos < self.input.len() && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') {
            ident.push(self.input[self.pos]); self.pos += 1;
        }
        match ident.as_str() {
            "Get" => Token::Get, // Case sensitive as requested
            "class" => Token::Class, 
            "new" => Token::New,
            "local" => Token::Local, 
            "print" => Token::Print, 
            "rest" => Token::Rest,
            "if" => Token::If, 
            "then" => Token::Then, 
            "while" => Token::While,
            "do" => Token::Do, 
            "is" => Token::Is, 
            "done" => Token::Done,
            _ => Token::Identifier(ident),
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut n = String::new();
        while self.pos < self.input.len() && (self.input[self.pos].is_digit(10) || self.input[self.pos] == '.') {
            n.push(self.input[self.pos]); self.pos += 1;
        }
        Token::Number(n.parse().unwrap_or(0.0))
    }

    fn lex_string(&mut self) -> Token {
        self.pos += 1; let mut s = String::new();
        while self.pos < self.input.len() && self.input[self.pos] != '"' {
            s.push(self.input[self.pos]); self.pos += 1;
        }
        self.pos += 1; Token::StringLit(s)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; }
    }
}