# Language (Pending a Better Name)
My attempt at writing a compiled programming language in Rust for x86_64 Linux.
This language compiles to assembly, assembles with nasm, and 
links with ld into a native executable.

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
An examples folder is included with the project showcasing the language features 
and giving real syntax examples. Combined with the listed features below, this 
should be enough to get a basic grasp on the syntax.

## Language Features
### Keywords
```
dump <expression>;

exit <expression>;
```

### Functions
```
func <function_name> {
    <body>
}

<function_name>();
```

### Loops 
```
for <init>; <condition>; <post> {
    <body>
}

while <condition> {
    <body>
}
```

### Local Variables
```
let <variable_name>;

let <variable_name> = <expression>;

<variable_name> = <expression>;
```

### Conditional Statements
```
if <condition> {
    <body>
}

if <condition> {
    <body>
} else {
    <body>
}
```

### Arithmetic Operators
```
let a;

a = 1 + 1;

a = 1 + 2 * 3;

a = 10 / 2 * (1 + 3);
```

### Logical Operators 
```
if 1 == 1 {}

if 1 >= 1 {}

if 1 && (0 || 1) {}

if 10 < 20 && 20 > 15 {}
```
