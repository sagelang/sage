# Introduction

Sage is a programming language where **agents are first-class citizens**.

Instead of building agents using Python frameworks like LangChain or CrewAI, you write agents as naturally as you write functions. Agents, their beliefs, and their interactions are semantic primitives baked into the compiler and runtime.

```sage
agent Researcher {
    belief topic: String

    on start {
        let summary: Inferred<String> = infer(
            "Write a concise 2-sentence summary of: {self.topic}"
        );
        emit(summary);
    }
}

agent Coordinator {
    on start {
        let r1 = spawn Researcher { topic: "quantum computing" };
        let r2 = spawn Researcher { topic: "CRISPR gene editing" };

        let s1 = await r1;
        let s2 = await r2;

        print(s1);
        print(s2);
        emit(0);
    }
}

run Coordinator;
```

## Why Sage?

**Agents as primitives, not patterns.** Most agent frameworks are libraries that impose patterns on top of a general-purpose language. Sage makes agents a first-class concept — the compiler understands what an agent is, what beliefs it holds, and how agents communicate.

**Type-safe LLM integration.** The `infer` expression lets you call LLMs with structured output. The type system ensures you handle inference results correctly.

**Compiles to native binaries.** Sage compiles to Rust, then to native code. Your agent programs are fast, self-contained binaries with no runtime dependencies.

**Concurrent by default.** Spawned agents run concurrently. The runtime handles scheduling and message passing.

## What You'll Learn

This guide covers:

1. **Getting Started** — Install Sage and write your first program
2. **Language Guide** — Syntax, types, and control flow
3. **Agents** — Beliefs, handlers, spawning, and messaging
4. **LLM Integration** — Using `infer` to call language models
5. **Reference** — CLI commands, environment variables, error codes

Let's get started with [installation](./getting-started/installation.md).
