# Language (Pending a Better Name)
My attempt at writing a compiled programming language in Rust for x86_64 Linux.
This language compiles to assembly, assembles with nasm, and links with ld 
into a native executable. My goal is to avoid as many dependencies as I can, so 
I am avoiding extra crates, and I will not be linking with standard libraries such 
as glibc.

## Usage
### Compile the Program
Debug Build:
```
cargo build
```

### Run the Tests 
```
cargo test
```

### Run the Compiler
```
./<compiler_path> <file_path> <flags>
```

| Flag         | Shorthand | Argument | Meaning               |
| -----------  | --------- | -------- | --------------------- |
| --parse-tree | -pt       |          | Print parse tree      |
| --assembly   | -a        |          | Keep intermediate asm |
| --tokens     | -t        |          | Print lexed tokens    |
| --run        | -r        |          | Run after compiling   |
| --output     | -o        | Out Path | Specify output path   |

## Examples
See the examples folder in the project root for some programs showcasing the 
language.

## Features
- [x] Compiling to native code
- [x] Integer type
- [x] Arithmetic with precedence
- [x] Locally scoped variables
- [x] Conditional if and else
- [x] For and while loops
- [x] Functions with params
- [ ] Assignment arithmetic operators (e.g. +=)
- [ ] Increment and decrement operators
- [ ] Logical negation operator
- [ ] Bitwise operators
- [ ] Conditional elif
- [ ] Ternary expressions
- [ ] Compile-time type checking
- [ ] Function param counting
- [ ] Global variables 
- [ ] String type
- [ ] Character type
- [ ] Structs 
- [ ] Including other files
