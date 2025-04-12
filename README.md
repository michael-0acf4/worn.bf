# sbf

A minimalistic language that extends Brainf*ck allowing the use of custom super
instructions and compiles into optimmized Brainf*ck code.

```rust
// A super instruction is a context aware generator that expands
// a parametrized Brainf*ck routine at compile time
// It is basically a macro but with validation that ensures
// that the enderlying instruction sequence does not ovewrite into potentially "shared" memory
super INSTR(arg1, arg2, ..) {
    // Bf code here
}

// A super instruction can call other super instruction
super printASCII(index) {
    > incr(10) [>add(6)<-]> +++++ add(index) .
}

printASCII(0) // A
printASCII(1) // B
printASCII(2) // C
+ . // D
++ . // E
printASCII(0) // E
printASCII(1) // F
```

# Why?

This is a toy project for that showcases static code analysis and static
optimization.
