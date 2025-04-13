# W.O.R.N: "Write Once, Run Nowhere"

A minimalistic language that extends Brainf\*ck allowing the use of custom super
instructions and compiles into optimmized Brainf\*ck code.

```rust
// A super instruction is a context aware generator that expands
// a parametrized Brainf*ck routine at compile time
// It is basically a macro but with validation
// Each argument is either a whole instruction, an inlined string or a number
super INSTR(arg1, arg2, ..) {
    // Bf code here
}
```

Example:

```rust
super incr(n) {
    // R repeats the second argument the amount of the first argument
    R(n, +) 
}

super six() {
    incr(6) // instruction as arg works as long as they are simple and does not have ',' as it is a valid bf instruction
}

super ten() {
    six()++++
}

super printASCII(index) {
    > ten() [>six()<-]> +++++ incr(index) .
}

printASCII(0) // A
printASCII(1) // B
printASCII(2) // C
+ . // D
+ . // E
printASCII(0) // A
printASCII(1) // B
>65+. // B

>>>>"CD"<<.>.>. // "CD" is a shortcut for  R(68, +)>R(69, +)
```

# Why?

This is a toy project for that showcases static code analysis and static
optimization.
