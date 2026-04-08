# File Embedding

One of Novel's key features is the ability to embed external source files directly into your documentation.

## Basic Usage

Use the `file` attribute in a code fence to embed a file:

````markdown
```rust file="./path/to/file.rs"
```
````

The code block body should be empty — the content will be loaded from the specified file.

## Path Resolution

### Relative Paths

Paths starting with `./` or `../` are resolved relative to the current markdown file:

````markdown
```rust file="./examples/hello.rs"
```
````

### Root-Relative Paths

Use `<root>/` prefix to reference files from the project root:

````markdown
```rust file="<root>/src/main.rs"
```
````

## Line Ranges

You can embed only specific lines from a file:

````markdown
```rust file="./examples/hello.rs#L3-L8"
```
````

Supported formats:
- `#L5` — single line
- `#L5-L10` — line range (inclusive)
- `#5-10` — line range without `L` prefix

## Live Example

Here is the Novel configuration file for this documentation site:

```toml file="<root>/novel.toml"
```

## Use Cases

::: tip When to use file embedding
- **Tested code**: Embed code that is actually compiled and tested, ensuring docs stay in sync
- **Configuration examples**: Show real configuration files from your project
- **API examples**: Include example code that lives alongside your source
:::

::: warning
Make sure the embedded files exist at build time. Missing files will produce a warning and show an error message in the code block.
:::
