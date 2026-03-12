## Brainfuck Interpreter & Compiler

This repo contains a small CLI tool that interprets [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) programs and can compile them to AArch64 (ARM64) assembly.

### Building
This project is written in Rust and can be easily built from source.

Build the binary by running:
```bash
cargo build --release
```

Once built, the binary can be found at `./target/release/brainfuck`.

### Usage

The tool has two subcommands: `interpret` and `compile`.

#### Interpret

Run a Brainfuck program directly:

```bash
./brainfuck interpret --input /path/to/program.bf
```

Available flags:

- `-i, --input <INPUT>`: Path to the Brainfuck source file *(required)*
- `-o, --output <OUTPUT>`: Optional file to write output to. Defaults to stdout
- `-w, --wrapping`: Enable wrapping of the data pointer at tape boundaries
- `-s, --size <SIZE>`: Size of the memory tape in cells [default: 30000]
- `-d, --debug`: Print the memory tape state at each instruction

#### Compile

Compile a Brainfuck program to AArch64 assembly:

```bash
./brainfuck compile --input /path/to/program.bf --output out.s
```

Available flags:

- `-i, --input <INPUT>`: Path to the Brainfuck source file *(required)*
- `-o, --output <OUTPUT>`: Optional file to write the assembly output to. Defaults to stdout

The generated `.s` file can then be assembled and linked on a Linux AArch64 system:

```bash
as -o out.o out.s
ld -o program out.o
./program
```

### Licence

This project is licensed under the GNU AGPL v3.0 license—see the [LICENSE](LICENSE) file for details.
