pub struct Optimizer {}

// PASS 1. combine contiguous + and -
// fold into a loop by
// try 1 or more methods, use the shortest (ig situation dependent)
// try with ASCII prints

// PASS 2. find unreachable code (e.g ++->. shouldnt care about ++-)
// => keep track of memory bound induced by > and <

// PASS 3. optimize loops
// 1. flatten if provably doable
// PASS 1. and 2. for inner
