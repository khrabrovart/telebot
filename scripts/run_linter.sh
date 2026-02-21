#!/bin/bash

set -euo pipefail

SRC_DIR="$(git rev-parse --show-toplevel)/src"

for project in "$SRC_DIR"/*; do
    if [ -d "$project" ] && [ -f "$project/Cargo.toml" ]; then
        echo "Running cargo clippy in $project..."
        (cd "$project" && cargo clippy --all-targets --all-features -- -D warnings)
    fi
done

echo "All cargo clippy checks passed."
