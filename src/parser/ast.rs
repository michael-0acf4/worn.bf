#[derive(Debug, Clone)]
pub enum SuperValue {
    Integer(u32),
    String(String),
    Literal(String),
    SuperCall {
        callee: WithPos<String>,
        args: Vec<WithPos<Instruction>>,
    },
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Add(i32),
    Move(i32),
    PutC,
    GetC,
    Loop {
        body: Vec<WithPos<Instruction>>,
    },
    // ext
    InlineValue(SuperValue),
    SuperFunction {
        name: WithPos<String>,
        args: Vec<WithPos<String>>,
        body: Vec<WithPos<Instruction>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithPos<T> {
    pub start: usize,
    pub end: usize,
    pub value: T,
}

impl<T> WithPos<T> {
    pub fn transfer<P>(&self, p: P) -> WithPos<P> {
        WithPos {
            value: p,
            start: self.start,
            end: self.end,
        }
    }
}

pub trait Reconstruct {
    fn reconstruct_at_depth(&self, depth: usize) -> String;

    fn reconstruct(&self) -> String {
        self.reconstruct_at_depth(0)
    }
}

impl<T: Reconstruct> Reconstruct for WithPos<T> {
    fn reconstruct_at_depth(&self, depth: usize) -> String {
        self.value.reconstruct_at_depth(depth)
    }
}

impl Reconstruct for SuperValue {
    fn reconstruct_at_depth(&self, depth: usize) -> String {
        let ret = match self {
            SuperValue::Integer(n) => format!("{}", n),
            SuperValue::String(s) => format!("{:?}", s),
            SuperValue::Literal(s) => format!("{s}"),
            SuperValue::SuperCall { callee, args } => {
                let arg_strs = args.iter().map(|arg| arg.reconstruct()).collect::<Vec<_>>();
                format!("{}({})", callee.value, arg_strs.join(", "))
            }
        };

        format!("{}{}", " ".repeat(depth), ret)
    }
}

impl Reconstruct for Instruction {
    fn reconstruct_at_depth(&self, depth: usize) -> String {
        match self {
            Instruction::Add(n) => {
                format!(
                    "{}{}",
                    " ".repeat(depth),
                    if *n > 0 {
                        "+".repeat(*n as usize)
                    } else {
                        "-".repeat((-n) as usize)
                    }
                )
            }
            Instruction::Move(n) => {
                format!(
                    "{}{}",
                    " ".repeat(depth),
                    if *n > 0 {
                        ">".repeat(*n as usize)
                    } else {
                        "<".repeat((-n) as usize)
                    }
                )
            }
            Instruction::Loop { body } => {
                let indent = " ".repeat(depth);
                format!(
                    "{}[\n{}\n{}]",
                    indent,
                    body.reconstruct_at_depth(depth + 1),
                    indent
                )
            }
            Instruction::PutC => format!("{}{}", " ".repeat(depth), '.'),
            Instruction::GetC => format!("{}{}", " ".repeat(depth), ','),
            Instruction::SuperFunction { name, args, body } => {
                let header = format!(
                    "super {}({})",
                    name.value,
                    args.iter()
                        .map(|a| a.value.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let body_str = body.reconstruct_at_depth(depth + 1);
                let indent = " ".repeat(depth);
                format!("{indent}{header} {{\n{body_str}\n{indent}}}\n")
            }
            Instruction::InlineValue(s) => s.reconstruct_at_depth(depth),
        }
    }
}

impl Reconstruct for Vec<WithPos<Instruction>> {
    fn reconstruct_at_depth(&self, depth: usize) -> String {
        self.iter()
            .map(|i| i.reconstruct_at_depth(depth))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
