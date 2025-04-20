use crate::parser::ast::{BInstr, Reconstruct};

pub struct Optimizer {
    pub level: u8,
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

        while let Some(instr) = iter.next() {
            match &instr {
                BInstr::Add(n) => {
                    let mut agg = *n;
                    while let Some(next) = iter.clone().next() {
                        if let BInstr::Add(m) = &next {
                            agg += *m;
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    if agg != 0 {
                        out.push(BInstr::Add(agg));
                    }
                }
                BInstr::Move(n) => {
                    let mut agg = *n;
                    while let Some(next) = iter.clone().next() {
                        if let BInstr::Move(m) = &next {
                            agg += *m;
                            iter.next();
                        } else {
                            break;
                        }
                    }

                    if agg != 0 {
                        out.push(BInstr::Move(agg));
                    }
                }
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

                    let best_fold = if self.level == 2 {
                        self.compress_incr(*n, 5.0)
                    } else {
                        // > 3
                        let best = [3, 4, 5, 8, 10]
                            .map(|base| (base, self.compress_incr(*n, base as f32)))
                            .into_iter()
                            .min_by_key(|(base, v)| (*base, v.len()));

                        if let Some((_base, best)) = best {
                            // println!("Found best chunk size {base}, will be used as base.");
                            best
                        } else {
                            unreachable!()
                        }
                    };

                    if best_fold.reconstruct().len() < (*n).abs() as usize {
                        out.extend(best_fold);
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

    fn compress_incr(&self, count: i32, chunk: f32) -> Vec<BInstr> {
        if count == 0 {
            // unrechable if pass1_fold applied
            return vec![BInstr::Add(count)];
        }

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

        // how much to multiply outside
        let k1 = k - k.floor();
        let outer_fact = chunk.powf(1.0 + k1).ceil(); // take 1.0 from latest exponent

        // Goal: N ~ outer_fact * C^inner_count
        // since C^(k1 + 1) . C^([k] - 1) = C^k
        let total = outer_fact * chunk.powf(inner_count as f32);
        let remainder = (count_f - total).floor() as i32;

        // println!(
        //     "Chunk {chunk}, Input {count}, k {k}, k1 {k1} => outer {outer_fact}, total {total}, rem {remainder}"
        // );

        let mut out = vec![];

        out.push(BInstr::Move(inner_count));
        out.push(BInstr::Add(outer_fact as i32)); // account for k2

        out = self.compress_incr_helper(out, sign * chunk as i32, inner_count); // account for k1

        out.push(BInstr::Move(-inner_count));
        out.push(BInstr::Add(sign * remainder));
        // or.. fold

        out
    }

    fn compress_incr_helper(
        &self,
        mut out: Vec<BInstr>,
        fact: i32,
        loop_count: i32,
    ) -> Vec<BInstr> {
        if loop_count <= 0 {
            return out;
        }

        // [< ??? >-]
        out.push(BInstr::LoopStart);
        out.push(BInstr::Move(-1));
        out.push(BInstr::Add(fact));

        out = self.compress_incr_helper(out, fact, loop_count - 1);

        out.push(BInstr::Move(1));
        out.push(BInstr::Add(-1));
        out.push(BInstr::LoopEnd);

        out
    }
}
