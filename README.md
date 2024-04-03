## Brainfuck Interpreter

This repo contains a small CLI tool that interprets [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) programs.

### Building
This project is written in Rust and can be easily built from source.

Build the binary by running:
```bash
cargo build --release
```

Once built, the binary can be found at `./target/release/brainfuck-interpreter`.

### Usage

Simply pass in a file containing a Brainfuck program:

```bash
./brainfuck-interpreter --input /path/to/program
```

The interpreter also accepts the following flags and arguments:

- `--debug`:  Enables debug information. This will print the memory tape at each instruction
- `--wrapping`: Enables wrapping of the data pointer.
- `--size <SIZE>`: The size of the memory tape [default: 30000]
- `--output <OUTPUT>`: An optional file to output to. If not specified, the program uses stdout

### Licence

This project is licensed under the GNU AGPL v3.0 license—see the [LICENSE](LICENSE) file for details.
