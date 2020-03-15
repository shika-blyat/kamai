use std::ops::Range;

#[derive(Debug, Clone)]
pub enum TokenElem {
    Space,
    Int(isize),
    Identifier(String),
    Equal,
    Op(String),
}
#[derive(Debug, Clone)]
pub struct Token {
    pub elem: TokenElem,
    pub range: Range<usize>,
    pub lexeme: String,
}
impl Token {
    pub fn new(elem: TokenElem, range: Range<usize>, lexeme: String) -> Self {
        Self {
            elem,
            range,
            lexeme,
        }
    }
}

pub struct Lexer {
    code: Vec<char>,
    current: usize,
}
impl Lexer {
    pub fn new(code: String) -> Self {
        Self {
            code: code.chars().collect(),
            current: 0,
        }
    }
    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        loop {
            match self.current() {
                Some(c) => {
                    if c.is_ascii_digit() {
                        tokens.push(self.take_num());
                        continue;
                    } else if c == '+' || c == '-' || c == '/' || c == '*' {
                        tokens.push(Token::new(
                            TokenElem::Op(c.to_string()),
                            self.current..self.current + 1,
                            c.to_string(),
                        ));
                        self.current += 1;
                        continue;
                    } else if c.is_ascii_alphabetic() {
                        tokens.push(self.take_identifier());
                        continue;
                    }
                    match c {
                        '\n' => {
                            while let Some(c) = self.current() {
                                if c == ' ' {
                                    tokens.push(Token::new(
                                        TokenElem::Space,
                                        self.current..self.current + 1,
                                        " ".to_string(),
                                    ));
                                    self.current += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        '=' => tokens.push(Token::new(
                            TokenElem::Equal,
                            self.current..self.current + 1,
                            "=".to_string(),
                        )),
                        ' ' => (),
                        c => return Err(format!("Unexpected char `{}`", c)),
                    }
                    self.current += 1;
                }
                None => return Ok(tokens),
            }
        }
    }
    pub fn take_num(&mut self) -> Token {
        let mut num = self.current().unwrap().to_string();
        self.current += 1;
        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                num.push(c)
            } else {
                break;
            }
            self.current += 1;
        }
        let num_len = num.len();
        let num_c = num.clone();
        /*let num = match num.parse(){
            Ok(num) => num,
            Err(_) =>
        };*/
        return Token::new(
            TokenElem::Int(num.parse().unwrap()),
            self.current - num_len..self.current,
            num_c,
        );
    }
    pub fn take_identifier(&mut self) -> Token {
        let mut num = self.current().unwrap().to_string();
        self.current += 1;
        while let Some(c) = self.current() {
            if c.is_ascii_alphanumeric() {
                num.push(c)
            } else {
                break;
            }
            self.current += 1;
        }
        let num_len = num.len();
        let num_c = num.clone();
        return Token::new(
            TokenElem::Identifier(num),
            self.current - num_len..self.current,
            num_c,
        );
    }
    pub fn next(&mut self) -> Option<char> {
        self.current += 1;
        self.current()
    }
    pub fn current(&self) -> Option<char> {
        if self.is_empty() {
            None
        } else {
            Some(self.code[self.current])
        }
    }
    pub fn is_empty(&self) -> bool {
        self.code.len() <= self.current
    }
}
