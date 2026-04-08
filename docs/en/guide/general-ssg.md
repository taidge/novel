# General SSG Mode

In addition to documentation sites, Novel can build blogs, marketing pages, portfolios, and other content sites. The general SSG features are **opt-in** and stack on top of the existing doc-site experience without breaking it.

## Collections

A collection is a top-level directory under `docs/` that contains a `_collection.toml` marker file. Pages inside it are grouped, sorted, paginated, and filterable.

```
docs/
├── guide/                 # existing doc section (untouched)
└── posts/
    ├── _collection.toml   # marker
    ├── hello-world.md
    ├── second-post.md
    └── third-post.md
```

`docs/posts/_collection.toml`:

```toml
layout = "blog"             # default layout for entries
list_layout = "list"        # layout for /posts/ index
sort_by = "date"            # date | weight | title
order = "desc"              # desc | asc
paginate_by = 10            # 0 disables pagination
publish = true
```

Output:

```
dist/posts/hello-world/index.html      # entry pages
dist/posts/index.html                  # paginated list (page 1)
dist/posts/page/2/index.html           # page 2
```

## Frontmatter for content pages

```yaml
---
title: Hello World
date: 2026-04-01
updated: 2026-04-07
draft: false
weight: 10
summary: Optional manual summary
tags: [novel, intro]
categories: [news]
series: novel-internals
authors: [chris]
expiry_date: 2027-01-01
---

Lead paragraph shown on the list page.

<!-- more -->

The rest of the post body.
```

| Field | Type | Description |
|---|---|---|
| `date` | `YYYY-MM-DD` | Publish date — used for sorting, archives, feeds |
| `updated` | `YYYY-MM-DD` | Last-updated date — exposed via OG `article:modified_time` |
| `draft` | bool | Excluded from build unless `--drafts` |
| `weight` | int | Sort key when `sort_by = "weight"` |
| `summary` | string | Manual summary (overrides `<!-- more -->` extraction) |
| `tags` | list | Taxonomy entries (see below) |
| `categories` | list | Taxonomy entries |
| `series` | string | Series identifier — generates `/series/<slug>/` |
| `authors` | list | Author names — exposed via OG `article:author` |
| `expiry_date` | `YYYY-MM-DD` | Page is excluded after this date unless `--future` |

## Drafts and future-dated content

```bash
novel build              # excludes drafts and future-dated pages
novel build --drafts     # include draft: true pages
novel build --future     # include pages with date > today
```

The same flags work via config:

```toml
[content]
drafts = false
future = false
summary_separator = "<!-- more -->"
```

## Summaries

If a markdown file contains `<!-- more -->`, everything before it becomes the page's `summary_html`, used by:

- list pages (`/posts/`)
- term pages (`/tags/<term>/`)
- per-collection feeds
- JSON Feed entries

`frontmatter.summary` overrides the separator.

## Taxonomies

Configure in `novel.toml`:

```toml
[taxonomies.tags]
name = "Tags"

[taxonomies.categories]
name = "Categories"
permalink = "/cat/{slug}/"   # optional, default /<key>/<slug>/
paginate_by = 10              # optional, no pagination by default
```

For each taxonomy Novel emits:

```
dist/tags/index.html           # overview (term cloud)
dist/tags/<term>/index.html    # individual term, with pagination
```

## Pagination

```toml
[pagination]
page_path = "page"             # /posts/page/2/
first_page_in_root = true      # /posts/ instead of /posts/page/1/
```

The same paginator powers collection list pages and taxonomy term pages.

## Series

Any page with `series: <name>` joins a virtual series index:

```
dist/series/<slug>/index.html
```

Entries are sorted **ascending by date** so the reading order makes sense.

## Date archives

Every page that has `date` is grouped automatically:

```
dist/archive/2026/index.html         # year archive
dist/archive/2026/04/index.html      # month archive
```

## Data files

Drop `.toml` or `.json` files under `docs/data/` and they become available to every template as `site_data`:

```
docs/data/
├── authors.toml
└── nav/links.json
```

```toml
# docs/data/authors.toml
[chris]
name = "Chris"
url = "https://example.com/chris"
```

In a template:

```jinja
<a href="{{ site_data.authors.chris.url }}">{{ site_data.authors.chris.name }}</a>
```

Subdirectories nest deeper into the tree, so `data/nav/links.json` is reachable via `site_data.nav.links`.

## Feeds

`FeedPlugin` (enabled by default in the CLI) emits, when `site.site_url` is set:

| File | Format |
|---|---|
| `feed.xml` | Site-wide Atom 1.0 |
| `feed.json` | Site-wide JSON Feed v1.1 |
| `<collection>/feed.xml` | One Atom feed per collection |

Per-collection feeds use `page.date` and `summary_html` for richer entries.

## Shortcodes (template helpers)

The minijinja engine registers two global functions:

```jinja
<link rel="stylesheet" href="{{ asset_url('assets/extra.css') }}">

<img src="/_resized/cover-800.jpg"
     srcset="{{ image_set('cover.jpg', [400, 800, 1600]) }}"
     sizes="(max-width: 600px) 400px, 800px"
     loading="lazy">
```

- `asset_url(path)` — prepends `site.base`
- `image_set(path, sizes)` — builds an `srcset` string against `/_resized/`

## Asset pipelines (cargo features)

### `sass`

Pure-Rust SCSS compilation via [`grass`](https://crates.io/crates/grass).

```bash
cargo install --path crates/cli --features novel-core/sass
```

```toml
[sass]
entries    = [["assets/scss/main.scss", "assets/css/main.css"]]
load_paths = ["assets/scss"]
```

Each entry resolves relative to the project root for input and to the output directory for the destination.

### `images`

Resize source images via the [`image`](https://crates.io/crates/image) crate.

```bash
cargo install --path crates/cli --features novel-core/images
```

```toml
[images]
sizes   = [400, 800, 1600]
quality = 82
```

Outputs land in `dist/_resized/<path>/<stem>-<width>.<ext>` and pair with the `image_set()` template helper.

## OG / Twitter Card meta

The default `base.html` already emits a rich Open Graph block when `site.site_url` is set. Content-page frontmatter automatically populates:

| OG meta | Source |
|---|---|
| `article:published_time` | `frontmatter.date` |
| `article:modified_time` | `frontmatter.updated` |
| `article:author` (per author) | `frontmatter.authors` |
| `article:tag` (per tag) | `frontmatter.tags` |

## Theme packs

Distribute reusable themes as plain directories:

```toml
[theme]
pack = "./themes/midnight"
```

Template lookup order:

1. `<project>/templates/<name>` — your project overrides
2. `<project>/<theme.pack>/templates/<name>` — the theme pack
3. Embedded defaults shipped with `novel-core`

Theme packs can also drop CSS / JS that you reference via `theme.custom_css` or `asset_url`.
