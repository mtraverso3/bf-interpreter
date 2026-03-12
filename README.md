## Brainfuck Interpreter & Compiler

This repository contains a Brainfuck CLI with three tools:

- `interpret`: run Brainfuck programs directly.
- `compile`: compile Brainfuck programs to either LLVM IR or AArch64 Linux assembly.
- `minify`: removes non-Brainfuck characters and runs optimization passes.


### Building

```bash
cargo build --release
```

The binary can be found at `./target/release/bf-tools`.

### Usage

#### Interpreter

```
Usage: bf-tools interpret [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>    Path to the Brainfuck source file
  -o, --output <OUTPUT>  Optional file to write output to. Defaults to stdout
  -w, --wrapping         Enable wrapping of the data pointer at tape boundaries
  -s, --size <SIZE>      Size of the memory tape in cells [default: 30000]
  -d, --debug            Print the memory tape state at each instruction (debug mode)
```


#### Compile mode

```
Usage: bf-tools compile [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>
          Path to the Brainfuck source file

  -o, --output <OUTPUT>
          Optional file to write the compiled output to. Defaults to stdout

  -w, --wrapping
          Enable wrapping of the data pointer at tape boundaries

  -s, --size <SIZE>
          Size of the memory tape in cells

          [default: 30000]

  -t, --target <TARGET>
          Output target format

          [default: llvm]

          Possible values:
          - llvm: LLVM IR (.ll) — compile with: clang -O2 -o program out.ll
          - arm:  AArch64 Linux assembly (.s) — assemble with: as out.s -o out.o && ld out.o -o program
```

#### Minify

```
Usage: bf-tools minify [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>      Path to the Brainfuck source file
  -o, --output <OUTPUT>    Optional file to write the minified output to. Defaults to stdout
  --no-optimize            Disable all optimization passes (still strips comments/whitespace)
  --pass <PASS>            Optimization passes to run. Repeat this flag to run multiple passes
      Possible values:
      - fold-add-sub: Fold contiguous +/- runs modulo 256 and keep the shorter direction
      - canonicalize-clear-loops: Canonicalize `[+]` and `[-]` style zeroing loops to `[-]`
      - remove-known-zero-loops: Remove loops that are provably dead because the current cell is known to be zero
```

### License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE).
