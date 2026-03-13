# Patterns

Common patterns for building LLM-powered agents.

## Parallel Research

Spawn multiple researchers, combine results:

```sage
agent Researcher {
    belief topic: String

    on start {
        let result: Inferred<String> = infer(
            "Research and provide 3 key facts about: {self.topic}"
        );
        emit(result);
    }
}

agent Synthesizer {
    belief findings: List<String>

    on start {
        let combined = join(self.findings, "\n\n");
        let synthesis: Inferred<String> = infer(
            "Given these research findings:\n{combined}\n\n" ++
            "Provide a unified summary highlighting connections."
        );
        emit(synthesis);
    }
}

agent Coordinator {
    on start {
        // Parallel research
        let r1 = spawn Researcher { topic: "quantum computing" };
        let r2 = spawn Researcher { topic: "machine learning" };
        let r3 = spawn Researcher { topic: "cryptography" };

        let f1 = await r1;
        let f2 = await r2;
        let f3 = await r3;

        // Synthesis
        let s = spawn Synthesizer {
            findings: [f1, f2, f3]
        };
        let result = await s;

        print(result);
        emit(0);
    }
}

run Coordinator;
```

## Chain of Thought

Break complex reasoning into steps:

```sage
agent ChainOfThought {
    belief question: String

    on start {
        let understand: Inferred<String> = infer(
            "Question: {self.question}\n\n" ++
            "First, restate the question in your own words and identify what's being asked."
        );

        let analyze: Inferred<String> = infer(
            "Question: {self.question}\n\n" ++
            "Understanding: {understand}\n\n" ++
            "Now, list the key concepts and relationships involved."
        );

        let solve: Inferred<String> = infer(
            "Question: {self.question}\n\n" ++
            "Understanding: {understand}\n\n" ++
            "Analysis: {analyze}\n\n" ++
            "Now, provide a step-by-step solution."
        );

        let answer: Inferred<String> = infer(
            "Question: {self.question}\n\n" ++
            "Solution: {solve}\n\n" ++
            "State the final answer concisely."
        );

        emit(answer);
    }
}
```

## Validation Loop

Have agents check each other's work:

```sage
agent Generator {
    belief task: String

    on start {
        let result: Inferred<String> = infer(
            "Complete this task: {self.task}"
        );
        emit(result);
    }
}

agent Validator {
    belief task: String
    belief result: String

    on start {
        let valid: Inferred<String> = infer(
            "Task: {self.task}\n\n" ++
            "Result: {self.result}\n\n" ++
            "Is this result correct and complete? " ++
            "Answer YES or NO, then explain briefly."
        );
        emit(valid);
    }
}

agent Main {
    on start {
        let task = "Write a haiku about programming";

        let gen = spawn Generator { task: task };
        let result = await gen;

        let val = spawn Validator { task: task, result: result };
        let validation = await val;

        print("Result: " ++ result);
        print("Validation: " ++ validation);
        emit(0);
    }
}

run Main;
```

## Map-Reduce

Process items in parallel, combine results:

```sage
agent Processor {
    belief item: String

    on start {
        let result: Inferred<String> = infer(
            "Process this item and extract key information: {self.item}"
        );
        emit(result);
    }
}

agent Reducer {
    belief items: List<String>

    on start {
        let combined = join(self.items, "\n---\n");
        let result: Inferred<String> = infer(
            "Combine these processed items into a summary:\n{combined}"
        );
        emit(result);
    }
}

agent MapReduce {
    on start {
        let items = ["doc1 content", "doc2 content", "doc3 content"];

        // Map phase - process in parallel
        let p1 = spawn Processor { item: "doc1 content" };
        let p2 = spawn Processor { item: "doc2 content" };
        let p3 = spawn Processor { item: "doc3 content" };

        let r1 = await p1;
        let r2 = await p2;
        let r3 = await p3;

        // Reduce phase
        let reducer = spawn Reducer { items: [r1, r2, r3] };
        let final_result = await reducer;

        print(final_result);
        emit(0);
    }
}

run MapReduce;
```

## Debate

Multiple agents argue different positions:

```sage
agent Debater {
    belief position: String
    belief topic: String

    on start {
        let argument: Inferred<String> = infer(
            "You are arguing {self.position} on the topic: {self.topic}\n\n" ++
            "Make your best argument in 2-3 sentences."
        );
        emit(argument);
    }
}

agent Judge {
    belief topic: String
    belief arg_for: String
    belief arg_against: String

    on start {
        let verdict: Inferred<String> = infer(
            "Topic: {self.topic}\n\n" ++
            "Argument FOR:\n{self.arg_for}\n\n" ++
            "Argument AGAINST:\n{self.arg_against}\n\n" ++
            "Which argument is stronger and why? Be brief."
        );
        emit(verdict);
    }
}

agent Main {
    on start {
        let topic = "AI will create more jobs than it destroys";

        let d1 = spawn Debater { position: "FOR", topic: topic };
        let d2 = spawn Debater { position: "AGAINST", topic: topic };

        let arg_for = await d1;
        let arg_against = await d2;

        let judge = spawn Judge {
            topic: topic,
            arg_for: arg_for,
            arg_against: arg_against
        };
        let verdict = await judge;

        print("FOR: " ++ arg_for);
        print("AGAINST: " ++ arg_against);
        print("VERDICT: " ++ verdict);
        emit(0);
    }
}

run Main;
```
