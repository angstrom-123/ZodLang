# Language (Pending a Better Name)
My attempt at writing a compiled programming language in Rust for x86_64 Linux.
This language compiles to assembly, assembles with nasm, and links with ld 
into a native executable. My goal is to avoid as many dependencies as I can, so 
I am avoiding extra crates and I will not be linking with standard libraries such 
as glibc. For ease of use, I am using Cargo for building and running tests, but 
this is not required.

## Usage
### Compile the Program
Debug Build:
> [!NOTE]
> Cargo is not required since no additional crates are used by this language.
```
cargo build
```

### Run the Tests 
```
cargo test
```

### Run the Compiler
```
<compiler_path> <input_file> <flags>
```

| Flag         | Shorthand | Argument | Meaning                             |
| -----------  | --------- | -------- | ----------------------------------- |
| --assembly   | -a        |          | Keep intermediate asm               |
| --include    | -I        | Dir Path | Specify path to search for includes |
| --inter-repr | -ir       |          | Emit intermediate representation    |
| --optimise   | -O        |          | Enable optimisation                 |
| --output     | -o        | Out Path | Specify output path                 |
| --parsetree  | -pt       |          | Print parse tree                    |
| --run        | -r        |          | Run after compiling                 |
| --tokens     | -t        |          | Print lexed tokens                  |
| --verbose    | -v        |          | Enable info logging                 |

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
- [x] Turing complete
- [ ] Assignment arithmetic operators (e.g. +=)
- [ ] Increment and decrement operators
- [ ] Logical negation operator
- [ ] Bitwise operators
- [ ] Conditional elif
- [ ] Ternary expressions
- [x] Compile-time type checking
- [x] Function param counting
- [ ] Global variables 
- [x] String type
- [x] Character type
- [ ] Structs 
- [x] Including other files
