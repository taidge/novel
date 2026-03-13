# Markdown Features

Sapid supports standard Markdown with GitHub Flavored Markdown (GFM) extensions.

## Headings

Use `#` through `######` for headings. Each heading gets an anchor link automatically.

## Emphasis

- **Bold** with `**text**`
- *Italic* with `*text*`
- ~~Strikethrough~~ with `~~text~~`

## Links

- [Internal link](/guide/getting-started)
- [External link](https://github.com) (opens in new tab automatically)

## Lists

Unordered:
- Item one
- Item two
  - Nested item

Ordered:
1. First
2. Second
3. Third

## Task Lists

- [x] Implemented markdown parsing
- [x] Added syntax highlighting
- [x] Added file embedding
- [ ] World domination

## Tables

| Feature | Description | Status |
|---------|-------------|--------|
| GFM | GitHub Flavored Markdown | Done |
| Highlighting | Syntax highlighting for code blocks | Done |
| Containers | Tip, warning, danger, info, note, details | Done |
| File Embed | Embed external files in code blocks | Done |
| Tabs | Tabbed content blocks | Done |
| Steps | Numbered step guides | Done |
| Badges | Inline badges | Done |

## Blockquotes

> Documentation is a love letter that you write to your future self.
> — Damian Conway

## Code Blocks

Inline code: `let x = 42;`

Fenced code block with syntax highlighting:

```rust title="example.rs"
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### Line Numbers

Add `showLineNumbers` to the code fence to display line numbers:

````markdown
```rust showLineNumbers
fn main() {
    println!("Hello!");
}
```
````

Or enable globally in `sapid.toml`:

```toml
[markdown]
show_line_numbers = true
```

### Line Highlighting

Highlight specific lines with `{1,3-5}`:

````markdown
```rust {1,4-5}
use std::io;

fn main() {
    println!("Hello!");
    println!("World!");
}
```
````

```rust {1,4-5}
use std::io;

fn main() {
    println!("Hello!");
    println!("World!");
}
```

### Diff Display

Use the `diff` language to show added and removed lines:

```diff
- let old_value = 1;
+ let new_value = 2;
  let unchanged = 3;
```

## Container Directives

::: tip
This is a helpful tip for your readers.
:::

::: warning
Pay attention to this warning.
:::

::: danger
This action is irreversible!
:::

::: info
Here is some additional information.
:::

::: note
A note for reference.
:::

::: details Click to expand
This content is hidden by default and can be expanded by clicking.

You can put any markdown content here, including:
- Lists
- **Bold text**
- Code: `let x = 1;`
:::

## Tabs

Group related content with tabs using `::: tabs` and `== Tab Title`:

::: tabs
== npm

```bash
npm install my-package
```

== yarn

```bash
yarn add my-package
```

== pnpm

```bash
pnpm add my-package
```
:::

Syntax:

````markdown
::: tabs
== First Tab

Content for the first tab.

== Second Tab

Content for the second tab.
:::
````

## Steps

Create numbered step-by-step guides with `::: steps`:

::: steps

### Install Rust

Download and install Rust from [rustup.rs](https://rustup.rs).

### Create a project

```bash
sapid init my-docs
```

### Start writing

Add `.md` files to the `docs/` directory.

:::

Syntax:

````markdown
::: steps

### First step

Description here.

### Second step

Description here.

:::
````

## Badges

Add inline badges with `{badge:TYPE|TEXT}`:

- This is {badge:tip|New} in v0.2
- This feature is {badge:warning|Experimental}
- This is {badge:danger|Deprecated}
- Status: {badge:info|Stable}

Syntax: `{badge:tip|Text}` — supported types: `tip`, `info`, `warning`, `danger`, `note`.

## Images

Images are lazy-loaded and support click-to-zoom:

```markdown
![Alt text](./image.png)
```

## Horizontal Rules

---

## HTML

Inline HTML is supported for cases where Markdown isn't enough.
