# Control Flow

Sage provides standard control flow constructs.

## If/Else

```sage
if x > 0 {
    print("positive");
} else if x < 0 {
    print("negative");
} else {
    print("zero");
}
```

Conditions must be `Bool` — no implicit truthy/falsy coercion.

## For Loops

Iterate over lists:

```sage
let numbers = [1, 2, 3, 4, 5];

for n in numbers {
    print(str(n));
}
```

With index tracking:

```sage
let names = ["Alice", "Bob", "Charlie"];
let i = 0;

for name in names {
    print(str(i) ++ ": " ++ name);
    i = i + 1;
}
```

## While Loops

```sage
let count = 0;

while count < 5 {
    print(str(count));
    count = count + 1;
}
```

## Early Return

Use `return` to exit a function early:

```sage
fn find_first_positive(numbers: List<Int>) -> Int {
    for n in numbers {
        if n > 0 {
            return n;
        }
    }
    return -1;
}
```

## Example: FizzBuzz

```sage
fn fizzbuzz(n: Int) -> String {
    if n % 15 == 0 {
        return "FizzBuzz";
    }
    if n % 3 == 0 {
        return "Fizz";
    }
    if n % 5 == 0 {
        return "Buzz";
    }
    return str(n);
}

agent Main {
    on start {
        let i = 1;
        while i <= 20 {
            print(fizzbuzz(i));
            i = i + 1;
        }
        emit(0);
    }
}

run Main;
```
