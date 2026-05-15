#!/usr/bin/env bash
set -euo pipefail

# Execute from inside the Coverage folder. All paths relative to Coverage.
# Usage: cd crates/envs/ce-security/src/Coverage && ./run_coverage.sh [options]

COVERAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$COVERAGE_DIR/results"
WORKSPACE_ROOT="$(cd "$COVERAGE_DIR/../../../../.." && pwd)"

# ---- Defaults ----
ITERATIONS=200
MODE="new"
OUTPUT_DIR="$RESULTS_DIR"
SEED=0

usage() {
    echo "Usage: $0 [--iterations N] [--mode new|old|compare] [--output-dir DIR] [--seed N]"
    echo "  Run from inside the Coverage folder: cd .../ce-security/src/Coverage && ./run_coverage.sh"
    echo "  Results go to Coverage/results/<timestamp>_<mode>."
    echo "  --seed 0 (default) uses a timestamp-based seed; any other value is reproducible."
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --mode)
            MODE="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --seed)
            SEED="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown argument: $1"
            usage
            ;;
    esac
done

cd "$WORKSPACE_ROOT"

echo "Building coverage_runner..."
cargo build -p ce-security --release
echo "Build succeeded."

BINARY="$WORKSPACE_ROOT/target/release/coverage_runner"

echo "Running coverage_runner in ${MODE} mode (seed=${SEED}) ..."
"$BINARY" \
    --iterations "$ITERATIONS" \
    --mode "$MODE" \
    --output-dir "$OUTPUT_DIR" \
    --seed "$SEED"

echo ""
echo "Results saved to: $OUTPUT_DIR"
