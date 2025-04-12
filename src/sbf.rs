use crate::intr::{Instruction, WithPos};

pub struct Program<'a> {
    source: &'a str,
    pos: usize,
}

#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedToken { message: String, span: Span },
    UnexpectedEof { message: String, span: Span },
}

impl<'a> Program<'a> {
    pub fn from(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<WithPos<Instruction>>, Error> {
        let mut instructions = vec![];
        while self.peek_char().is_some() {
            self.skip_whitespace();
            if self.starts_with("super") {
                instructions.push(self.parse_super_function()?);
            } else {
                instructions.push(self.parse_instruction()?);
            }
        }

        Ok(instructions)
    }

    fn parse_super_function(&mut self) -> Result<WithPos<Instruction>, Error> {
        let start = self.pos;
        self.consume("super")?;
        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.consume("(")?;
        let args = self.parse_args_def()?;
        self.consume(")")?;
        println!("dsdsdsdsa");
        self.skip_whitespace();
        self.consume("{")?;
        let body = self.parse_until("}")?;
        self.consume("}")?;
        let end = self.pos;

        Ok(WithPos {
            start,
            end,
            value: Instruction::SuperFunction { name, args, body },
        })
    }

    fn parse_args_def(&mut self) -> Result<Vec<String>, Error> {
        let mut args = vec![];
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(')') {
                break;
            }
            let arg = self.parse_identifier()?;
            args.push(arg);
            self.skip_whitespace();

            if self.peek_char() == Some(',') {
                self.pos += 1;
            } else {
                break;
            }
        }

        Ok(args)
    }

    fn parse_until(&mut self, end: &str) -> Result<Vec<WithPos<Instruction>>, Error> {
        let mut result = vec![];
        while !self.starts_with(end) {
            result.push(self.parse_instruction()?);
        }
        println!("here");
        Ok(result)
    }

    fn parse_instruction(&mut self) -> Result<WithPos<Instruction>, Error> {
        self.skip_whitespace();
        let start = self.pos;
        let c = self
            .peek_char()
            .ok_or_else(|| self.err_eof("Expected instruction"))?;

        let ret = match c {
            '+' | '-' => {
                self.pos += 1;
                Ok(Instruction::Add(if c == '+' { 1 } else { -1 }))
            }
            '>' | '<' => {
                self.pos += 1;
                Ok(Instruction::Move(if c == '>' { 1 } else { -1 }))
            }
            '[' => {
                self.pos += 1;
                Ok(Instruction::LoopStart)
            }
            ']' => {
                self.pos += 1;
                Ok(Instruction::LoopEnd)
            }
            '.' => {
                self.pos += 1;
                Ok(Instruction::PutC)
            }
            ',' => {
                self.pos += 1;
                Ok(Instruction::GetC)
            }
            c if c.is_alphabetic() => self.parse_super_call(),
            _ => Err(self.err("Unknown instruction")),
        };
        let end = self.pos;

        ret.map(|instr| WithPos {
            start,
            end,
            value: instr,
        })
    }

    fn parse_super_call(&mut self) -> Result<Instruction, Error> {
        println!("call");
        let name = self.parse_identifier()?;
        self.consume("(")?;
        let args = self.parse_super_call_args()?;
        self.consume(")")?;
        Ok(Instruction::SuperCall { name, args })
    }

    fn parse_super_call_args(&mut self) -> Result<Vec<WithPos<Instruction>>, Error> {
        let mut args = vec![];
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(')') {
                break;
            }
            println!("arg");

            args.push(self.parse_instruction()?);
            println!("arg??");
            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.pos += 1;
            } else {
                break;
            }
        }

        Ok(args)
    }

    fn parse_identifier(&mut self) -> Result<String, Error> {
        self.skip_whitespace();
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if start == self.pos {
            Err(self.err("Expected identifier"))
        } else {
            Ok(self.source[start..self.pos].to_string())
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }

    fn consume(&mut self, expected: &str) -> Result<(), Error> {
        if self.starts_with(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(self.err(&format!("Expected '{expected}'")))
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn err(&self, message: &str) -> Error {
        Error::UnexpectedToken {
            message: message.to_string(),
            span: Span {
                start: self.pos,
                end: self.pos + 1,
            },
        }
    }

    fn err_eof(&self, message: &str) -> Error {
        Error::UnexpectedEof {
            message: message.to_string(),
            span: Span {
                start: self.pos,
                end: self.pos,
            },
        }
    }
}
