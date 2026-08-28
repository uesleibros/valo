# Performance

Valo executes programs with a tree-walking interpreter. That design is simple and
easy to extend, but it makes the cost of every statement visible: whatever the
interpreter does per node, it does millions of times. Performance work here is
mostly about removing work from those paths, not about clever optimizations.

## Measuring

The benchmark suite lives in [`bench/`](../../bench/README.md). Always measure a
release build; a debug build is dominated by unoptimized interpreter overhead and
tells you nothing useful.

```sh
cargo build --release -p valo_cli
./scripts/bench.sh
```

Each benchmark targets one part of the interpreter:

| Benchmark | Exercises |
|---|---|
| `loop.valo` | Counting loops, integer arithmetic, scalar assignment |
| `arrays.valo` | Array element reads and writes |
| `strings.valo` | String concatenation and builtin string calls |
| `calls.valo` | Recursive calls, frame setup, parameter binding |
| `objects.valo` | Object allocation, field access, method dispatch |

`scripts/bench.sh` reports the best of several runs, since wall-clock timings on
a developer machine are noisy. Treat a change under roughly 10% as noise unless
it reproduces.

## The release profile

The workspace sets `lto = "fat"` and `codegen-units = 1` for release builds. The
interpreter's dispatch loop spans several modules, so cross-crate inlining
matters more here than build time does.

`panic = "abort"` is *not* set: the FFI layer relies on `catch_unwind` to turn a
panic inside a native callback into a diagnostic rather than a crash.

## What made the interpreter slow

Three patterns accounted for most of the cost, and they are worth recognizing
because they recur.

### Cloning a whole value to inspect it

Indexed assignment (`data(i) = x`) needs to know whether the target is an array
or an object with a default property. It answered that by cloning the variable's
value and matching on it. Cloning `Value::Array` deep-copies every element, so a
single element write was O(n) and filling an array was O(n²) — over three minutes
to fill and read 100k elements.

The fix is to inspect the value in place and clone only the cases where a clone
is cheap. `Value::Object` and `Value::ComObject` are reference-counted, so
cloning them is a refcount bump; everything else is answered without a clone.

The same pattern appeared in assignment, which cloned the previous value so the
caller could run deterministic termination on it. Only objects have a
`Class_Terminate` hook, so only objects need to come back.

### Allocating to look up a name

Basic is case-insensitive, so every symbol table is keyed by a case-folded name,
and every variable read folded its identifier into a fresh `String`. Identifiers
are short and ASCII in practice, so `runtime::with_folded` folds into a stack
buffer and hands out a `&str`, falling back to allocation only for long or
non-ASCII names.

All three former copies of this logic — in the preprocessor, the semantic
analyzer, and the interpreter — now delegate to
[`runtime::naming`](../../core/src/runtime/naming.rs), so the three layers cannot
drift on what "the same name" means.

### Doing cold work on a hot path

Executing a block built a label index and a line-number index up front. Both are
only consulted by `GoTo`, `Resume`, and error handling, and a block is re-entered
on every loop iteration. They are now built lazily, so only the cold paths pay.

Assignment had a similar problem: it formatted a `__return_<name>` key on every
statement to check whether the assignment targeted the enclosing function's
result. Most frames hold no return slot at all, so that check now short-circuits
before building the key.

## Results

| Benchmark | Before | After | Speedup |
|---|---|---|---|
| Array fill and read (100k elements) | 195 s | 0.72 s | 270x |
| Nested arithmetic loop (2M iterations) | 4.74 s | 1.16 s | 4.1x |
| String building (40k iterations) | 0.25 s | 0.07 s | 3.6x |

Both columns were measured on the same machine, back to back, comparing the
commit before this work against the current build.

## What comes next

The tree-walking interpreter still re-resolves names, re-checks statement shapes,
and rebuilds frames on every execution. A bytecode VM addresses all three by
moving that work to a compile step, and it is the next significant item on the
[roadmap](roadmap.md). The benchmarks exist so that work can be judged against
numbers rather than intuition.

Smaller wins that remain available in the current design:

- resolve identifiers to slot indices during validation, so the interpreter stops
  hashing names at run time
- make `Value::String` reference-counted, so passing strings around stops copying
  them
- cache method and property lookups per call site
