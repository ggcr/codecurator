#!/bin/bash

set -e
if [ -z "$1" ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

DIR="$1"
total=0
for file in "$DIR"/*.jsonl; do
    file_count=$(jq -s '[.[] | .tokens] | add' "$file")
    total=$((total + file_count))
done

echo "Total tokens: $total"
