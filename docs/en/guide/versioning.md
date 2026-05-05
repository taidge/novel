# Versioning

Novel can build frozen documentation versions from subdirectories under your docs root.

## Directory Layout

```text
docs/
  next/
    index.md
    guide/getting-started.md
  v1/
    index.md
    guide/getting-started.md
```

## Configuration

```toml title="novel.toml"
[versions]
current = "next"

[[versions.items]]
code = "next"
label = "Next"
dir = "next"

[[versions.items]]
code = "v1"
label = "1.0"
dir = "v1"
```

The `current` version keeps the normal, unprefixed routes. In the example above, `docs/next/guide/getting-started.md` builds to `/guide/getting-started`, while `docs/v1/guide/getting-started.md` builds to `/v1/guide/getting-started`.

You can override the route prefix for any version:

```toml
[[versions.items]]
code = "v1"
label = "1.0"
dir = "v1"
path = "/docs/1.0"
```

## Version Selector

When the same relative file exists in multiple versions, Novel adds a version selector to the navbar. Selecting another version navigates to that version's equivalent page.

## Links

For archived versions, absolute internal Markdown links are rewritten into the version prefix. For example, `/guide/configuration` inside the `v1` source becomes `/v1/guide/configuration`.
