# 主题与外观

Novel 自带一个简洁、响应式的默认主题(内置深色模式),并且在设计上允许你按需进行任意程度的定制 —— 小到调整一个颜色变量,大到发布一个完整的可复用**主题包**。

定制能力是分层的。选择能够满足你需求的最高层级即可:

| 层级 | 你能改动的内容 | 工作量 |
|------|----------------|--------|
| 1. 配置选项 | Logo、页脚、横幅、导航、社交链接 | 几分钟 |
| 2. CSS 变量 | 通过 `[theme.colors]` 设置品牌色、间距、字体 | 几分钟 |
| 3. 自定义 CSS | 在默认样式之后加载的任意样式表 | 低 |
| 4. 模板覆盖 | 替换你 `templates/` 目录中的单个模板 | 中等 |
| 5. 主题包 | 发布一个包含模板 + CSS + 资源的可复用目录 | 中等 |
| 6. 切换引擎 | 使用 Tera 或 Handlebars 替代 MiniJinja | 中等 |

## 1. 内置选项

大多数站点只需要 `[theme]` 下的选项即可,不需要其他任何东西。全部选项都记录在[配置](./configuration.md)中,以下是常用的几项:

```toml title="novel.toml"
[theme]
dark_mode      = true
footer         = "Copyright 2025 My Project | Built with Novel"
edit_link      = "https://github.com/user/repo/edit/main/docs/"
edit_link_text = "Edit this page on GitHub"
last_updated   = true
source_link    = "https://github.com/user/repo"

[theme.banner]
text        = "We just released v2.0!"
link        = "/guide/changelog"
dismissible = true
```

### 深色模式

深色模式默认启用。偏好设置会保存到 `localStorage`;如果没有存储任何偏好,Novel 会跟随 `prefers-color-scheme`。将 `dark_mode` 设为 `false` 可隐藏切换按钮。

### 响应式布局

默认主题在桌面端是三列布局(侧边栏 / 内容 / TOC),在平板上是两列布局(内容 / TOC),在移动端是单列布局并配有汉堡菜单。

## 2. 通过 CSS 变量定制品牌色

整个默认样式表由 CSS 自定义属性驱动。你可以**无需编辑 CSS 文件**就覆盖其中**任何一个** —— 只需要在 `novel.toml` 中设置 `[theme.colors]`。键名就是 CSS 变量名,**不带**开头的 `--`:

```toml title="novel.toml"
[theme.colors]
accent       = "#e11d48"
accent-hover = "#be123c"
accent-bg    = "#fff1f2"
bg-primary   = "#fffaf5"
text-primary = "#1f1f28"
font-sans    = '"Inter", system-ui, sans-serif'
```

Novel 会将它们写入页面,形如:

```html
<style>:root, [data-theme="light"] { --accent: #e11d48; /* ... */ }</style>
```

你可能会想要覆盖的常用变量:

| 变量 | 默认值 | 用途 |
|------|--------|------|
| `--accent` | `#3b82f6` | 主品牌色(链接、按钮) |
| `--accent-hover` | `#2563eb` | accent 的悬停状态 |
| `--accent-bg` | `#eff6ff` | 带色调的背景(提示、高亮) |
| `--bg-primary` | `#ffffff` | 页面背景 |
| `--bg-secondary` | `#f6f8fa` | 侧边栏、卡片背景 |
| `--text-primary` | `#1a1a2e` | 正文文本 |
| `--text-secondary` | `#4a5568` | 次要文本 |
| `--border-color` | `#e2e8f0` | 分割线、边框 |
| `--code-bg` | `#f1f5f9` | 内联代码背景 |
| `--font-sans` | 系统 UI 字体栈 | 正文字体 |
| `--font-mono` | JetBrains Mono 字体栈 | 代码字体 |
| `--sidebar-width` | `260px` | 左侧边栏宽度 |
| `--toc-width` | `220px` | 右侧 TOC 宽度 |
| `--navbar-height` | `60px` | 顶部导航栏高度 |

