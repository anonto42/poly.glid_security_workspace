#!/bin/bash
# Detect changed projects between git refs

BASE=${1:-main}
HEAD=${2:-HEAD}

echo "🔍 Detecting changes between $BASE and $HEAD..."

# Get changed files
changed_files=$(git diff --name-only $BASE..$HEAD 2>/dev/null || echo "")

# Find affected projects
affected_projects=()
for file in $changed_files; do
    if [[ $file =~ projects/([^/]+)/([^/]+)/([^/]+) ]]; then
        project="${BASH_REMATCH[3]}"
        if [[ ! " ${affected_projects[@]} " =~ " ${project} " ]]; then
            affected_projects+=("$project")
        fi
    fi
done

echo "📦 Affected projects: ${affected_projects[@]}"
echo "${affected_projects[@]}"
