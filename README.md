# WORN "Write Once, Run Nowhere"

A minimalistic language that extends Brainf\*ck allowing the use of custom super
instructions and compiles into optimized Brainf\*ck code.

# Overview

## CLI

```
WORN (Write Once, Run Nowhere): The "ultimate" Brainfuck emitter/compiler/optimizer

Usage: worn [OPTIONS] <FILE>

Arguments:
  <FILE>  Input source file

Options:
  -o <OUTPUT>                Set the output file
  -O, --optimize <OPTIMIZE>  Custom optimization level [default: 3]
  -p, --print
  -h, --help                 Print help
```

## Notions

```rust
// A super instruction is a context aware generator that expands
// a parametrized Brainf*ck routine at compile time
// It is basically a macro but with validation
// Each argument is either a whole instruction, an inlined string or a number
super INSTR(arg1, arg2, ..) {
    // Brainf\*ck or worn code here
}
```

## Examples

```rust
super incr(n) {
    // R is a native super instruction that repeats the second argument the amount of the first argument
    R(n, +) 
}

super six() {
    incr(6) // instruction as arg works as long as they are simple and of the same 'kind' (e.g. contiguous +)
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

The above code should expand into the following (optimization disabled)

```rust
// Should print ABCDEABB CD

>++++++++++[>++++++<-]>+++++.>++++++++++[>++++++<-]>++++++.>++++++++++[>++++++<-]>+++++++.+.+.>++++++++++[>++++++<-]>+++++.>++++++++++[>++++++<-]>++++++.>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.>>>>+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++>++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++<<.>.>.

// After -O4, from 368 to 254 instructions
>++++++++++[>++++++<-]>+++++.>++++++++++[>++++++<-]>++++++.>++++++++++[>++++++<-]>+++++++.+.+.>++++++++++[>++++++<-]>+++++.>++++++++++[>++++++<-]>++++++.>>++++++++++++++[<+++++>-]<----.>>>>>++++++++++++++[<+++++>-]<--->>++++++++++++++[<+++++>-]<--<<.>.>.
```

Super functions can be nested..

```rust
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

## Optimization schemes

Optimization here does not mean make it run fast, but rather **shorten** the
instruction count and **maybe** make it fast and memory efficient along the way.

Two Braif\*ck programs are considered equal if the stdout and stdin are the
same, we do not really care about the resulting memory layout.

The most obvious step is expression folding, that can be decided at compile
time.

```rust
R(68, +)R(68, -)<<>>+.

// becomes
+.
```

When a fold is too large, we can break it down into a multiplications. To be
considered large, a +/- fold needs to be higher than 10.

```rust
R(69, +).

// becomes
+++[>+++++[>++++<-]<-]>>+++++++++.
```

Which is effectively smaller than 69 consecutive '+' (34 instructions), and even
in this case the following is shorter (32 instructions).

```rust
+++[>+++++[>+++++<-]<-]>>------.
```

To preserve memory layout the resulting code is slightly denser (35
instructions).

```rust
>>>+++[<+++++[<+++++>-]>-]<<------.
```

Why this form?

For example, one of the shortest/easiest way to represent a value close to 10 is
doing 2 * 5 since `++++++++++` is just `>++[<+++++>-]` (only 3 instructions
more).

```rust
// 16 = 3 * 5 + 1
>+++[<+++++>-]+

// 20 = 4 * 5 + 0
>++++[<+++++>-]+

// 32 = 6 * 5 + 2
>++++++[<+++++>-]+
```

Works well but we can do better with nested loops (requires temporary memory).
The trick is to loop close to the target.

An easy trick that I found very useful (in the context of code generation) is to
find the exponent of some number C that is closest to the target value. Use that
to define the amount of inner loop to effectively perform a multiplication.

Assume

$$N = C^k = C^{\lfloor k \rfloor -1} \times \lceil C^{ k - \lfloor k \rfloor + 1 } \rceil + \delta$$

So if we have some value $N = 169$ and $C = 3$
$$169 = 3^k \rightarrow k = log_3 (169) \approx 4.669$$

We break down into an inner loop count $\lfloor 4.669 \rfloor-1 = 3$, and an
outer factor $\lceil 3^{4.669 - \lfloor 4.669 \rfloor + 1 = 1.669} \rceil = 7$.

This trick works and is exact as whatever the difference $\delta$, we get it
minus the target, then we add/sub to compensate.

$$\delta = N - C^{\lfloor k \rfloor -1} \times \lceil C^{ k - \lfloor k \rfloor + 1 } \rceil = 169 - 3^{3} \times 7 = -20$$

Then we get..

```rust
// 169
R(169, +)

// Compiles into
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

// Folds into
>>>+++++++[<+++[<+++[<+++>-]>-]>-]<<<--------------------

// After another pass..
>>>+++++++[<+++[<+++[<+++>-]>-]>-]<<<>+++++++[<--->-]<+

// And another basic fold pass..
>>>+++++++[<+++[<+++[<+++>-]>-]>-]<<+++++++[<--->-]<+
```

When we have repeating I/O, we can do the same. We simply decrement, print/get,
repeat until we reach 0, the decrement amount is just a value that is folded
using the technique previouvsly discussed.

This does break an assumption though: "I/O does not write into memory", as such
it is unsafe for some programs.

```rust
// stdout
> amount_fold [-<.>]<
// stdin
> amount_fold [-<,>]<
```

When does this break? A simple case is reusing contiguous buffers, one common
case is when you want to prepare a few contiguous cells. In general, it is not a
real problem though. If your code has many consecutive prints you can always
express it in a way that it is safely optimizable.

```rust
// This will break as it relies on B never being overwritten
"AB" < R(50, .) > R(50, .)
// -O4 -a unsafe-fold-io
// >+++++++++++[<++++++>-]<->>>++++++++[<+++[<+++>-]>-]<<------>>>>+++[<++[<++[<++[<++>-]>-]>-]>-]<<<<++[-<.>]>>>>>+++[<++[<++[<++[<++>-]>-]>-]>-]<<<<++[-<.>]<


// That can be fixed though and can be rewritten as
"A" R(50, .) > "B" R(50, .)
// -O4 -a unsafe-fold-io
// >+++++++++++[<++++++>-]<->>>>>+++[<++[<++[<++[<++>-]>-]>-]>-]<<<<++[-<.>]>>++++++++[<+++[<+++>-]>-]<<------>>>>>+++[<++[<++[<++[<++>-]>-]>-]>-]<<<<++[-<.>]<
```

This can be enabled with `-a unsafe-fold-io` and `-O3` or higher.

> [!WARNING]
>
> Although I made some accent on I/O in particular, most above optimization
> tricks work because they all assume that at any point in the program any
> upcoming memory cells are all 0. There is no way to check that at compile
> without running the actual program.
>
> I/O folding can be very unsafe but the other techniques should work on most
> programs.
>
> When authoring a program you can rewrite it in a form that enables the above
> assumptions to enable the optimizers.
>
> You can disable optimization with `-O0` or use simple folding with `-O1`.
