# TODO - Array Tuple Syntax Implementation

## Milestone: Complete Array Tuple Syntax Implementation

The "consolation prize" approach uses standard Rust array tuple syntax `[("key", "value")]` instead of the complex brace syntax transformation.

### Implementation Approach

1. **Array Tuple Syntax**: Use `[("key", "value")]` syntax (standard Rust)
2. **From/Into Traits**: Implement `From<[(&str, &str); N]>` for `HashMap<K, V>`
3. **Builder Methods**: Accept generic parameters that implement `Into<HashMap<K, V>>`
4. **Zero Complexity**: No proc macros, no syntax transformation needed

### Current Status ✅

- [x] Examples updated to use array syntax `[("key", "value")]`
- [x] README updated to show array syntax
- [x] Documentation updated to reflect array approach
- [x] All syntax working with existing From/Into trait implementations

### Benefits

- **Simple**: Uses standard Rust syntax, no magic
- **Fast**: No compile-time transformation overhead  
- **Reliable**: No complex proc macro edge cases
- **Ergonomic**: Clean, readable syntax that Rust developers understand
- **Compatible**: Works with existing hashbrown HashMap infrastructure

### Examples Working

```rust
// All of these work with the array tuple syntax:
FluentAi::agent_role("researcher")
    .additional_params([("beta", "true"), ("debug", "false")])
    .metadata([("key", "val"), ("foo", "bar")])
    .tools((
        Tool::<Perplexity>::new([("citations", "true"), ("format", "json")]),
    ))
```

The array tuple approach provides a clean, simple solution that works immediately without complex proc macro infrastructure.