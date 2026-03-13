# Types

Sage has a simple but expressive type system.

## Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | 64-bit signed integer | `42`, `-17` |
| `Float` | 64-bit floating point | `3.14`, `-0.5` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"hello"` |
| `Unit` | No value (like Rust's `()`) | — |

## Compound Types

### List\<T\>

Ordered collection of elements:

```sage
let numbers: List<Int> = [1, 2, 3];
let names: List<String> = ["Alice", "Bob"];
let empty: List<Int> = [];
```

## User-Defined Types

### Records

Define structured data with named fields:

```sage
record Point {
    x: Int,
    y: Int,
}

record Person {
    name: String,
    age: Int,
}
```

Construct records and access fields:

```sage
let p = Point { x: 10, y: 20 };
let sum = p.x + p.y;

let person = Person { name: "Alice", age: 30 };
print(person.name);
```

### Enums

Define types with a fixed set of variants:

```sage
enum Status {
    Active,
    Inactive,
    Pending,
}

enum Direction {
    North,
    South,
    East,
    West,
}
```

Use enum variants directly:

```sage
let s = Active;
let d = North;
```

### Match Expressions

Pattern match on enums and other values:

```sage
fn describe(s: Status) -> String {
    return match s {
        Active => "running",
        Inactive => "stopped",
        Pending => "waiting",
    };
}
```

Match on integers with a wildcard:

```sage
fn classify(n: Int) -> String {
    return match n {
        0 => "zero",
        1 => "one",
        _ => "many",
    };
}
```

The compiler checks that all variants are covered (exhaustiveness checking).

### Constants

Define compile-time constants:

```sage
const MAX_RETRIES: Int = 3;
const DEFAULT_NAME: String = "anonymous";
```

## Agent Types

### Agent\<T\>

A handle to a spawned agent that will emit a value of type `T`:

```sage
agent Worker {
    on start {
        emit(42);
    }
}

agent Main {
    on start {
        let w: Agent<Int> = spawn Worker {};
        let result: Int = await w;
        emit(result);
    }
}
```

### Inferred\<T\>

The result of an LLM inference call:

```sage
let summary: Inferred<String> = infer("Summarize: {topic}");
```

`Inferred<T>` can be used anywhere `T` is expected — the type coerces automatically.

## Type Inference

Sage infers types when possible:

```sage
let x = 42;              // Int
let name = "Sage";       // String
let list = [1, 2, 3];    // List<Int>
```

Explicit annotations are required for:
- Function parameters
- Beliefs
- Ambiguous cases

## Type Annotations

Use `: Type` syntax:

```sage
let x: Int = 42;
let items: List<String> = [];

fn double(n: Int) -> Int {
    return n * 2;
}

agent Worker {
    belief count: Int
}
```
