use crate::parser::{
    ast::{Instruction, Reconstruct, SuperValue, WithPos},
    parse_program,
};

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum CompileError {
    Invalid {
        message: String,
        start: usize,
        end: usize,
    },
    UndeclaredFunction {
        name: String,
        start: usize,
        end: usize,
    },
    UndeclaredSymbol {
        name: String,
        start: usize,
        end: usize,
    },
}

pub trait Named {
    fn get_name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct ScopedStack<S> {
    symbols: Vec<S>,
    scope: Vec<Vec<String>>,
}

impl<S: Named + Clone> ScopedStack<S> {
    pub fn new() -> Self {
        Self {
            symbols: vec![],
            scope: vec![],
        }
    }

    pub fn new_scope(&mut self) {
        self.scope.push(Vec::new());
    }

    pub fn end_scope(&mut self) {
        if let Some(scope) = self.scope.pop() {
            for var in scope {
                let last_match = self.symbols.iter().rposition(|t| t.get_name().eq(&var));
                if let Some(index) = last_match {
                    let _ = self.symbols.remove(index);
                } else {
                    panic!("Invalid check state");
                }
            }
        } else {
            panic!("Invalid scope state, new_scope not called?");
        }
    }

    /// Find the nearest visible symbol from the end to the start
    pub fn find_rvisiblle(&self, name: &str) -> Option<S> {
        match self.symbols.iter().rposition(|v| v.get_name().eq(name)) {
            Some(idx) => Some(self.symbols[idx].clone()),
            None => None,
        }
    }

    pub fn push(&mut self, value: S) {
        self.symbols.push(value.clone());
        self.scope
            .last_mut()
            .map(|last| last.push(value.get_name()));
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub of: WithPos<Instruction>,
}

#[derive(Debug, Clone)]
pub struct VariableSet {
    pub name: WithPos<String>,
    pub value: WithPos<Instruction>,
}

impl Named for VariableSet {
    fn get_name(&self) -> String {
        self.name.value.clone()
    }
}

impl Named for SymbolInfo {
    fn get_name(&self) -> String {
        match &self.of.value {
            Instruction::InlineValue(value) => match value {
                SuperValue::Literal(s) => return s.to_string(),
                _ => {}
            },
            Instruction::SuperFunction { name, .. } => return name.value.clone(),
            _ => {}
        }

        panic!(
            "Underlying symbol ({:?}) is neither a literal token nor a function declaration",
            self.of.reconstruct()
        )
    }
}

#[derive(Debug)]
pub struct Context {
    func_scope: ScopedStack<SymbolInfo>,
    variable_scope: ScopedStack<VariableSet>,
    output: Vec<String>,
}

impl Context {
    /// Create a new context, based on the current one
    ///
    /// To use when a check requires a self-contained mutable context
    pub fn create() -> Self {
        Self {
            func_scope: ScopedStack::new(),
            variable_scope: ScopedStack::new(),
            output: vec![],
        }
    }

    pub fn push_func(&mut self, func: WithPos<Instruction>) {
        self.func_scope.push(SymbolInfo { of: func });
    }

    pub fn push_variable(&mut self, name: WithPos<String>, value: WithPos<Instruction>) {
        self.variable_scope.push(VariableSet { name, value });
    }

    pub fn new_scope(&mut self) {
        self.func_scope.new_scope();
        self.variable_scope.new_scope();
    }

    pub fn end_scope(&mut self) {
        self.func_scope.end_scope();
        self.variable_scope.end_scope();
    }

