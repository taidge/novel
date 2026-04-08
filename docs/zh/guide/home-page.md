# 首页

首页是通过根目录 `docs/index.md` 文件的 frontmatter 来配置的。

## 设置

在 frontmatter 中设置 `page_type: home` 以启用首页布局:

```yaml title="docs/index.md"
---
page_type: home
hero:
  name: 我的项目
  text: 创造令人惊叹的东西
  tagline: 为开发者打造的快速现代工具
  actions:
    - text: 开始使用
      link: /guide/getting-started
      theme: brand
    - text: GitHub
      link: https://github.com/user/repo
      theme: alt
features:
  - title: 快速
    icon: "\u26A1"
    details: 基于 Rust 构建,追求极致速度。
  - title: 简单
    icon: "\U0001F4DD"
    details: 写 Markdown,得到一个漂亮的站点。
  - title: 灵活
    icon: "\U0001F527"
    details: 一切皆可按需自定义。
---
```

## Hero 区域

Hero 区域是首页顶部的大横幅。

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `name` | string | 大号标题文字 |
| `text` | string | 次级标题 |
| `tagline` | string | 标题下方的描述文字 |
| `actions` | array | 行动按钮 |
| `image` | object | 可选的 Hero 图片 |

### 按钮

每个按钮都是 Hero 区域中的一个行动号召:

```yaml
actions:
  - text: 开始使用           # 按钮文字
    link: /guide/intro       # 跳转地址
    theme: brand             # "brand"(主色) 或 "alt"(次色)
```

### Hero 图片

```yaml
hero:
  image:
    src: /hero-image.png
    alt: 我的项目 Logo
```

## Features 网格

Features 区域在 Hero 下方以网格形式展示特性卡片:

```yaml
features:
  - title: 特性名称
    icon: "\U0001F680"        # emoji 或文字图标
    details: 该特性的描述。
    link: /guide/feature       # 可选:使卡片可点击
```

| 字段 | 类型 | 描述 |
|-------|------|-------------|
| `title` | string | 特性卡片标题 |
| `icon` | string | 显示在标题上方的 emoji 或文字 |
| `details` | string | 特性描述 |
| `link` | string | 可选 URL —— 使卡片可点击 |

## 正文内容

`---` 之下的任何 Markdown 内容会被渲染在 Features 网格下方。可以用来为首页追加额外的章节。
