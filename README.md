# WORN "Write Once, Run Nowhere"

A minimalistic language that extends Brainf\*ck allowing the use of custom super
instructions and compiles into optimized Brainf\*ck code.

```rust
// A super instruction is a context aware generator that expands
// a parametrized Brainf*ck routine at compile time
// It is basically a macro but with validation
// Each argument is either a whole instruction, an inlined string or a number
super INSTR(arg1, arg2, ..) {
    // Bf code here
}
```

# Optimization

Optimization here does not mean make it run fast, but rather **shorten** the
character count and **maybe** make it fast and memory efficient along the way.

Two Braif\*ck programs are considered equal if the stdout and stdin side-effects
are the same, we do not really care about the resulting memory layout.

# Examples

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

>>>>"CD"<<.>.>. // "CD" is a shortcut for  R(67, +)>R(68, +)
```

The above code should expand to the following

```bf
ABCDEABB CD

>++++++++++[>++++++<-]>+++++.>++++++++++[>++++++<-]>++++++.>++++++++++[>++++++<-]>+++++++.+.+.>++++++++++[>++++++<-]>+++++.>++++++++++[>++++++<-]>++++++.>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.>>>>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++<<.>.>.
```

Super functions can be nested..

```rs
super incr(n) {
    super first(a) {
        super second(b) {
            super third(c) {
                R(b, +)
            }

            super fourth(d) {
                third(d)
            }

            fourth(b)
        }

        second(a)
    }

    first(n)
}
```
