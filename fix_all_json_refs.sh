#!/bin/bash

echo "Fixing ALL JSON references to Array Tuple Syntax..."

# Replace hashbrown-json with array-tuples in all files
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/hashbrown-json/array-tuples/g' {} \;

# Replace json_syntax with array_tuple_syntax in all files
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/json_syntax/array_tuple_syntax/g' {} \;

# Replace "JSON object syntax" with "Array tuple syntax"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/JSON object syntax/Array tuple syntax/g' {} \;

# Replace "JSON Syntax" with "Array Tuple Syntax"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/JSON Syntax/Array Tuple Syntax/g' {} \;

# Replace "JSON syntax" with "Array tuple syntax"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/JSON syntax/Array tuple syntax/g' {} \;

# Replace "JSON-like syntax" with "Array tuple syntax"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/JSON-like syntax/Array tuple syntax/g' {} \;

# Replace "JSON object" with "Array tuple"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/JSON object/Array tuple/g' {} \;

# Replace "🔥 Amazing hashbrown HashMap macros with full JSON object support" with array tuple version
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/🔥 Amazing hashbrown HashMap macros with full JSON object support/🔥 Amazing hashbrown HashMap macros with array tuple syntax support/g' {} \;

# Replace "Hashbrown JSON Syntax" with "Hashbrown Array Tuple Syntax"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/Hashbrown JSON Syntax/Hashbrown Array Tuple Syntax/g' {} \;

# Replace any remaining "JSON" references that should be "Array Tuple"
find . -type f \( -name "*.toml" -o -name "*.rs" -o -name "*.md" \) -not -path "./.history/*" -not -path "./.git/*" -exec sed -i '' 's/JSON macros/Array tuple macros/g' {} \;

# Fix profile names in nextest.toml
sed -i '' 's/\[profile\.hashbrown-json\]/[profile.array-tuples]/g' ./.config/nextest.toml

echo "All JSON references have been updated to Array Tuple Syntax!"