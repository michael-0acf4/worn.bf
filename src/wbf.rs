use crate::parser::ast::{BInstr, Instruction, Reconstruct, SuperValue, WithPos};

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

impl ToString for CompileError {
    fn to_string(&self) -> String {
        match self {
            CompileError::Invalid {
                message,
                start,
                end,
            } => {
                format!("{message} at {start} .. {end}")
            }
            CompileError::UndeclaredFunction { name, start, end }
            | CompileError::UndeclaredSymbol { name, start, end } => {
                let prefix = if matches!(self, CompileError::UndeclaredSymbol { .. }) {
                    "symbol"
                } else {
                    "function"
                };

                format!("Undeclared {prefix} {name:?} at {start} .. {end}")
            }
        }
    }
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

#[derive(Debug, Clone)]
pub struct GenericSymbol {
    pub name: String,
}

impl Named for GenericSymbol {
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug)]
pub struct Context {
    func_scope: ScopedStack<SymbolInfo>,
    variable_scope: ScopedStack<VariableSet>,
    fncall_stack: ScopedStack<GenericSymbol>,
    output: Vec<BInstr>,
}

impl Context {
    /// Create a new context, based on the current one
    ///
    /// To use when a check requires a self-contained mutable context
    pub fn create() -> Self {
        Self {
            func_scope: ScopedStack::new(),
            variable_scope: ScopedStack::new(),
            fncall_stack: ScopedStack::new(),
            output: vec![],
        }
    }

    pub fn push_func(&mut self, func: WithPos<Instruction>) {
        self.func_scope.push(SymbolInfo { of: func });
    }

    pub fn push_variable(&mut self, name: WithPos<String>, value: WithPos<Instruction>) {
        self.variable_scope.push(VariableSet { name, value });
    }

    pub fn push_fncall(&mut self, callee: String) {
        self.fncall_stack.push(GenericSymbol { name: callee });
    }

    pub fn new_scope(&mut self) {
        self.func_scope.new_scope();
        self.variable_scope.new_scope();
        self.fncall_stack.new_scope();
    }

    pub fn end_scope(&mut self) {
        self.func_scope.end_scope();
        self.variable_scope.end_scope();
        self.fncall_stack.end_scope();
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

pub struct WBFEmitter {
    context: Context,
    pub program: Vec<WithPos<Instruction>>,
}

impl WBFEmitter {
    pub fn new(program: Vec<WithPos<Instruction>>) -> Self {
        Self {
            context: Context::create(),
            program,
        }
    }

    pub fn finalize(self) -> Result<Vec<BInstr>, String> {
        Ok(self.context.output)
    }

    pub fn emit_inline_seq(&mut self, ss: Vec<BInstr>) -> Result<(), CompileError> {
        for s in ss {
            self.emit_inline(s)?;
        }

        Ok(())
    }

    pub fn emit_inline(&mut self, s: BInstr) -> Result<(), CompileError> {
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
            SuperValue::Integer(n) => self.emit_inline(BInstr::Add(*n as i32)),
            SuperValue::String(s) => self.emit_inline_seq({
                let chunks = s
                    .chars()
                    .map(|s| BInstr::Add(s as i32 & 0xff))
                    .collect::<Vec<_>>();
                let total = chunks.len();
                let mut output = vec![];
                for (i, instr) in chunks.into_iter().enumerate() {
                    output.push(instr);
                    if i + 1 != total {
                        output.push(BInstr::Move(1))
                    }
                }

                output
            }),
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
                if self
                    .context
                    .fncall_stack
                    .find_rvisiblle(&callee.value)
                    .is_some()
                {
                    return Err(CompileError::Invalid {
                        message: format!("{:?} cannot be recursive", callee.value),
                        start: callee.start,
                        end: callee.end,
                    });
                }

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

                        self.context.new_scope();
                        self.context.push_fncall(callee.value.to_owned());
                        self.emit_body(&body)?;
                        self.context.end_scope();

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
        self.emit_inline(BInstr::LoopStart)?;
        self.emit_body(&body)?;
        self.emit_inline(BInstr::LoopEnd)
    }

    pub fn emit_instr(&mut self, instr: &WithPos<Instruction>) -> Result<(), CompileError> {
        match &instr.value {
            Instruction::Add(_) | Instruction::Move(_) | Instruction::PutC | Instruction::GetC => {
                self.emit_inline(instr.value.clone().into())?
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
