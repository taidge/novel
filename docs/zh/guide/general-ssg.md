# 通用 SSG 模式

除了文档站点之外,Novel 还可以构建博客、营销页、作品集以及其他内容型站点。通用 SSG 相关特性是**可选启用**的,叠加在现有的文档站点体验之上,而不会破坏它。

## 集合(Collections)

集合是 `docs/` 下包含 `_collection.toml` 标记文件的顶层目录。目录中的页面会被分组、排序、分页,并可通过过滤访问。

```
docs/
├── guide/                 # 现有文档分区(不受影响)
└── posts/
    ├── _collection.toml   # 标记
    ├── hello-world.md
    ├── second-post.md
    └── third-post.md
```

`docs/posts/_collection.toml`:

```toml
layout = "blog"             # 条目的默认布局
list_layout = "list"        # /posts/ 索引页的布局
sort_by = "date"            # date | weight | title
order = "desc"              # desc | asc
paginate_by = 10            # 0 表示禁用分页
publish = true
```

输出:

```
dist/posts/hello-world/index.html      # 条目页
dist/posts/index.html                  # 分页列表(第 1 页)
dist/posts/page/2/index.html           # 第 2 页
```

## 内容页面的 Frontmatter

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

在列表页中显示的引言段落。

<!-- more -->

正文剩余部分。
```

| 字段 | 类型 | 描述 |
|---|---|---|
| `date` | `YYYY-MM-DD` | 发布日期 —— 用于排序、归档、订阅源 |
| `updated` | `YYYY-MM-DD` | 最后更新日期 —— 通过 OG `article:modified_time` 暴露 |
| `draft` | bool | 构建时排除,除非使用 `--drafts` |
| `weight` | int | 当 `sort_by = "weight"` 时用作排序键 |
| `summary` | string | 手动摘要(覆盖 `<!-- more -->` 提取) |
| `tags` | list | 分类条目(见下文) |
| `categories` | list | 分类条目 |
| `series` | string | 系列标识符 —— 生成 `/series/<slug>/` |
| `authors` | list | 作者名 —— 通过 OG `article:author` 暴露 |
| `expiry_date` | `YYYY-MM-DD` | 该日期之后页面被排除,除非使用 `--future` |

## 草稿和未来日期的内容

```bash
novel build              # 排除草稿和未来日期的页面
novel build --drafts     # 包含 draft: true 的页面
novel build --future     # 包含 date > 今天 的页面
```

同样的开关也可以通过配置设置:

```toml
[content]
drafts = false
future = false
summary_separator = "<!-- more -->"
```

## 摘要

如果一个 Markdown 文件包含 `<!-- more -->`,那么它之前的所有内容都会成为该页的 `summary_html`,被以下位置使用:

- 列表页(`/posts/`)
- 词条页(`/tags/<term>/`)
- 每个集合的订阅源
- JSON Feed 条目

`frontmatter.summary` 会覆盖分隔符。

## 分类(Taxonomies)

在 `novel.toml` 中配置:

```toml
[taxonomies.tags]
name = "Tags"

[taxonomies.categories]
name = "Categories"
permalink = "/cat/{slug}/"   # 可选,默认为 /<key>/<slug>/
paginate_by = 10              # 可选,默认不分页
```

Novel 会为每个分类生成:

```
dist/tags/index.html           # 概览(标签云)
dist/tags/<term>/index.html    # 单个词条,带分页
```

## 分页

```toml
[pagination]
page_path = "page"             # /posts/page/2/
first_page_in_root = true      # /posts/ 而不是 /posts/page/1/
```

同一个分页器同时驱动集合列表页和分类词条页。

## 系列(Series)

任何带有 `series: <name>` 的页面都会加入一个虚拟的系列索引:

```
dist/series/<slug>/index.html
```

条目按**日期升序**排序,这样阅读顺序才合理。

## 日期归档

所有设置了 `date` 的页面都会被自动分组:

```
dist/archive/2026/index.html         # 年度归档
dist/archive/2026/04/index.html      # 月度归档
```

## 数据文件

将 `.toml` 或 `.json` 文件放到 `docs/data/` 下,它们就会作为 `site_data` 在所有模板中可用:

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

在模板中:

```jinja
<a href="{{ site_data.authors.chris.url }}">{{ site_data.authors.chris.name }}</a>
```

子目录会更深地嵌套到树中,因此 `data/nav/links.json` 可以通过 `site_data.nav.links` 访问。

## 订阅源

`FeedPlugin`(CLI 中默认启用)在设置了 `site.site_url` 时会生成:

| 文件 | 格式 |
|---|---|
| `feed.xml` | 站点范围的 Atom 1.0 |
| `feed.json` | 站点范围的 JSON Feed v1.1 |
| `<collection>/feed.xml` | 每个集合一个 Atom feed |

每个集合的订阅源会使用 `page.date` 和 `summary_html`,使条目更丰富。

## Shortcode(模板辅助函数)

MiniJinja 引擎注册了两个全局函数:

```jinja
<link rel="stylesheet" href="{{ asset_url('assets/extra.css') }}">

<img src="/_resized/cover-800.jpg"
     srcset="{{ image_set('cover.jpg', [400, 800, 1600]) }}"
     sizes="(max-width: 600px) 400px, 800px"
     loading="lazy">
```

- `asset_url(path)` —— 前置 `site.base`
- `image_set(path, sizes)` —— 构建针对 `/_resized/` 的 `srcset` 字符串

## 资源管线(cargo feature)

### `sass`

通过 [`grass`](https://crates.io/crates/grass) 进行纯 Rust 的 SCSS 编译。

```bash
cargo install --path crates/cli --features novel-core/sass
```

```toml
[sass]
entries    = [["assets/scss/main.scss", "assets/css/main.css"]]
load_paths = ["assets/scss"]
```

每个条目的输入相对于项目根目录解析,输出相对于输出目录解析。

### `images`

通过 [`image`](https://crates.io/crates/image) crate 对源图片进行缩放。

```bash
cargo install --path crates/cli --features novel-core/images
```

```toml
[images]
sizes   = [400, 800, 1600]
quality = 82
```

输出位于 `dist/_resized/<path>/<stem>-<width>.<ext>`,与 `image_set()` 模板辅助函数配对使用。

## OG / Twitter Card meta

默认的 `base.html` 在设置了 `site.site_url` 时已经会输出丰富的 Open Graph 块。内容页的 frontmatter 会自动填充:

| OG meta | 来源 |
|---|---|
| `article:published_time` | `frontmatter.date` |
| `article:modified_time` | `frontmatter.updated` |
| `article:author`(每个作者一个) | `frontmatter.authors` |
| `article:tag`(每个标签一个) | `frontmatter.tags` |

## 主题包

把可复用的主题以普通目录的形式分发:

```toml
[theme]
pack = "./themes/midnight"
```

模板查找顺序:

1. `<project>/templates/<name>` —— 你项目的覆盖
2. `<project>/<theme.pack>/templates/<name>` —— 主题包
3. `novel-core` 内置的默认模板

主题包也可以提供 CSS / JS,你可以通过 `theme.custom_css` 或 `asset_url` 来引用。
