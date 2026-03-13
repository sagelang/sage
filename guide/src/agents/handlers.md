# Event Handlers

Agents respond to events through handlers. Each handler runs when its corresponding event occurs.

## on start

Runs when the agent is spawned:

```sage
agent Worker {
    on start {
        print("Worker started!");
        emit(42);
    }
}
```

Every agent must have an `on start` handler — it's where the agent's main logic lives.

## on message

Runs when the agent receives a message:

```sage
agent Accumulator {
    on start {
        // Wait for messages
    }

    on message(value: Int) {
        print("Received: " ++ str(value));
    }
}
```

See [Messaging](./messaging.md) for details on sending messages.

## on stop

Runs when the agent is about to terminate (not yet implemented):

```sage
agent Worker {
    on start {
        // Do work
        emit(0);
    }

    on stop {
        print("Cleaning up...");
    }
}
```

## Handler Order

1. `on start` runs first, exactly once
2. `on message` can run multiple times, whenever a message arrives
3. `on stop` runs last, after `emit`

## emit

The `emit` expression signals that the agent has produced its result:

```sage
agent Calculator {
    belief a: Int
    belief b: Int

    on start {
        let result = self.a + self.b;
        emit(result);  // Agent is done
    }
}
```

After `emit`:
- The agent's result is available to whoever awaited it
- The agent proceeds to cleanup (`on stop`)
- No more messages are processed

## Emit Type Consistency

All `emit` calls in an agent must have the same type:

```sage
agent Example {
    on start {
        if condition {
            emit(42);      // Int
        } else {
            emit("error"); // Error: expected Int, got String
        }
    }
}
```

## Handler Scope

Each handler has its own scope. Variables don't persist between handlers:

```sage
agent Example {
    on start {
        let x = 42;
        // x is only visible here
    }

    on message(n: Int) {
        // x is not visible here
        // Use beliefs for persistent state
    }
}
```

Use beliefs for state that needs to persist across handlers.
