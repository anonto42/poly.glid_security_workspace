#!/bin/bash
# Detect changed projects between git refs

BASE=${1:-main}
HEAD=${2:-HEAD}

echo "🔍 Detecting changes between $BASE and $HEAD..."

# Get changed files
changed_files=$(git diff --name-only $BASE..$HEAD 2>/dev/null || echo "")

# Find affected projects by walking up to the nearest dir with a build file
affected_projects=()
for file in $changed_files; do
    if [[ $file != projects/* ]]; then continue; fi
    dir=$(dirname "$file")
    while [ "$dir" != "." ] && [ "$dir" != "/" ]; do
        if [ -f "$dir/Cargo.toml" ] || [ -f "$dir/package.json" ] || [ -f "$dir/pyproject.toml" ] || [ -f "$dir/go.mod" ]; then
            project=$(basename "$dir")
            if [[ ! " ${affected_projects[@]} " =~ " ${project} " ]]; then
                affected_projects+=("$project")
            fi
            break
        fi
        dir=$(dirname "$dir")
    done
done

echo "📦 Affected projects: ${affected_projects[@]}"
echo "${affected_projects[@]}"
