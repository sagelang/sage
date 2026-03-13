# Messaging

Agents can communicate by sending messages to each other.

## send

Sends a message to another agent:

```sage
send(agent_handle, message);
```

Example:

```sage
agent Receiver {
    on start {
        // Start and wait for messages
    }

    on message(text: String) {
        print("Received: " ++ text);
    }
}

agent Sender {
    on start {
        let r = spawn Receiver {};
        send(r, "Hello!");
        send(r, "World!");
        emit(0);
    }
}

run Sender;
```

## Message Types

Messages can be any type:

```sage
agent NumberReceiver {
    on message(n: Int) {
        print("Got number: " ++ str(n));
    }
}

agent ListReceiver {
    on message(items: List<String>) {
        for item in items {
            print(item);
        }
    }
}
```

## Type Safety

The message type must match the handler's expected type:

```sage
agent Worker {
    on message(n: Int) {
        print(str(n));
    }
}

agent Main {
    on start {
        let w = spawn Worker {};
        send(w, 42);       // OK
        send(w, "hello");  // Error: expected Int, got String
        emit(0);
    }
}
```

## Fire and Forget

`send` is asynchronous — it doesn't wait for the message to be processed:

```sage
agent Main {
    on start {
        let w = spawn Worker {};
        send(w, "message 1");  // Returns immediately
        send(w, "message 2");  // Returns immediately
        send(w, "message 3");  // Returns immediately
        // All three messages are queued
        emit(0);
    }
}
```

## Messaging vs Awaiting

| | `await` | `send` |
|---|---------|--------|
| Direction | Get result from agent | Send data to agent |
| Blocking | Yes, waits for result | No, returns immediately |
| Use case | Get final result | Ongoing communication |

## Example: Accumulator

```sage
agent Accumulator {
    belief initial: Int

    on start {
        // Just start - actual work in message handler
    }

    on message(n: Int) {
        // Note: beliefs are immutable, so this pattern
        // requires a different approach in practice
        print("Adding: " ++ str(n));
    }
}

agent Main {
    on start {
        let acc = spawn Accumulator { initial: 0 };

        send(acc, 10);
        send(acc, 20);
        send(acc, 30);

        sleep_ms(100);  // Give time for messages to process
        emit(0);
    }
}

run Main;
```

## Current Limitations

- No way to get a response to a message (use `spawn`/`await` instead)
- No message ordering guarantees between different senders
- No acknowledgment of message delivery

For request-response patterns, spawn a new agent and await its result instead of using messaging.
