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
