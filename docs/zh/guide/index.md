---
title: 指南
description: 使用 Novel 搭建文档站点所需的一切 —— 从你的第一个页面到高级功能。
---

# 指南

欢迎来到 Novel 指南。Novel 是一个使用 Rust 构建的极速静态文档站点生成器 —— 既可以作为独立的 CLI 工具使用,也可以作为库嵌入到你自己的 Rust 应用程序中。

> English version: [Guide](/en/guide/)

## 从这里开始

初次使用 Novel?按顺序阅读以下页面:

- [**快速开始**](./getting-started.md) —— 安装 CLI、创建项目、运行开发服务器
- [**配置**](./configuration.md) —— `novel.toml` 参考(title、base、theme、markdown 选项等)
- [**路由**](./routing.md) —— 磁盘上的文件如何映射为 URL
- [**Markdown**](./markdown.md) —— GFM、语法高亮、选项卡、步骤、徽章、容器指令
- [**Frontmatter**](./frontmatter.md) —— 页面级元数据:title、layout、date、tags、canonical、noindex 等

## 编写内容

- [**首页**](./home-page.md) —— 使用 hero 和 features 构建一个落地页
- [**文件嵌入**](./file-embed.md) —— 嵌入外部源代码文件,让文档与代码保持同步
- [**静态资源**](./static-assets.md) —— 图片、下载文件和其他公共资源

## 不止文档 —— 通用 SSG 模式

Novel 不仅是文档生成器。它对通用静态站点提供了一级支持:

- [**通用 SSG**](./general-ssg.md) —— 集合、分类、分页、数据文件,以及让 Novel 成为 Hugo / Zola 替代方案的所有能力

## 自定义

- [**主题与外观**](./theming.md) —— 颜色、深色模式、页脚、编辑链接、公告横幅、模板覆盖、主题包
- [**搜索**](./search.md) —— 内置的客户端搜索索引

## 发布

- [**部署**](./deploy.md) —— 将站点发布到 GitHub Pages、Netlify、Vercel、Cloudflare Pages 或任意静态主机
- [**库 API**](./library.md) —— 将 Novel 嵌入到你自己的 Rust 应用或 Web 服务器中

::: tip
大多数页面都有合理的默认值 —— 只需几行 `novel.toml` 配置和一个 `docs/` 目录的 Markdown 文件,你就能得到一个可用的站点。先从[快速开始](./getting-started.md)入手,需要定制某个具体能力时再回来查阅。
:::
