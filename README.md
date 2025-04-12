# sbf

A minimalistic language that extends Brainf*ck allowing the use of custom super
instructions and compiles into optimmized Brainf*ck code.

```rust
// A super instruction is a context aware generator that expands
// a parametrized Brainf*ck routine at compile time
// It is basically a macro but with validation that ensures
// that the enderlying instruction sequence does not ovewrite into potentially "shared" memory

// Each argument is either a whole instruction, an inlined string or a number
super INSTR(arg1, arg2, ..) {
    // Bf code here
}
```

Example:

```rust
super six() {
    // R repeats the first argument the amount of the second argument
    R(+, 6)
}

super ten() {
    six()++++
}

super printASCII(index) {
    > ten() [>six()<-]> +++++ R(+, index) .
}

printASCII() // A
printASCII(+) // B
printASCII(++) // C
+ . // D
++ . // E
printASCII() // E
printASCII(+) // F

>65+. // B
>"CD".>. // "CD" is a shortcut for  R(+, 68)>R(+, 69)
```

# Why?

This is a toy project for that showcases static code analysis and static
optimization.
