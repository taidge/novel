# Novel

[English](./README.md) | [简体中文](./README.zh.md)

**基于 Rust 构建的高性能静态文档站点生成器 —— 可独立运行，也可作为库嵌入使用。**

Novel 能够在毫秒级时间内将一个 Markdown 文件夹转换为精美的文档网站。你可以把它当作命令行工具使用，也可以作为库直接嵌入到自己的 Rust 应用中。

## 特性

- **极速构建** —— 完全使用 Rust 编写，构建速度以毫秒计，而非秒。
- **可嵌入** —— 既可作为 CLI 使用,也可作为库嵌入：`DirNovel::new("docs").build()`。
- **Markdown 优先** —— 开箱即用的 GFM、语法高亮、选项卡、步骤、徽章和容器指令。
- **文件嵌入** —— 支持从外部文件按行范围嵌入源代码，让文档与代码保持同步。
- **精美主题** —— 内置深色模式、响应式布局、搜索、上下页导航和图片缩放。
- **SEO 与 AI 友好** —— 内置站点地图、RSS/JSON Feed、`llms.txt`、编辑链接、最后更新时间戳和自定义 head 标签。
- **完整文档站能力** —— 支持文档版本化、每页 Markdown 镜像、可选 PWA 输出和静态页面反馈。

## 安装

Novel 要求 Rust 1.94 或更高版本。crates.io 上的 `novel-cli` 名称属于另一个项目,
因此请从本项目的 GitHub 仓库安装 CLI:

```bash
cargo install --git https://github.com/taidge/novel --package novel-cli --locked
```

## 快速开始

```bash
# 创建新项目
novel init my-docs
cd my-docs

# 启动支持热重载的开发服务器
novel dev

# 生产环境构建
novel build

# 预览生产构建结果
novel preview
```

新项目的目录结构如下：

```
my-docs/
├── docs/
│   ├── index.md          # 首页
│   └── guide/
│       ├── _meta.json    # 侧边栏顺序
│       ├── getting-started.md
│       └── markdown.md
├── novel.toml            # 配置文件
└── .gitignore
```

开发服务器默认运行在 `http://localhost:3000`，当 `.md`、`.json` 或 `.toml` 文件发生变化时会自动重新构建。

## 配置

在项目根目录创建 `novel.toml`：

```toml
title = "My Docs"
description = "My documentation site"
docs_dir = "docs"
output_dir = "dist"
base = "/"
lang = "zh"
site_url = "https://example.com"  # 启用站点地图和 RSS

[markdown]
show_line_numbers = false
check_dead_links = false

[theme]
dark_mode = true
footer = "Built with Novel"
show_git_updated_at = true
edit_url = "https://github.com/user/repo/edit/main/docs/"
source_url = "https://github.com/user/repo"
```

大多数字段都有合理的默认值 —— 你只需配置想要自定义的部分即可。

## 作为库使用

Novel 可以直接嵌入到你自己的 Rust 应用中。

```toml
[dependencies]
novel-core = { path = "path/to/novel/crates/core" }
```

仅需三行即可构建并写入磁盘：

```rust
use novel_core::{DirNovel, Novel};

let site = DirNovel::new("docs").build()?;
site.write_to("dist")?;
```

或通过 Builder API 进行定制：

```rust
use novel_core::{DirNovel, Novel};

let site = DirNovel::new("docs")
    .title("My API Reference")
    .description("Generated docs for my-crate")
    .base("/docs/")
    .site_url("https://example.com")
    .with_theme(|t| {
        t.dark_mode = true;
        t.footer = Some("Built with Novel".into());
        t.show_git_updated_at = true;
    })
    .build()?;

site.write_to("./output")?;
```

### 在 Web 服务器中使用

`.build()` 返回的 `BuiltSite` 可以按需渲染页面，因此能轻松集成到 Axum（或任何其他）服务器中：

```rust
use novel_core::{DirNovel, Novel};

let site = DirNovel::new("docs").build()?;
let page = site.page("/guide/intro").unwrap();
let html = site.render_page(page)?;
```

完整的 API 参考请查阅 [docs/guide/library.md](./docs/guide/library.md)。

## 工作空间结构

Novel 是一个 Cargo 工作空间，包含三个 crate：

| Crate | 说明 |
|-------|------|
| `crates/shared` | 共享类型与配置 |
| `crates/core`   | 站点构建和渲染引擎 |
| `crates/cli`    | `novel` 命令行二进制 |

## 文档

完整文档位于 `docs/` 目录，可通过 `novel build` 构建。涵盖以下主题：

- 快速上手、配置、路由
- Markdown 特性、Frontmatter、文件嵌入
- 主题、首页、静态资源
- 搜索、部署、库 API

## 许可证

基于 [Apache License 2.0](./LICENSE) 开源。
