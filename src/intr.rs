#[derive(Debug, Clone)]
pub enum Instruction {
    Add(i32),
    Move(i32),
    PutC,
    GetC,
    LoopStart,
    LoopEnd,
    SuperCall {
        name: String,
        args: Vec<WithPos<Instruction>>,
    },
    SuperFunction {
        name: String,
        args: Vec<String>,
        body: Vec<WithPos<Instruction>>,
    },
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct WithPos<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
}

impl WithPos<Instruction> {
    pub fn reconstruct(&self) -> String {
        self.value.reconstruct()
    }
}

impl Instruction {
    pub fn reconstruct(&self) -> String {
        match self {
            Instruction::Add(n) => {
                if *n > 0 {
                    "+".repeat(*n as usize)
                } else {
                    "-".repeat((-n) as usize)
                }
            }
            Instruction::Move(n) => {
                if *n > 0 {
                    ">".repeat(*n as usize)
                } else {
                    "<".repeat((-n) as usize)
                }
            }
            Instruction::LoopStart => "[".to_string(),
            Instruction::LoopEnd => "]".to_string(),
            Instruction::PutC => ".".to_string(),
            Instruction::GetC => ",".to_string(),
            Instruction::SuperCall { name, args } => {
                let arg_strs = args.iter().map(|arg| arg.reconstruct()).collect::<Vec<_>>();
                format!("{}({})", name, arg_strs.join(", "))
            }
            Instruction::SuperFunction { name, args, body } => {
                let header = format!("super {}({})", name, args.join(", "));
                let body_str = body
                    .iter()
                    .map(|instr| instr.reconstruct())
                    .collect::<String>();
                format!("{header} {{\n    {body_str}\n}}")
            }
        }
    }
}
