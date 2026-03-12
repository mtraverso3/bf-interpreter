## Brainfuck Interpreter & Compiler

This repository contains a Brainfuck CLI with two tools:

- `interpret`: run Brainfuck programs directly.
- `compile`: compile Brainfuck programs to either LLVM IR or AArch64 Linux assembly.


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

### License

This project is licensed under the GNU AGPL v3.0 license. See [LICENSE](LICENSE).
