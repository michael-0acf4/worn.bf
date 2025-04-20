use crate::{
    cli::AdvOptions,
    parser::ast::{BInstr, Reconstruct},
};

pub struct Optimizer {
    pub level: u8,
    pub adv_opt: Vec<AdvOptions>,
}

type Program = Vec<BInstr>;

impl Optimizer {
    pub fn apply(&self, mut program: Program) -> Program {
        if self.level == 0 {
            return program;
        }

        program = self.pass1_fold(program);
        program = self.pass2_smort_fold(program);
        if self.level >= 4 {
            program = self.pass2_smort_fold(program);
            program = self.pass1_fold(program);
        }

        program
    }

    fn pass1_fold(&self, program: Program) -> Program {
        let mut out = vec![];
        let mut iter = program.into_iter();
        macro_rules! aggregate_instr {
            ($variant:ident, $n:ident, $iter:ident, $out:ident) => {{
                let mut agg = *$n;
                while let Some(next) = $iter.clone().next() {
                    if let BInstr::$variant(m) = &next {
                        agg += *m;
                        $iter.next();
                    } else {
                        break;
                    }
                }

                if agg != 0 {
                    $out.push(BInstr::$variant(agg));
                }
            }};
        }

        while let Some(instr) = iter.next() {
            match &instr {
                BInstr::Add(n) => aggregate_instr!(Add, n, iter, out),
                BInstr::Move(n) => aggregate_instr!(Move, n, iter, out),
                BInstr::PutC(n) => aggregate_instr!(PutC, n, iter, out),
                BInstr::GetC(n) => aggregate_instr!(GetC, n, iter, out),
                _ => out.push(instr),
            }
        }

        out
    }

    fn pass2_smort_fold(&self, program: Program) -> Program {
        if self.level < 2 {
            return program;
        }

        let mut out = vec![];
        let mut iter = program.into_iter();
        while let Some(instr) = iter.next() {
            match &instr {
                BInstr::Add(n) => {
                    if *n == 0 {
                        continue;
                    }

                    let compr = if self.level == 2 {
                        compress_incr(*n, Some(5.0), true)
                    } else {
                        // > 3
                        compress_incr(*n, None, true)
                    };

                    let recons = compr.reconstruct();
                    if recons.len() < (*n).abs() as usize {
                        out.extend(compr);
                    } else {
                        // no op
                        out.push(instr);
                    }
                }
                BInstr::PutC(n) | BInstr::GetC(n) => {
                    let fold_io = self.adv_opt.contains(&AdvOptions::UnsafeFoldIO);
                    if *n == 0 || !fold_io {
                        if !fold_io {
                            out.push(instr);
                        }

                        continue;
                    }

                    // Extremely unsafe when there are prepared values ahead
                    // Memory can be overwritten
                    // Which should make sense since I/O are assumed to have no effect on memory.
                    // Folding with a temp counter breaks that assumption
                    let mut compr = vec![];
                    compr.push(BInstr::Move(1));
                    compr.extend(if self.level == 2 {
                        compress_incr(*n as i32, Some(5.0), false)
                    } else {
                        // > 3
                        compress_incr(*n as i32, None, false)
                    });
                    compr.extend(vec![
                        BInstr::LoopStart,
                        BInstr::Add(-1),
                        BInstr::Move(-1),
                        match instr {
                            BInstr::PutC(_) => BInstr::PutC(1),
                            BInstr::GetC(_) => BInstr::GetC(1),
                            _ => unreachable!(),
                        },
                        BInstr::Move(1),
                        BInstr::LoopEnd,
                    ]);
                    compr.push(BInstr::Move(-1));

                    let recons = compr.reconstruct();
                    if recons.len() < *n as usize {
                        out.extend(compr);
                    } else {
                        // no op
                        out.push(instr);
                    }
                }
                _ => out.push(instr),
            }
        }

        out
    }
}

#[derive(Debug)]
pub struct CompressConst {
    chunk: i32,
    sign: i32,
    remainder: i32,
    inner_count: i32,
    outer_fact: i32,
    weight: i32,
}

fn compress_incr_constants(count: i32, chunk: f32, upper: bool) -> CompressConst {
    // Idea:
    // One of the smallest/easiest way to represent something
    // close to 10 for example doing multiples of 5
    // e.g.
    // ++++++++++ can be written as >++[<+++++>-]
    // which is just 3 characters more

    let sign = count.signum();
    let count_f = (count as f32).abs();

    let k = count_f.log(chunk); // N = C^k => k = log_C(N) 
    let inner_count = (k.floor() - 1.0) as i32; // how many nested loop: 5^inner_count

    let k1 = k - k.floor();
    let k2 = chunk.powf(1.0 + k1);
    let outer_fact = if upper { k2.ceil() } else { k2.floor() }; // take 1.0 from latest exponent

    // Goal: N ~ outer_fact * C^inner_count
    // since C^(k1 + 1) . C^([k] - 1) = C^k
    let total = outer_fact * chunk.powf(inner_count as f32);
    let remainder = (count_f - total).floor() as i32;

    // The folded code should scale the same rate as this
    // Goal is to minimize it for a given count
    let weight = (outer_fact as i32) + inner_count * (chunk as i32) + remainder.abs();

    CompressConst {
        sign,
        chunk: chunk as i32,
        remainder,
        inner_count,
        outer_fact: outer_fact as i32,
        weight,
    }
}

pub fn find_best_parameters(count: i32, from: u32, to: u32, upper: bool) -> CompressConst {
    let mut best = compress_incr_constants(count, from as f32, upper);
    for chunk in (from + 1)..=to {
        let current = compress_incr_constants(count, chunk as f32, upper);
        if current.weight < best.weight {
            best = current;
        }
    }

    best
}

fn compress_incr(count: i32, chunk: Option<f32>, upper: bool) -> Vec<BInstr> {
    if count == 0 {
        // unreachable after basic fold
        return vec![];
    }

    let CompressConst {
        sign,
        chunk,
        remainder,
        inner_count,
        outer_fact,
        weight: _,
    } = if let Some(chunk) = chunk {
        compress_incr_constants(count, chunk, upper)
    } else {
        find_best_parameters(count, 2, 100, upper)
    };

    let mut out = vec![];
    out.push(BInstr::Move(inner_count));
    out.push(BInstr::Add(outer_fact));
    out = compress_incr_helper(out, sign * chunk, inner_count);
    out.push(BInstr::Move(-inner_count));
    out.push(BInstr::Add(sign * remainder));

    out
}

fn compress_incr_helper(mut out: Vec<BInstr>, fact: i32, loop_count: i32) -> Vec<BInstr> {
    if loop_count <= 0 {
        return out;
    }

    // [< ??? >-]
    out.push(BInstr::LoopStart);
    out.push(BInstr::Move(-1));
    out.push(BInstr::Add(fact));

    out = compress_incr_helper(out, fact, loop_count - 1);

    out.push(BInstr::Move(1));
    out.push(BInstr::Add(-1));
    out.push(BInstr::LoopEnd);

    out
}
