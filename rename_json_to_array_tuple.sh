#!/bin/bash

# Script to rename all JSON references to Array Tuple Syntax

echo "Starting comprehensive JSON to Array Tuple rename..."

# Replace in all .toml files
find . -name "*.toml" -not -path "./.history/*" -exec sed -i '' 's/hashbrown-json/array-tuples/g' {} \;
find . -name "*.toml" -not -path "./.history/*" -exec sed -i '' 's/"json"/"array-tuples"/g' {} \;

# Replace in all .rs files
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/hashbrown-json/array-tuples/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/json_syntax/array_tuple_syntax/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/json_builder/array_tuple_builder/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/json_ext/array_tuple_ext/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/JsonObject/ArrayTupleObject/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/JsonHashMap/ArrayTupleHashMap/g' {} \;

# Replace in all .md files (excluding history)
find . -name "*.md" -not -path "./.history/*" -exec sed -i '' 's/hashbrown-json/array-tuples/g' {} \;
find . -name "*.md" -not -path "./.history/*" -exec sed -i '' 's/json_syntax/array_tuple_syntax/g' {} \;
find . -name "*.md" -not -path "./.history/*" -exec sed -i '' 's/JSON Syntax/Array Tuple Syntax/g' {} \;
find . -name "*.md" -not -path "./.history/*" -exec sed -i '' 's/JSON object syntax/Array tuple syntax/g' {} \;
find . -name "*.md" -not -path "./.history/*" -exec sed -i '' 's/JSON-like syntax/Array tuple syntax/g' {} \;

# Rename source files
if [ -f "packages/macros/src/json_syntax.rs" ]; then
    mv packages/macros/src/json_syntax.rs packages/macros/src/array_tuple_syntax.rs
fi

if [ -f "packages/macros/src/json_builder.rs" ]; then
    mv packages/macros/src/json_builder.rs packages/macros/src/array_tuple_builder.rs
fi

if [ -f "packages/collections/src/json_ext.rs" ]; then
    mv packages/collections/src/json_ext.rs packages/collections/src/array_tuple_ext.rs
fi

if [ -f "packages/llm/src/json_syntax.rs" ]; then
    mv packages/llm/src/json_syntax.rs packages/llm/src/array_tuple_syntax.rs
fi

# Update module declarations in lib.rs files
find . -name "lib.rs" -not -path "./.history/*" -exec sed -i '' 's/mod json_syntax;/mod array_tuple_syntax;/g' {} \;
find . -name "lib.rs" -not -path "./.history/*" -exec sed -i '' 's/mod json_builder;/mod array_tuple_builder;/g' {} \;
find . -name "lib.rs" -not -path "./.history/*" -exec sed -i '' 's/mod json_ext;/mod array_tuple_ext;/g' {} \;

# Update use statements
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/use.*json_syntax/use crate::array_tuple_syntax/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/use.*json_builder/use crate::array_tuple_builder/g' {} \;
find . -name "*.rs" -not -path "./.history/*" -exec sed -i '' 's/use.*json_ext/use crate::array_tuple_ext/g' {} \;

echo "Rename complete! Remember to update any remaining manual references."