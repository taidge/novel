# 快速开始

几分钟内让 Novel 跑起来。

## 安装

```bash
cargo install novel-cli
```

## 创建新项目

```bash
novel init my-docs
cd my-docs
```

这会创建一个具有以下结构的项目:

```
my-docs/
├── docs/
│   ├── index.md          # 首页
│   └── guide/
│       ├── _meta.json    # 侧边栏排序
│       ├── getting-started.md
│       └── markdown.md
├── novel.toml            # 配置
└── .gitignore
```

## 开发

启动带有热重载的开发服务器:

```bash
novel dev
```

在浏览器中打开 `http://localhost:3000`。对 `.md`、`.json` 或 `.toml` 文件的修改会自动触发重新构建。

## 生产构建

```bash
novel build
```

输出会写入 `dist/` 目录。使用任意静态文件服务器即可发布。

## 预览

```bash
novel preview
```

在本地 4000 端口上预览构建后的输出。

## 配置

在项目根目录下创建 `novel.toml`:

```toml title="novel.toml"
title = "My Docs"
description = "My documentation site"
root = "docs"
out_dir = "dist"
base = "/"
lang = "zh"
site_url = "https://example.com"  # 启用 sitemap 与 RSS

[markdown]
show_line_numbers = false   # 代码块是否显示行号
check_dead_links = false    # 构建时校验内部链接

[theme]
dark_mode = true
footer = "Built with Novel"
last_updated = true         # 显示 git 最后更新时间
edit_link = "https://github.com/user/repo/edit/main/docs/"
source_link = "https://github.com/user/repo"

# 公告横幅
[theme.banner]
text = "Novel v0.2 发布!"
link = "/guide/getting-started"
dismissible = true
```

::: tip
大多数字段都有合理的默认值。只需要配置你想定制的部分即可。
:::

## 作为库使用

Novel 也可以嵌入到你自己的 Rust 应用中 —— 参见[库 API](/zh/guide/library) 指南。
