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
single element write was O(n) and filling an array was O(n²), which came to over
three minutes to fill and read 100k elements.

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

All three former copies of this logic, in the preprocessor, the semantic
analyzer, and the interpreter, now delegate to
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

## Strings are shared, not copied

`Value::String` holds an `Rc<String>`. A `Value` is copied constantly, into a
variable, an argument, or an array element, and copying the text each time was
the largest cost of moving a string around. Copying one is now a refcount bump:
four million copies of a 62-character string went from 1460ms to 999ms.

`Rc<str>` was tried first, and was worse. It is eight bytes smaller, but
building one from a `String` copies the bytes again, and building is as common
as copying, since every concatenation and every `Format` produces a `String`
first. It cost 15% on concatenation to save a further 15% on copies.

Both were measured the same way, and the first attempt at measuring them was
wrong: short runs alternating between two builds showed the change as 8% *slower*
before longer runs showed it 32% faster. Process startup is 58ms here, so a
benchmark has to run for seconds before the difference it is measuring is larger
than the noise around it.

## Calling a procedure does not copy it

A procedure holds its body. Calling one used to copy that body, so the cost of a
call was proportional to how much code the procedure contained. 300,000 calls
to a function with 120 statements that never ran took 28.9 seconds, against 1.4
for an empty one. Procedures, methods, and constructors are now shared
(`Rc<Procedure>`, `Rc<Function>`), and the same benchmark takes 1.6 seconds:
body size costs a call essentially nothing.

A generic procedure is still copied, because substituting its type arguments
rewrites it. That happens per call site, not per procedure, and only for the
ones that are generic.

Overload resolution now works out the argument types only when there is more
than one candidate. Building that list walks the arguments and folds their
names, which for a name with one procedure behind it was work with nothing to
decide: doing it lazily took 18% off a benchmark that passes a string to a
function 600,000 times.

## Assigning to a local took five lookups

Reading a variable cost about 38ns and assigning to one cost 185ns, five times
as much for what should be less work. The ratio was the clue: assignment did
five hash lookups where a read does one.

It checked for a return slot, then for a field of the enclosing instance, then
whether a local by that name existed, and then looked the local up twice more
inside the write itself. Every one of those folds the name and hashes it.

Now it is one. A frame knows whether `Me` is bound without looking, so outside
a method the instance-field check is free; the write does its own single lookup
and hands the value back when there is no such local, so the caller can try
elsewhere without having cloned it.

| | before | after |
|---|---|---|
| `a = 1`, five million times | 1415ms | 902ms |
| `a = b + c`, five million times | 2271ms | 1632ms |
| an empty `For` loop, five million times | 377ms | 294ms |
| twenty million reads and writes | 21.4s | 15.7s |

The empty loop improved because a `For` assigns its counter through the same
path, which is a fair summary of how much of a program this is.

## The hash is not what a variable read costs

Rust's default hasher is SipHash, chosen to resist an attacker who controls the
keys. Nothing in the interpreter is in that position: the keys are identifiers
out of the program being run. Swapping it for the multiply-and-rotate hash that
rustc uses is the obvious move, and it made variable reads 6% *slower*: 20.9
seconds against 22.0 for twenty million reads, the same way round on five
consecutive rounds.

Short names are why. SipHash has a tight path for a handful of bytes, and a
hand-written replacement pays a separate round for the tail and another for the
length, which is more work rather than less at that size.

The useful part is what it says about where the time goes. If replacing the
hash function changes so little, hashing is not what a variable read costs. The
lookup is, and the way to remove a lookup is to know the slot: that is what
[resolving identifiers to slot indices](roadmap.md) is for.

## What could not be measured

A call formats two strings for the stack trace and the scope name, on every
call, whether or not anything reads them. Removing both and measuring showed
the build with *less* work in it running slower, on repeated interleaved runs.
That is not a result about the change; it is the noise floor of the machine
doing the measuring, which on the day was wider than the effect.

Recorded because the alternative is to keep rediscovering it. An optimisation
worth roughly one allocation per call needs a quieter machine or a harness that
measures the interpreter rather than the process, and until then it is not worth
the change it would take to land.
