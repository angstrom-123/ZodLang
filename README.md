# ZodLang - A Simple Toy Programming Language
My attempt at writing a compiled programming language in Rust for x86_64 Linux.
ZodLang compiles to assembly, assembles with nasm, and links with ld 
into a native executable. My goal is to avoid as many dependencies as I can, so 
I am avoiding extra crates and I will not be linking with standard libraries such 
as glibc. For ease of use, I am using Cargo for building and running tests, but 
this is not required.

## Usage
### Compile the Program
```
cargo build -r
```
> [!NOTE]
> Cargo is not required since no additional crates are used by zodlang, so you 
> can compile using rustc fairly easily.

### Run the Tests 
```
cargo test
```

### Run the Compiler
```
<path_to_zodc> <input_file> <flags>
```

By default, after running `cargo build -r`,  zodc will be in `./target/release/zodc`.

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
The language is Turing complete and usable for simple tasks, but it lacks several 
convenience features for the sake of easy implementation. For example, array subscript 
syntax is only supported on the heap, and only in one dimension. Perhaps I will update 
this in the future, but it is more likely that I will write a new language instead.

- [x] Compiling to native code
- [x] Integer type
- [x] Character type
- [x] String type (character array)
- [x] Arithmetic with precedence
- [x] Locally scoped variables
- [x] Conditional if and else
- [x] For and while loops
- [x] Functions with params
- [x] Turing complete
- [x] 1-Dimensional array subscript syntax 
- [x] Compile-time type checking
- [x] Function param counting
- [x] Standard library
- [x] Including other files
- [ ] N-Dimensional array subscript syntax 
- [ ] Complete syntax for stack based arrays
- [ ] Assignment arithmetic operators (e.g. +=)
- [ ] Increment and decrement operators
- [ ] Logical negation operator
- [ ] Bitwise operators
- [ ] Conditional elif
- [ ] Ternary expressions
- [ ] Global variables 
- [ ] Structs 
