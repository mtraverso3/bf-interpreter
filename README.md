## Brainfuck Interpreter & Compiler

This repository contains a Brainfuck CLI with two modes:

- `interpret`: run Brainfuck programs directly.
- `compile`: compile Brainfuck programs to either LLVM IR or AArch64 Linux assembly.

The parser builds an AST first, then both execution backends operate on that AST.

### Building

```bash
cargo build --release
```

Binary path:

- `./target/release/brainfuck-interpreter`

### Usage

#### Interpret mode

```bash
./target/release/brainfuck-interpreter interpret --input /path/to/program.bf
```

Flags:

- `-i, --input <INPUT>`: Brainfuck source file path (required)
- `-o, --output <OUTPUT>`: optional output file (defaults to stdout)
- `-w, --wrapping`: enable data pointer wrapping at tape boundaries
- `-s, --size <SIZE>`: tape size in cells (default: `30000`, must be `> 0`)
- `-d, --debug`: print interpreter state at each instruction

`','` input semantics in interpreter are byte-based: read exactly one byte from stdin (EOF maps to `0`).

#### Compile mode

```bash
./target/release/brainfuck-interpreter compile --input /path/to/program.bf
```

Flags:

- `-i, --input <INPUT>`: Brainfuck source file path (required)
- `-o, --output <OUTPUT>`: optional output file (defaults to stdout)
- `-t, --target <TARGET>`: output target (`llvm` default, `arm` optional)
- `-w, --wrapping`: emit wrapping pointer behavior in generated code
- `-s, --size <SIZE>`: tape size in cells (default: `30000`, must be `> 0`)

### Targets

#### LLVM IR (default)

Generate `.ll` and build with clang:

```bash
./target/release/brainfuck-interpreter compile --input my.bf --output out.ll
clang -O2 -o program out.ll
./program
```

#### AArch64 Linux assembly

Generate `.s` for AArch64 Linux:

```bash
./target/release/brainfuck-interpreter compile --target arm --input my.bf --output out.s
```

Assemble and link on an AArch64 Linux system:

```bash
as -o out.o out.s
ld -o program out.o
./program
```

### License

This project is licensed under the GNU AGPL v3.0 license. See [LICENSE](LICENSE).
