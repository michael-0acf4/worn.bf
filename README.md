# worn

"Write Once, Run Nowhere"

A minimalistic language that extends Brainf\*ck allowing the use of custom super
instructions and compiles into optimmized Brainf\*ck code.

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
super incr(n) {
    // R repeats the first argument the amount of the second argument
      R(I(+), n) 
       // ^^
       // inline instructions should work as arguments as long as they are single,
       // or multiple with each not having ',' as it is a valid bf instruction
       // otherwise the remaining instruction are considered to be part of the first half
}

super six() {
    incr(6)
}

super ten() {
    six()++++
}

super printASCII(index) {
    > ten() [>six()<-]> +++++ incr(index) .
}

printASCII() // A
printASCII(+) // B
printASCII(++) // C
+ . // D
++ . // E
printASCII() // E
printASCII(+) // F

>65+. // B
>"CD".>. // "CD" is a shortcut for  R(I(+), 68)>R(I(+), 69)
```

# Why?

This is a toy project for that showcases static code analysis and static
optimization.
