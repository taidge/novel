# AI Context Files

Novel generates AI-readable documentation context files during `novel build`.

## Generated Files

The CLI enables `LlmsTxtPlugin` and `MarkdownMirrorPlugin` by default and writes:

- `llms.txt` — a compact Markdown map of public documentation pages
- `llms-full.txt` — the public documentation content in one file
- `.well-known/llms.txt` — compatibility copy
- `.well-known/llms-full.txt` — compatibility copy
- one `.md` mirror per public page, such as `/guide/getting-started.md`

These files are useful when users add your documentation as context in AI coding tools, or when crawlers need a clean overview without parsing site chrome, JavaScript, and rendered HTML.

## Page Filtering

Novel excludes pages that should not be treated as public documentation:

- Pages with `noindex: true`
- Pages that redirect elsewhere
- The generated 404 page

Draft and future-dated pages are already filtered by the normal content pipeline unless you build with `--drafts` or `--future`.

## Full Context Source

For filesystem-backed builds, `llms-full.txt` prefers the original Markdown source and removes frontmatter before writing the page body. This preserves code fences and author-written Markdown. If source Markdown is unavailable, Novel falls back to plain text extracted from rendered HTML.

## Custom Files

You can override the generated files by placing your own `llms.txt`, `llms-full.txt`, or `.well-known/llms.txt` files in your docs directory. Static assets are copied after plugin output, so your custom files win.

## Markdown Mirrors

Markdown mirrors are enabled by default:

```toml
[markdown_mirror]
enabled = true
strip_frontmatter = true
```

Set `enabled = false` if you only want the aggregate `llms.txt` files. When enabled, doc pages also show a `Markdown` link in the footer.

## Library Usage

When using Novel as a library, register the plugin:

```rust
use novel_core::plugins::LlmsTxtPlugin;
use novel_core::plugins::MarkdownMirrorPlugin;

let site = novel_core::DirNovel::new("docs")
    .plugin(LlmsTxtPlugin)
    .plugin(MarkdownMirrorPlugin)
    .build()?;
```

You can also generate the strings directly from a built site:

```rust
let llms = site.llms_txt();
let full = site.llms_full_txt();
```
