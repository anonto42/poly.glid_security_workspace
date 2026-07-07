#!/bin/bash
# Validate workspace structure

echo "🔍 Validating workspace..."

errors=0
warnings=0

# Check required directories
for dir in projects docs; do
    if [ ! -d "$dir" ]; then
        echo "  ❌ Missing directory: $dir"
        ((errors++))
    else
        echo "  ✅ $dir exists"
    fi
done

# Check required files
for file in Makefile workspace.toml; do
    if [ ! -f "$file" ]; then
        echo "  ❌ Missing file: $file"
        ((errors++))
    else
        echo "  ✅ $file exists"
    fi
done

if [ $errors -gt 0 ]; then
    echo "❌ Workspace validation failed with $errors errors."
    exit 1
else
    echo "✅ Workspace validation succeeded!"
    exit 0
fi
