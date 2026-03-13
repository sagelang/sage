# Spawning & Awaiting

Agents are created with `spawn` and their results are retrieved with `await`.

## spawn

Creates a new agent and returns a handle:

```sage
let worker = spawn Worker { task: "process data" };
```

The spawned agent starts running immediately and concurrently with the spawning agent.

### Spawn Syntax

```sage
spawn AgentName { belief1: value1, belief2: value2 }
```

All beliefs must be provided:

```sage
agent Point {
    belief x: Int
    belief y: Int
}

// Correct
let p = spawn Point { x: 10, y: 20 };

// Error: missing belief `y`
let p = spawn Point { x: 10 };
```

### Agent Handle Type

`spawn` returns an `Agent<T>` where `T` is the emit type:

```sage
agent Worker {
    on start {
        emit(42);  // Emits Int
    }
}

let w: Agent<Int> = spawn Worker {};
```

## await

Waits for an agent to emit its result:

```sage
let worker = spawn Worker {};
let result = await worker;  // Blocks until Worker emits
```

### Await Type

`await` returns the type that the agent emits:

```sage
agent StringWorker {
    on start {
        emit("done");
    }
}

let w = spawn StringWorker {};
let result: String = await w;
```

### Await Blocks

`await` suspends the current agent until the result is ready. Other agents continue running.

## Concurrent Execution

Spawned agents run concurrently:

```sage
agent Sleeper {
    belief ms: Int

    on start {
        sleep_ms(self.ms);
        emit(self.ms);
    }
}

agent Main {
    on start {
        // All three start immediately
        let s1 = spawn Sleeper { ms: 100 };
        let s2 = spawn Sleeper { ms: 200 };
        let s3 = spawn Sleeper { ms: 300 };

        // Total time: ~300ms (not 600ms)
        let r1 = await s1;
        let r2 = await s2;
        let r3 = await s3;

        emit(0);
    }
}

run Main;
```

## Pattern: Fan-Out/Fan-In

Spawn multiple workers, await all results:

```sage
agent Researcher {
    belief topic: String

    on start {
        let result: Inferred<String> = infer(
            "One sentence about: {self.topic}"
        );
        emit(result);
    }
}

agent Coordinator {
    on start {
        // Fan out
        let r1 = spawn Researcher { topic: "AI" };
        let r2 = spawn Researcher { topic: "Robotics" };
        let r3 = spawn Researcher { topic: "Quantum" };

        // Fan in
        let s1 = await r1;
        let s2 = await r2;
        let s3 = await r3;

        print(s1);
        print(s2);
        print(s3);
        emit(0);
    }
}

run Coordinator;
```

## Pattern: Pipeline

Chain agents together:

```sage
agent Step1 {
    belief input: String
    on start {
        let result = self.input ++ " -> step1";
        emit(result);
    }
}

agent Step2 {
    belief input: String
    on start {
        let result = self.input ++ " -> step2";
        emit(result);
    }
}

agent Main {
    on start {
        let s1 = spawn Step1 { input: "start" };
        let r1 = await s1;

        let s2 = spawn Step2 { input: r1 };
        let r2 = await s2;

        print(r2);  // "start -> step1 -> step2"
        emit(0);
    }
}

run Main;
```
