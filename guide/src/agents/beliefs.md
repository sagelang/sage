# Beliefs (State)

Beliefs are an agent's private state. They're initialized when the agent is spawned and can be accessed throughout the agent's lifetime.

## Declaring Beliefs

```sage
agent Person {
    belief name: String
    belief age: Int
}
```

Beliefs must have explicit type annotations.

## Initializing Beliefs

When spawning an agent, provide values for all beliefs:

```sage
let p = spawn Person { name: "Alice", age: 30 };
```

Missing beliefs cause a compile error:

```sage
// Error: missing belief `age` in spawn
let p = spawn Person { name: "Alice" };
```

## Accessing Beliefs

Use `self.beliefName` inside the agent:

```sage
agent Greeter {
    belief name: String

    on start {
        print("Hello, " ++ self.name ++ "!");
        emit(0);
    }
}
```

## Beliefs Are Immutable

Beliefs cannot be reassigned after initialization:

```sage
agent Counter {
    belief count: Int

    on start {
        // This won't work — beliefs are immutable
        // self.count = self.count + 1;

        // Use a local variable instead
        let count = self.count;
        count = count + 1;
        emit(count);
    }
}
```

## Entry Agent Beliefs

The entry agent (the one in `run`) cannot have required beliefs:

```sage
// Error: entry agent cannot have required beliefs
agent Main {
    belief config: String

    on start {
        emit(0);
    }
}

run Main;  // How would we provide `config`?
```

## Design Pattern: Configuration

Use beliefs to configure agent behavior:

```sage
agent Fetcher {
    belief url: String
    belief timeout: Int

    on start {
        // Use self.url and self.timeout
        emit("done");
    }
}

agent Main {
    on start {
        let f1 = spawn Fetcher {
            url: "https://api.example.com/a",
            timeout: 5000
        };
        let f2 = spawn Fetcher {
            url: "https://api.example.com/b",
            timeout: 3000
        };

        let r1 = await f1;
        let r2 = await f2;
        emit(0);
    }
}

run Main;
```

## Unused Belief Warning

The compiler warns about beliefs that are never accessed:

```sage
agent Example {
    belief used: Int
    belief unused: String  // Warning: unused belief `unused`

    on start {
        emit(self.used);
    }
}
```
