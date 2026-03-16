# Lingot

A concise, safe, and readable scripting language built with Rust.

Lingot is designed to be easy to learn for non-programmers while remaining powerful for experienced developers. It features type inference, string interpolation, built-in file operations, shell interop, and a minimal keyword set — all compiled to a fast native binary.

## Features

- **Minimal syntax** — 26 reserved keywords, semicolons optional, `//` comments
- **Type inference** — types are inferred at initialization and locked (optional annotations)
- **Immutable by default** — `let` is immutable, `let dyn` enables reassignment
- **String interpolation** — `"Hello {name}, you are {age}"`
- **Built-in file operations** — `read()`, `write().to()`, `move().to()`, `delete()`, `list()`
- **Shell interop** — `shell("git status")` executes system commands
- **Error handling** — `try/catch` + `fail` with automatic propagation
- **Functions** — first-class, generic via monomorphisation
- **REPL** — interactive mode for quick experiments
- **Fast** — ~1 MB native binary, instant startup

## Quick Start

### Install from crates.io

```bash
cargo install lingot
```

### Build from source

```bash
git clone https://github.com/theoctano/lingot.git
cd lingot
cargo build --release
cp target/release/lingot /usr/local/bin/
```

### First script

Create `hello.ling`:

```
let name = "World"
display("Hello {name}!")

let numbers = [1, 2, 3, 4, 5]
repeat {
  display("{n} squared = {n * n}")
} for (n in numbers)
```

Run it:

```bash
lingot run hello.ling
```

## Language Overview

### Variables

```
let name = "Alice"           // immutable, type inferred as Text
let dyn counter = 0          // mutable, type inferred as Number
counter = counter + 1        // ok
counter = "hello"            // error: type locked to Number
```

### Functions

```
let greet (who) {
  return "Hello {who}!"
}

display(greet("World"))      // Hello World!
display(greet(42))           // Hello 42!
```

### Control Flow

```
if (score > 100) {
  display("High score!")
} else {
  display("Keep trying")
}

while (x > 0) {
  x = x - 1
}

repeat {
  display(i)
} for (i in 1..10)

repeat {
  display(item)
} for (item in items)
```

### Error Handling

```
let divide (a, b) {
  if (b == 0) {
    fail "Division by zero"
  }
  return a / b
}

try {
  let result = divide(10, 0)
  display(result)
} catch (error) {
  display("Error: {error}")
}
```

### Shell

```
try {
  let status = shell("git status")
  display(status)
} catch (error) {
  display("Command failed: {error}")
}
```

### File Operations

```
write("Hello from Lingot!").to("output.txt")
let content = read("output.txt")
display(content)

move("output.txt").to("archive/output.txt")
rename("archive/output.txt").to("backup.txt")
delete("archive/backup.txt")

let files = list(".")
display(files)
```

## Built-in Functions

| Function | Description |
|----------|-------------|
| `display(value)` | Print a value to stdout |
| `shell(command)` | Execute a system command, return stdout |
| `read(path)` | Read a file, return its content as Text |
| `write(content).to(path)` | Write content to a file |
| `move(source).to(dest)` | Move a file |
| `rename(path).to(name)` | Rename a file |
| `delete(path)` | Delete a file or directory |
| `list(path)` | List directory contents |
| `prefix(path).with(text)` | Add a prefix to a filename |
| `suffix(path).with(text)` | Add a suffix to a filename |

## Operators

Lingot supports both symbolic and keyword-based operators:

| Symbolic | Keyword | Description |
|----------|---------|-------------|
| `&&` | `and` | Logical AND |
| `\|\|` | `or` | Logical OR |
| `!` | `not` | Logical NOT |
| `==` | `is` | Equality |
| `!=` | `is not` | Inequality |
| `>` | `greater than` | Greater than |
| `<` | `lesser than` | Less than |

## Types

| Type | Example |
|------|---------|
| `Text` | `"hello"` |
| `Number` | `42`, `3.14` |
| `Bool` | `true`, `false` |
| `List` | `[1, 2, 3]` |
| `Void` | implicit return of functions without `return` |

Numbers unify integers and floats. Integer division stays integer (`7 / 2 = 3`), float involvement produces floats (`7.0 / 2 = 3.5`). Division by zero is always a `fail`.

## Security

Lingot is a scripting language. The `shell()` function executes arbitrary system commands. Only run `.ling` scripts you trust, the same way you would with bash or Python scripts.

File operations (`read`, `write`, `move`, `delete`) use Rust's standard library directly with no shell intermediary.

## CLI

```bash
lingot run <file.ling>    # Run a script
lingot repl               # Start the interactive REPL
lingot --version          # Show version
lingot --help             # Show help
```

## License

MIT
