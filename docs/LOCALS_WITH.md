`LocalsWith` are immutable, local-scoped value bindings without allocating
Scratch variables. Primarily used to preserve argument
semantics during function inlining.

This is an internal compiler extension, not part of Scratch.
 
Caveats:
- The body cannot yield or trigger a screen refresh.
- Only bindings in the innermost scope are accessible.

# Demonstration

```txt
def bar(a) {
    x = 5;
    print(a); // must print 4
}
def main() {
    x = 4;
    bar(x);
}
```

During optimization passes, Naive inlining produces incorrect behavior:

```txt
def main() {
    x = 4;
    x = 5;
    print(x); // Prints 5 instead of 4
}
```

With `LocalsWith`, the argument value is preserved:

```txt
def main() {
    x = 4;
    with a = x {
        x = 5;
        print(a); // Prints 4
    }
}
```
