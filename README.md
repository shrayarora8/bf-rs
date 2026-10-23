# bf-rs

A Brainfuck interpreter in Rust. I parse the program text into a syntax tree first and then run the tree, instead of running the characters directly. Matching `[` with `]` is checked once while parsing, so the interpreter never has to search for a bracket while the program runs.

[Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) is a tiny language with eight commands. They move a pointer along a row of byte cells (`>` `<`), change the current cell (`+` `-`), read and write bytes (`,` `.`), and loop while the current cell is non-zero (`[` `]`).

## Running it

You need Rust ([rustup.rs](https://rustup.rs)).

```bash
git clone https://github.com/shrayarora8/bf-rs
cd bf-rs
cargo run -q -- examples/hello.bf
```
```
Hello World!
```

Programs that read input (the `,` command) take it from stdin.

```bash
printf 'hello' | cargo run -q -- examples/reverse.bf
```
```
olleh
```

```bash
printf '34' | cargo run -q -- examples/add.bf
```
```
7
```

A program with unmatched brackets is rejected before anything runs.

```bash
echo '+[-]]' > bad.bf
cargo run -q -- bad.bf
```
```
bad.bf: brackets don't match
```

## How it works

Here is what happens to `,>,[<+>-]<.`, which adds two input bytes.

```
",>,[<+>-]<."                                   program text
        ↓  parse_bf
Sequence([InpByte, IncData, InpByte,
          Loop(Sequence([DecData, IncCurrByte, IncData, DecCurrByte])),
          DecData, PrintCurrByte])              syntax tree
        ↓  BFInterpreter::interpret, input [5, 6]
[11]                                            output
```

**The tree.** `BFProg` is an enum with one variant per simple command, plus `Sequence` (programs run one after another) and `Loop` (a program between brackets). There is no variant for a lone `[` or `]`, so an unmatched bracket can't be represented at all.

**Parsing.** `parse_bf` reads the text left to right and keeps a stack of lists. Each `[` pushes a new list for the loop's body. Each `]` pops that list, wraps it in a `Loop`, and adds it to the list underneath. A `]` with no loop open, or text that ends with a loop still open, returns `None`. Characters that aren't commands are ignored, so comments can go anywhere in a program.

**Running.** `interpret` walks the tree recursively. A `Loop` becomes a Rust `while` loop that runs its body while the current cell is non-zero.

## Behavior at the edges

Brainfuck leaves a few things undefined. These are the choices I made.

- Cells are bytes that wrap around (255 + 1 is 0, and 0 - 1 is 255).
- The tape starts with one cell and grows to the right as the pointer moves.
- Reading input after it has run out stores 0.
- Moving left of the first cell stops the program with an error.
- Output is collected and written when the program finishes, so a program that never finishes prints nothing.

## Tests

`cargo test` runs 8 tests.

- **Printing (2)** check that a tree prints back as the right text, and that the closest program you can build to the invalid `]><+[]` doesn't print as it.
- **Running (2)** check that adding 5 and 6 outputs 11, and that reading past the end of the input gives 0 and decrementing 0 wraps to 255.
- **Parsing (4)** check that valid text becomes the right tree, that three kinds of unmatched brackets are rejected, that parsing and then running gives 11, and that printing a tree and parsing it back can give a different tree that prints the same (`+` comes back as `Sequence([+])`).
