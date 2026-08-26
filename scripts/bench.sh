#!/usr/bin/env bash
# Runs the benchmark suite against a release build and prints wall-clock timings.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$root/target/release/valo"
[ -x "$binary" ] || binary="$root/target/release/valo.exe"

if [ ! -x "$binary" ]; then
    echo "release binary not found; run: cargo build --release -p valo_cli" >&2
    exit 1
fi

repeats="${BENCH_REPEATS:-3}"

for benchmark in "$root"/bench/*.valo; do
    name="$(basename "$benchmark" .valo)"
    best=""
    for _ in $(seq "$repeats"); do
        start=$(date +%s%N)
        "$binary" run "$benchmark" > /dev/null
        end=$(date +%s%N)
        elapsed=$(( (end - start) / 1000000 ))
        if [ -z "$best" ] || [ "$elapsed" -lt "$best" ]; then
            best="$elapsed"
        fi
    done
    printf '%-12s %6s ms\n' "$name" "$best"
done
