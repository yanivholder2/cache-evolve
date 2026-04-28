# Conventions

## Module

Use `parent/mod.rs` style: the module root lives inside its directory as
`mod.rs`, not as a `parent.rs` file alongside the `parent/` directory.

## Logging

- Format: `StructName.function_name: ...` for struct methods,
  `function_name: ...` for free functions. JNI exports use the Java method
  name (e.g. `getFromCacheIfPresent:`, `associativeInitCache:`).
- Prefer inline format variables `{var}` over positional `{}, var` in format
  strings whenever the argument is a simple variable name. Use positional for
  expressions/field accesses.

## Variable naming

When unwrapping an `Option`/`Result` (or any narrowing) where the inner value
means the same thing as the outer, shadow the name instead of inventing a new
one: `if let Some(evicted) = evicted { ... }`, not
`if let Some(lost) = evicted { ... }`. Only rename when the unwrapped form is
semantically distinct (e.g. `raw_bytes` → `parsed`).
