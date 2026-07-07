#!/bin/bash
# Generate dependency graph

echo "📊 Generating dependency graph..."

# Read workspace.toml
deps=$(grep -A 10 "\[dependencies\]" workspace.toml 2>/dev/null | grep -v "\[dependencies\]" || echo "")

echo "digraph G {"
echo "  rankdir=LR;"
echo "  node [shape=box, style=filled, fillcolor=lightblue];"

while IFS= read -r line; do
    if [[ $line =~ \"(.*)\"\ =\ \[(.*)\] ]]; then
        project="${BASH_REMATCH[1]}"
        deps_list="${BASH_REMATCH[2]}"
        for dep in $deps_list; do
            dep="${dep%\",*}"
            dep="${dep#\"}"
            echo "  \"$project\" -> \"$dep\";"
        done
    fi
done <<< "$deps"

echo "}"
