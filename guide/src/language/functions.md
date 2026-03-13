# Functions

Functions in Sage are defined at the top level and can be called from anywhere.

## Defining Functions

```sage
fn greet(name: String) -> String {
    return "Hello, " ++ name ++ "!";
}

fn add(a: Int, b: Int) -> Int {
    return a + b;
}
```

## Calling Functions

```sage
let message = greet("World");
let sum = add(1, 2);
```

## Return Types

All functions must declare their return type:

```sage
fn double(n: Int) -> Int {
    return n * 2;
}

fn print_message(msg: String) -> Unit {
    print(msg);
    return;
}
```

Use `Unit` for functions that don't return a meaningful value.

## Recursion

Functions can call themselves:

```sage
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn fibonacci(n: Int) -> Int {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
```

## Built-in Functions

Sage provides several built-in functions:

| Function | Signature | Description |
|----------|-----------|-------------|
| `print` | `(String) -> Unit` | Print to console |
| `str` | `(T) -> String` | Convert any value to string |
| `len` | `(List<T>) -> Int` | Get list length |
| `push` | `(List<T>, T) -> List<T>` | Append to list |
| `join` | `(List<String>, String) -> String` | Join strings |
| `int_to_str` | `(Int) -> String` | Convert int to string |
| `str_contains` | `(String, String) -> Bool` | Check substring |
| `sleep_ms` | `(Int) -> Unit` | Sleep for milliseconds |

## Example

```sage
fn summarize_list(items: List<String>) -> String {
    let count = len(items);
    let joined = join(items, ", ");
    return "Found " ++ str(count) ++ " items: " ++ joined;
}

agent Main {
    on start {
        let result = summarize_list(["apple", "banana", "cherry"]);
        print(result);
        emit(0);
    }
}

run Main;
```

Output:
```
Found 3 items: apple, banana, cherry
```