    pub fn resolve_variable_rec(&mut self, name: &str) -> Option<WithPos<Instruction>> {
        let mut found = None;
        let mut key = name.to_owned();
        loop {
            if let Some(var) = self.variable_scope.find_rvisiblle(&key) {
                found = Some(var.value.clone());
                if let Some(name) = &var.value.value.as_literal() {
                    // println!("{key} => {name}");
                    key = name.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        found
    }
}

pub struct SBFEmitter {
    context: Context,
    program: Vec<WithPos<Instruction>>,
}

impl SBFEmitter {
    pub fn new(s: &str) -> Result<Self, String> {
        Ok(Self {
            context: Context::create(),
            program: parse_program(s)?,
        })
    }

    pub fn finalize(&self) -> Result<String, String> {
        Ok(self.context.output.join(""))
    }

    pub fn emit_inline(&mut self, s: String) -> Result<(), CompileError> {
        self.context.output.push(s);
        Ok(())
    }

    pub fn emit_native_repeat(
        &mut self,
        to_repeat: &WithPos<Instruction>,
    ) -> Result<bool, CompileError> {
        let count_val = self
            .context
            .resolve_variable_rec("__count")
            .map(|v| v.value.as_integer())
            .flatten();

        return if let Some(count) = count_val {
            for _ in 0..count {
                self.emit_instr(to_repeat)?;
            }

            Ok(true)
        } else {
            Err(CompileError::Invalid {
                message: format!(
                    "First argument of repeat function R is expected to be an integer, got {} instead",
                    to_repeat.value.reconstruct()
                ),
                start: to_repeat.start,
                end: to_repeat.end,
            })
        };
    }

    pub fn emit_super_value(
        &mut self,
        super_value: &WithPos<SuperValue>,
    ) -> Result<(), CompileError> {
        match &super_value.value {
            SuperValue::Integer(n) => self.emit_inline("+".repeat(*n as usize)),
            SuperValue::String(s) => self.emit_inline(
                s.chars()
                    .map(|s| Instruction::Add(s as i32 & 0xff).reconstruct())
                    .collect::<Vec<_>>()
                    .join(">"),
            ),
            SuperValue::Literal(lit) => {
                if let Some(var) = self.context.resolve_variable_rec(&lit) {
                    self.emit_instr(&var)?;
                    return Ok(());
                }

                Err(CompileError::UndeclaredSymbol {
                    name: lit.to_owned(),
                    start: super_value.start,
                    end: super_value.end,
                })
            }
            SuperValue::SuperCall {
                callee,
                args: callee_args,
            } => {
                if callee.value == "R" && callee_args.len() == 2 {
                    self.context.new_scope();
                    self.context.push_variable(
                        callee.transfer("__count".to_owned()),
                        callee_args[0].clone(),
                    );
                    self.emit_native_repeat(&callee_args[1])?;
                    self.context.end_scope();

                    return Ok(());
                } else if let Some(s) = self.context.func_scope.find_rvisiblle(&callee.value) {
                    if let Instruction::SuperFunction { args, body, .. } = s.of.value {
                        self.context.new_scope();
                        for (name, value) in args.iter().zip(callee_args.iter()) {
                            self.context.push_variable(name.clone(), value.clone());
                        }
                        self.emit_body(&body)?;
                        self.context.end_scope();

                        return Ok(());
                    }
                }

                Err(CompileError::UndeclaredFunction {
                    name: callee.value.clone(),
                    start: callee.start,
                    end: callee.end,
                })
            }
        }
    }

    pub fn emit_loop(&mut self, body: &[WithPos<Instruction>]) -> Result<(), CompileError> {
        self.emit_inline("[".to_string())?;
        self.emit_body(&body)?;
        self.emit_inline("]".to_string())
    }

    pub fn emit_instr(&mut self, instr: &WithPos<Instruction>) -> Result<(), CompileError> {
        match &instr.value {
            Instruction::Add(_) | Instruction::Move(_) | Instruction::PutC | Instruction::GetC => {
                self.emit_inline(instr.value.reconstruct())?
            }
            Instruction::Loop { body } => self.emit_loop(body)?,
            Instruction::InlineValue(super_value) => {
                self.emit_super_value(&instr.transfer(super_value.clone()))?
            }
            Instruction::SuperFunction { .. } => {
                self.context.push_func(instr.clone());
            }
        }

        Ok(())
    }

    pub fn emit_body(&mut self, body: &[WithPos<Instruction>]) -> Result<(), CompileError> {
        for instr in &body.to_owned() {
            self.emit_instr(instr)?;
        }

        Ok(())
    }

    pub fn compile(&mut self) -> Result<(), CompileError> {
        self.emit_body(&self.program.clone())
    }
}
