---
title: Frontmatter
description: 通过 Markdown frontmatter 控制页面元数据、布局和导航行为。
---

# Frontmatter

每个 Markdown 文件都可以在开头加入以 `---` 包围的 YAML frontmatter。Frontmatter 用于控制页面元数据和布局选项。

## 基础字段

```yaml
---
title: 页面标题
description: 本页面的简短描述
---
```

| 字段 | 类型 | 默认值 | 描述 |
|-------|------|---------|-------------|
| `title` | string | 第一个 `# 一级标题` | 页面标题(用于 `<title>` 标签和侧边栏) |
| `description` | string | `""` | 页面描述(用于 `<meta>` 标签) |

### 内容 / 博客字段

对于博客和内容集合,还识别以下额外字段。完整说明见[通用 SSG 模式](./general-ssg)。

| 字段 | 类型 | 描述 |
|---|---|---|
| `published_at` | `YYYY-MM-DD` | 发布日期 —— 用于排序、归档、订阅和 OG `article:published_time` |
| `updated_at` | `YYYY-MM-DD` | 最后更新日期 —— 对应 OG `article:modified_time` |
| `draft` | bool | 构建时排除,除非传入 `--drafts` |
| `weight` | int | 当 `sort_by = "weight"` 时作为排序键 |
| `summary` | string | 手动摘要(覆盖 `<!-- more -->` 分隔符) |
| `taxonomies` | map | 按已配置分类 key 组织的分类条目 |
| `series` | string | 系列 id —— 生成 `/series/<slug>/` |
| `authors` | list | 作者名 —— 对应 OG `article:author` |
| `expires_at` | `YYYY-MM-DD` | 该日期之后被排除,除非传入 `--future` |

## 页面类型

通过 `layout` 控制页面布局:

```yaml
---
layout: home   # 或 doc、custom、404
---
```

| 值 | 描述 |
|-------|-------------|
| `home` | 带 hero 与 features 网格的首页 |
| `doc` | 标准文档页(默认) |
| `custom` | 自定义页面,无侧边栏 |
| `404` | 未找到页面 |

## 布局控制

```yaml
---
sidebar: false    # 该页隐藏侧边栏
navbar: false     # 该页隐藏导航栏
outline: false    # 该页隐藏目录大纲
---
```

## Hero 与 Features

仅在 `layout: home` 时使用。完整说明见[首页](/guide/home-page)。

```yaml
---
layout: home
hero:
  name: 项目名
  text: 标语文字
  tagline: 更长的描述
  actions:
    - text: 开始使用
      url: /zh/guide/
      theme: brand
features:
  - title: 特性
    icon: "\u26A1"
    details: 描述
---
```

## 自定义 head 标签

为特定页面添加自定义的 `<head>` 标签:

```yaml
---
head:
  - tag: meta
    attrs:
      property: og:title
      content: 我的页面标题
  - tag: link
    attrs:
      rel: canonical
      href: https://example.com/page
  - tag: title
    content: 自定义浏览器标题
---
```

为提供安全的默认行为,自定义 head 项只允许 `meta`、`link` 和
`title`,并按标签限制可用属性。构建时会拒绝 `script`、`style`、
`base`、`meta http-equiv`、事件处理属性和危险 URL scheme 等可执行或改变
导航的配置。如果站点确实需要分析脚本或其他脚本,请在受信任的自定义模板中添加。

::: warning 受信任输入边界
Markdown 原始 HTML、自定义模板、主题包和自定义 CSS 会被当作受信任的项目代码,
不会经过净化处理。不要直接构建来自不受信任贡献者且未经审核的内容。
:::
