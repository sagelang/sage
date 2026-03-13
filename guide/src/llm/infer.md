# The infer Expression

The `infer` expression is how Sage programs interact with large language models.

## Basic Usage

```sage
let result: Inferred<String> = infer("What is the capital of France?");
print(result);  // "Paris" (or similar)
```

## String Interpolation

Use `{identifier}` to include variables in prompts:

```sage
agent Researcher {
    belief topic: String

    on start {
        let summary: Inferred<String> = infer(
            "Write a 2-sentence summary of: {self.topic}"
        );
        emit(summary);
    }
}
```

Multiple interpolations:

```sage
let format = "JSON";
let topic = "climate change";

let result: Inferred<String> = infer(
    "Output a {format} object with key facts about {topic}"
);
```

## The Inferred\<T\> Type

`infer` returns `Inferred<T>`, which wraps the LLM's response:

```sage
let response: Inferred<String> = infer("Hello!");
```

`Inferred<T>` coerces to `T` automatically:

```sage
let response: Inferred<String> = infer("Hello!");
print(response);  // Works - Inferred<String> coerces to String
```

## Structured Output

`infer` can return any type, including user-defined records:

```sage
record Summary {
    title: String,
    key_points: List<String>,
    sentiment: String,
}

agent Analyzer {
    topic: String

    on start {
        let result: Inferred<Summary> = infer(
            "Analyze this topic and provide a structured summary: {self.topic}"
        );
        print("Title: " ++ result.title);
        print("Sentiment: " ++ result.sentiment);
        emit(result);
    }
}
```

The runtime automatically:
1. Injects the expected schema into the prompt
2. Parses the LLM's response as JSON
3. Retries with error feedback if parsing fails (configurable via `SAGE_INFER_RETRIES`)

This works with any OpenAI-compatible API, including Ollama.

## Error Handling

If the LLM call fails (network error, API error), the program will panic. Future versions will support error handling.

## Example: Multi-Step Reasoning

```sage
agent Reasoner {
    belief question: String

    on start {
        let step1: Inferred<String> = infer(
            "Break down this question into sub-questions: {self.question}"
        );

        let step2: Inferred<String> = infer(
            "Given these sub-questions: {step1}\n\nAnswer each one briefly."
        );

        let step3: Inferred<String> = infer(
            "Given the original question: {self.question}\n\n" ++
            "And these answers: {step2}\n\n" ++
            "Provide a final comprehensive answer."
        );

        emit(step3);
    }
}

agent Main {
    on start {
        let r = spawn Reasoner {
            question: "How do vaccines work and why are they important?"
        };
        let answer = await r;
        print(answer);
        emit(0);
    }
}

run Main;
```

## Concurrent Inference

Multiple `infer` calls can run concurrently via spawned agents:

```sage
agent Summarizer {
    belief text: String

    on start {
        let summary: Inferred<String> = infer(
            "Summarize in one sentence: {self.text}"
        );
        emit(summary);
    }
}

agent Main {
    on start {
        let s1 = spawn Summarizer { text: "Long article about AI..." };
        let s2 = spawn Summarizer { text: "Long article about robotics..." };
        let s3 = spawn Summarizer { text: "Long article about space..." };

        // All three LLM calls happen concurrently
        let r1 = await s1;
        let r2 = await s2;
        let r3 = await s3;

        print(r1);
        print(r2);
        print(r3);
        emit(0);
    }
}

run Main;
```
