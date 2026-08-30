# Valo Benchmarks

Small, self-contained programs that exercise the interpreter's hot paths. They
exist to catch performance regressions, not to compare Valo against other
languages.

| Benchmark | Exercises |
|---|---|
| [`loop.valo`](loop.valo) | Counting loops, integer arithmetic, scalar assignment |
| [`arrays.valo`](arrays.valo) | Array element reads and writes |
| [`strings.valo`](strings.valo) | String concatenation and builtin string calls |
| [`calls.valo`](calls.valo) | Recursive calls, frame setup, parameter binding |
| [`objects.valo`](objects.valo) | Object allocation, field access, method dispatch |

## Running

Always measure a release build; a debug build is dominated by unoptimized
interpreter overhead and tells you nothing useful.

```sh
cargo build --release -p valo_cli
./scripts/bench.sh
```

To run one benchmark directly:

```sh
./target/release/valo run bench/loop.valo
```

## Comparing two builds

A single number tells you nothing on its own. Timings on a developer machine
drift over minutes, so a build measured now against one measured five minutes
ago can differ by more than the change between them:

```sh
cargo build --release && cp target/release/valo /tmp/before
# make the change
cargo build --release && cp target/release/valo /tmp/after
./scripts/bench-compare.sh /tmp/before /tmp/after
```

It runs the two alternately and reports, for each benchmark, how much the same
build varied from its own fastest run to its median one. A change smaller than
that is marked `not measured`, because it has not been.

Each benchmark is sized to run for a few seconds. Shorter is tempting and
wrong: process startup alone is around 60ms, and a real 20% change has hidden
under the jitter of a run lasting a fraction of a second more than once.
