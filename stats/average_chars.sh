#!/bin/bash

set -e
if [ -z "$1" ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

DIR="$1"
total=0
N=0
for file in "$DIR"/*.jsonl; do
    char_count=$(jq -s '[.[] | .text] | length' "$file")
    total=$((total + char_count))
    N=$((N+1))
done

echo "Average chars: $((total / N))"