完整列表请参阅 [`crates/core/assets/style.css`](https://github.com/chrislearn/novel/blob/main/crates/core/assets/style.css),其中包含容器(tip / warning / danger)颜色、横幅颜色、diff 颜色,以及 `[data-theme="dark"]` 下的深色模式覆盖。

::: tip
`[theme.colors]` 中的覆盖只会影响 `light` 主题块。若也要为深色模式重新配色,请使用自定义 CSS 文件(下一节)并以 `[data-theme="dark"]` 作为选择器。
:::

## 3. 自定义 CSS 文件

当一次性的变量覆盖不够用时,将 `theme.custom_css` 指向一个 CSS 文件(路径相对于项目根目录)。它的内容会在默认样式表**之后**被内联到每个页面,因此可以覆盖任何内容:

```toml title="novel.toml"
[theme]
custom_css = "assets/custom.css"
```

```css title="assets/custom.css"
/* 重新配色深色主题 */
[data-theme="dark"] {
    --accent: #fbbf24;
    --bg-primary: #0a0a0f;
}

/* 自定义字体 */
@font-face {
    font-family: "My Brand";
    src: url("/fonts/brand.woff2") format("woff2");
}
:root { --font-sans: "My Brand", system-ui, sans-serif; }

/* 微调组件 */
.hero-name {
    background: linear-gradient(90deg, #f97316, #e11d48);
    -webkit-background-clip: text;
    color: transparent;
}
```

::: note
`custom_css` 是**内联**的,不是通过 `<link>` 引入 —— 它会为每个 HTML 页面增加字节数。对于较大的样式表,请将文件放在 `docs/` 下作为[静态资源](./static-assets.md),然后从 `base.html` 覆盖中自行 `<link>`。
:::

## 4. 模板覆盖

Novel 默认使用 [MiniJinja](https://docs.rs/minijinja)(一个兼容 Jinja2 的引擎)。渲染时,它按以下顺序查找每个模板:

1. `<project_root>/templates/<name>`
2. `<project_root>/<theme.pack>/templates/<name>`(如果设置了主题包)
3. 嵌入在二进制中的默认模板

这意味着你可以**只**覆盖你关心的模板。任何你没有提供的模板,仍会回退到内置版本。

### 可用模板

| 模板 | 渲染内容 | 继承自 |
|------|----------|--------|
| `base.html` | 页面外壳(`<html>`、导航栏、页脚、`<head>`) | — |
| `doc.html` | 文档页面(侧边栏 + 内容 + TOC) | `base.html` |
| `home.html` | 带 hero + features 的落地页 | `base.html` |
| `page.html` | 全宽页面(无侧边栏、无 TOC) | `base.html` |
| `blog.html` | 带日期标题的居中文章布局 | `base.html` |
| `list.html` | 分页列表(集合、分类词条) | `base.html` |
| `terms.html` | 分类概览(例如 `/tags/`) | `base.html` |
| `404.html` | 404 页面 | `base.html` |

目录结构示例:

```text
my-site/
├── novel.toml
├── docs/
│   └── ...
└── templates/
    ├── base.html        ← 可选:完整覆盖
    ├── doc.html         ← 仅覆盖文档页
    └── home.html        ← 仅覆盖首页
```

### 覆盖区块而不是整个文件

由于每个布局都继承自 `base.html`,你几乎不需要重写整个模板。在你的 `templates/` 目录中创建一个文件,让它从内置模板 `{% extends %}`,并只覆盖你想修改的区块。`base.html` 暴露的区块:

| 区块 | 含义 |
|------|------|
| `title` | `<title>` 内部 |
| `description` | `<meta name="description">` 标签内部 |
| `head` | `<head>` 内的额外标签(在 OG / JSON-LD **之前**渲染) |
| `head_extra` | 在 OG / JSON-LD 之后渲染的额外 `<head>` 标签 |
| `og_meta` | Open Graph + Twitter card 标签 |
| `json_ld` | 结构化数据脚本 |
| `nav_extra` | 追加到 `<nav class="navbar">` 内部的内容 |
| `content_before` / `content` / `content_after` | 页面主体 |
| `footer_extra` | 追加到 `<footer>` 内部的内容 |

```html title="templates/base.html"
{% extends "base.html" %}
{# 这会无限循环 —— 应改为覆盖具体布局。 #}
```

一个更有用的模式 —— 通过覆盖 `doc.html` 为每个页面添加自定义样式表 + Plausible 分析(如果你确实需要对所有布局生效,也可以覆盖 `base.html`):

```html title="templates/doc.html"
{% extends "base.html" %}

{% block head_extra %}
    <link rel="stylesheet" href="{{ asset_url('css/extra.css') }}">
    <script defer data-domain="example.com" src="https://plausible.io/js/script.js"></script>
{% endblock %}

{# 原样复用默认的文档主体。 #}
{% block content %}{{ super() }}{% endblock %}
```

::: warning
如果你直接覆盖 `base.html`(而不是某个子布局),请在 `templates/base.html` 中放一个**不同**的文件 —— 不要用 `{% extends "base.html" %}`,因为那会解析为*你自己的*文件并陷入循环。最简单的办法是复制上游的 [`base.html`](https://github.com/chrislearn/novel/blob/main/crates/core/templates/base.html) 再进行编辑。
:::

### 渲染上下文

每个模板都会收到以下上下文:

| 变量 | 类型 | 描述 |
|------|------|------|
| `site` | object | 完整的 `SiteConfig`(`site.title`、`site.base`、`site.theme.*`、`site.markdown.*` 等) |
| `page` | object 或 null | 当前页面(`page.title`、`page.content_html`、`page.frontmatter`、`page.toc`、`page.last_updated`、`page.prev_page`、`page.next_page`、`page.breadcrumbs` 等) |
| `nav` | array | 顶部导航项 |
| `sidebar` | array | 当前分区的侧边栏项 |
| `toc` | array | 当前页面的目录项 |
| `edit_url` | string 或 null | 计算得到的"编辑此页"URL |
| `edit_link_text` | string | 编辑链接的文字 |
| `last_updated_text` | string | 最后更新时间戳的文字 |
| `theme_css_overrides` | string 或 null | 渲染后的 `[theme.colors]` CSS 字符串 |
| `custom_css_content` | string 或 null | 内联的 `theme.custom_css` 内容 |
| `asset_css` / `asset_js` | string | 启用 `asset_fingerprint` 时带哈希的资源文件名 |
| `site_data` | object | 从 `<docs>/data/` 加载的数据文件 |
| `paginator` | object 或 null | 列表页上:`items`、`current_page`、`total_pages`…… |
| `terms` | array 或 null | 分类概览页上 |
| `list_title` | string 或 null | 列表页 / 词条页的标题 |

### 内置模板函数

每次渲染时都会注册两个辅助函数:

- `asset_url(path)` —— 为路径加上站点 `base` 前缀,例如当 `base = "/docs/"` 时,`asset_url("css/extra.css")` → `/docs/css/extra.css`。
- `image_set(path, sizes)` —— 为图像管线生成的图片构建响应式 `srcset` 字符串:
  ```html
  <img src="{{ asset_url('images/hero.jpg') }}"
       srcset="{{ image_set('images/hero.jpg', [400, 800, 1600]) }}"
       sizes="(max-width: 600px) 100vw, 50vw">
  ```

插件可以注册额外的辅助函数 —— 参见[库指南](./library.md)中的 `Plugin::register_template_helpers` 钩子。

## 5. 主题包(可复用主题)

**主题包**就是一个包含模板 + 资源覆盖的目录,你只需将 `theme.pack` 指向它即可。主题包的查找顺序在 `<project_root>/templates/` *之后*,但在内置模板 *之前*,所以 `templates/` 中的站点级覆盖仍会优先于主题包。

```text
themes/
└── midnight/
    └── templates/
        ├── base.html
        ├── doc.html
        └── home.html
```

```toml title="novel.toml"
[theme]
pack       = "./themes/midnight"
custom_css = "./themes/midnight/assets/style.css"
```

你可以把主题包放进它自己的 git 仓库、发布出去,并在多个项目之间共享。主题包的使用者仍然可以通过在本地 `templates/` 中放入同名文件来覆盖任何内容。

::: tip
主题包内只有 `templates/` 会被自动解析。如果你的主题包里有 CSS、图片或 JS,请通过 `theme.custom_css` 显式引用,或通过符号链接 / 复制到 `docs/` 下,让它们作为[静态资源](./static-assets.md)被采集。
:::

## 6. 切换模板引擎

默认使用 MiniJinja。如果你偏好 Tera 或 Handlebars 的语法,请在 `novel-core` 上启用对应的 cargo feature 并设置 `template_engine`:

```toml title="novel.toml"
template_engine = "tera"        # 或 "handlebars" 或 "minijinja"(默认)
```

每种引擎都有自己的一套内置模板,分别位于 `crates/core/templates_tera/` 和 `crates/core/templates_hbs/`。你项目 `templates/` 文件夹中的覆盖必须使用与你所选引擎匹配的语法。

## 7. 自定义资源(CSS / JS)

默认的样式表和 JS 被编译到二进制中,名为 `style.css` 和 `main.js`,并在每次构建时写入 `<out_dir>/assets/`。你有三种方式来附加额外资源:

1. **自定义 CSS**(最简单,内联到每个页面) —— 使用 `theme.custom_css`。
2. **静态文件** —— 将文件放到 `docs/`(或 `root` 指向的位置)之下,并从模板中引用它们。它们会被原样复制。参见[静态资源](./static-assets.md)。
3. **Sass 管线** —— 启用 `sass` cargo feature 并配置 `[sass]`:
   ```toml title="novel.toml"
   [sass]
   entries    = [["assets/scss/main.scss", "assets/css/main.css"]]
   load_paths = ["assets/scss"]
   ```

如果设置了 `asset_fingerprint = true`,Novel 会将 `style.css` / `main.js` 哈希为 `style.<hash>.css` / `main.<hash>.js`,并通过模板上下文中的 `asset_css` / `asset_js` 暴露出来 —— 在覆盖模板中请使用它们,而不要硬编码文件名。

## 内置组件

默认主题开箱即用地包含一些可在 Markdown 中使用的可视组件:

| 组件 | 描述 |
|------|------|
| **容器指令** | `::: tip`、`warning`、`danger`、`info`、`note`、`details` |
| **选项卡** | 带标签页的内容块 |
| **步骤** | 编号的分步指南 |
| **徽章** | 内联状态徽章 |
| **代码块** | 语法高亮、行号、标题、复制按钮 |
| **图片缩放** | 点击图片放大 |
| **返回顶部** | 滚动时显示的浮动按钮 |
| **目录** | 从页面标题自动生成 |

每种组件的语法请参见 [Markdown 指南](./markdown.md)。

## 配方:最小化的自定义主题

把前面的内容组合起来 —— 一个完全换肤的站点:

```text
my-site/
├── novel.toml
├── assets/
│   └── custom.css
└── templates/
    └── base.html
```

```toml title="novel.toml"
title = "Acme Docs"
[theme]
custom_css = "assets/custom.css"

[theme.colors]
accent       = "#f97316"
accent-hover = "#ea580c"
bg-primary   = "#fffaf5"
font-sans    = '"Inter", system-ui, sans-serif'
```

```css title="assets/custom.css"
[data-theme="dark"] {
    --accent: #fb923c;
    --bg-primary: #0b0b0f;
    --bg-secondary: #14141c;
}
.navbar { backdrop-filter: blur(12px); }
```

```html title="templates/base.html"
{# 从上游 base.html 复制后修改 —— 参见"主题 › 覆盖区块"中的链接 #}
<!DOCTYPE html>
<html lang="{{ site.lang }}" data-theme="light">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}{{ site.title }}{% endblock %}</title>
    <link rel="stylesheet" href="{{ site.base }}assets/{{ asset_css }}">
    {% if theme_css_overrides %}<style>:root { {{ theme_css_overrides | safe }} }</style>{% endif %}
    {% if custom_css_content %}<style>{{ custom_css_content | safe }}</style>{% endif %}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet">
    {% block head %}{% endblock %}
</head>
<body>
    <!-- 你的自定义布局 -->
    {% block content %}{% endblock %}
</body>
</html>
```

就是这样 —— 一个完全换肤的站点,拥有自定义外壳、自定义配色和自定义字体,仍然由 Novel 的路由、Markdown 和搜索能力驱动。
