#!/usr/bin/env bash
# Compares two builds on the benchmark suite, interleaving their runs.
#
# Wall-clock timings on a developer machine drift over minutes: a build measured
# now and another measured five minutes later can differ by more than the change
# between them. Running them alternately, in the same conditions, is what makes
# a difference readable.
#
#     cargo build --release && cp target/release/valo /tmp/before
#     ...make a change...
#     cargo build --release && cp target/release/valo /tmp/after
#     ./scripts/bench-compare.sh /tmp/before /tmp/after
#
# Each side also reports how much it varied against itself, from its fastest run
# to its median one. A change smaller than that has not been measured, whatever
# the sign says, and the script prints that rather than leaving it to be read
# off the numbers.
set -euo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: bench-compare.sh <before-binary> <after-binary> [benchmark...]" >&2
    exit 2
fi

before="$1"
after="$2"
shift 2

for binary in "$before" "$after"; do
    if [ ! -x "$binary" ]; then
        echo "not executable: $binary" >&2
        exit 1
    fi
done

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repeats="${BENCH_REPEATS:-7}"

benchmarks=("$@")
if [ "${#benchmarks[@]}" -eq 0 ]; then
    benchmarks=("$root"/bench/*.valo)
fi

run_once() {
    local binary="$1" benchmark="$2" start end
    start=$(date +%s%N)
    "$binary" run "$benchmark" > /dev/null 2>&1
    end=$(date +%s%N)
    echo $(( (end - start) / 1000000 ))
}

printf '%-12s %10s %10s %9s %8s\n' benchmark before after change noise
for benchmark in "${benchmarks[@]}"; do
    name="$(basename "$benchmark" .valo)"
    before_times=()
    after_times=()

    for _ in $(seq "$repeats"); do
        # Alternating means a machine that gets busier partway through slows
        # both sides, rather than whichever happened to run second.
        before_times+=("$(run_once "$before" "$benchmark")")
        after_times+=("$(run_once "$after" "$benchmark")")
    done

    before_sorted=($(printf '%s\n' "${before_times[@]}" | sort -n))
    after_sorted=($(printf '%s\n' "${after_times[@]}" | sort -n))
    middle=$(( repeats / 2 ))

    before_best="${before_sorted[0]}"
    after_best="${after_sorted[0]}"
    # Fastest against median, rather than against slowest: the slowest run is
    # nearly always the first one, paying for a cold cache, and letting it set
    # the noise floor would hide every real change behind it.
    before_noise=$(( (before_sorted[middle] - before_best) * 100 / before_best ))
    after_noise=$(( (after_sorted[middle] - after_best) * 100 / after_best ))
    noise=$(( before_noise > after_noise ? before_noise : after_noise ))

    change=$(( (after_best - before_best) * 100 / before_best ))
    verdict=""
    if [ "${change#-}" -le "$noise" ]; then
        verdict="  not measured"
    fi

    printf '%-12s %8s ms %8s ms %+8s%% %+7s%%%s\n' \
        "$name" "$before_best" "$after_best" "$change" "$noise" "$verdict"
done

echo
echo "Best of $repeats runs each, alternating. Noise is how much the same build"
echo "varied from its fastest run to its median one. A change smaller than that"
echo "has not been measured."
