# 配置

Novel 通过位于项目根目录的 `novel.toml` 文件进行配置。所有字段均为可选,且都有合理的默认值。

## 基础选项

```toml title="novel.toml"
# 站点标题,显示在导航栏和页面标题中
title = "My Docs"

# 站点描述,用于 meta 标签
description = "Documentation for my project"

# 文档根目录(相对于项目根目录)
root = "docs"

# 构建输出目录
out_dir = "dist"

# 基础 URL 路径 —— 部署到子路径时设置
# 例如 "/docs/" 对应 https://example.com/docs/
base = "/"

# 默认语言(用于 <html lang="...">)
lang = "zh"

# 完整站点 URL —— 启用 sitemap.xml 和 feed.xml 生成
site_url = "https://example.com"

# Logo 图片路径(显示在导航栏)
logo = "/logo.svg"

# Favicon 路径
icon = "/favicon.ico"

# 移除 URL 中的 .html 扩展名
clean_urls = false
```

## Markdown 选项

```toml title="novel.toml"
[markdown]
# 默认在所有代码块上显示行号
show_line_numbers = false

# 默认对长代码行进行换行
default_wrap_code = false

# 构建时检查内部死链
check_dead_links = false
```

## 主题选项

```toml title="novel.toml"
[theme]
# 启用深色/浅色模式切换
dark_mode = true

# 页脚文本(支持 HTML)
footer = "Built with Novel | Apache 2.0 License"

# 在页面上显示 git 最后更新时间
last_updated = true

# 最后更新时间的自定义文本
last_updated_text = "最后更新"

# "编辑此页面"链接模板
# 页面的相对文件路径会被追加到该 URL 后
edit_link = "https://github.com/user/repo/edit/main/docs/"

# 编辑链接的自定义文本
edit_link_text = "在 GitHub 上编辑此页"

# 源码仓库链接(在导航栏显示 GitHub 图标)
source_link = "https://github.com/user/repo"
```

### 导航

默认情况下,导航会根据文档目录下的顶级目录自动生成。如需自定义:

```toml title="novel.toml"
[[theme.nav]]
text = "指南"
link = "/zh/guide/"

[[theme.nav]]
text = "API"
link = "/api/"

[[theme.nav]]
text = "Blog"
link = "https://blog.example.com"
```

### 侧边栏

默认情况下,侧边栏会根据目录结构和 `_meta.json` 文件自动生成。如需自定义:

```toml title="novel.toml"
# /guide/ 下页面的侧边栏
[[theme.sidebar."/guide/"]]
type = "link"
text = "快速开始"
link = "/guide/getting-started"

[[theme.sidebar."/guide/"]]
type = "link"
text = "配置"
link = "/guide/configuration"
```

### 社交链接

```toml title="novel.toml"
[[theme.social_links]]
icon = "GitHub"
link = "https://github.com/user/repo"

[[theme.social_links]]
icon = "Twitter"
link = "https://twitter.com/user"
```

### 公告横幅

在每个页面顶部显示公告横幅:

```toml title="novel.toml"
[theme.banner]
text = "Novel v1.0 正式发布!"
link = "/guide/getting-started"
dismissible = true
```

## 通用 SSG 选项

Novel 除了文档,也可以用来构建博客和内容站点。以下部分都是可选的,默认关闭。完整参考见[通用 SSG 模式](./general-ssg)。

```toml
[content]
drafts = false                 # 是否包含 draft: true 的页面
future = false                 # 是否包含日期晚于今天的页面
summary_separator = "<!-- more -->"

[pagination]
page_path = "page"             # /posts/page/2/
first_page_in_root = true      # 首页位于 /posts/

[taxonomies.tags]
name = "Tags"

[taxonomies.categories]
name = "Categories"

# 主题包: 额外的模板加载目录
[theme]
pack = "./themes/midnight"

# SCSS(需要 `sass` cargo feature)
[sass]
entries    = [["assets/scss/main.scss", "assets/css/main.css"]]
load_paths = ["assets/scss"]

# 图片缩放(需要 `images` cargo feature)
[images]
sizes   = [400, 800, 1600]
quality = 82
```

## 国际化(i18n)

Novel 原生支持多语言文档。通过 `[i18n]` 配置各个语言的目录与元数据:

```toml title="novel.toml"
[i18n]
default_locale = "en"

[[i18n.locales]]
code = "en"
name = "English"
dir  = "en"
title = "Novel"
description = "Fast static documentation site generator"

[[i18n.locales]]
code = "zh"
name = "简体中文"
dir  = "zh"
title = "Novel"
description = "Rust 编写的极速静态文档站点生成器"

# 可选:某个语言的导航/侧边栏/页脚覆盖
[i18n.locales.theme]
footer = "由 Novel 构建 | Apache 2.0 许可证"
```

此时每个语言的文档会被读取自 `docs/<dir>/`,输出到 `/<code>/...` 路径下。根路径会生成一个基于 `navigator.language` 的自动跳转页面。

## 文档版本化

```toml
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

路由和版本选择器行为见[文档版本化](./versioning)。

## AI 与 Markdown 镜像

```toml
[markdown_mirror]
enabled = true
strip_frontmatter = true
```

CLI 默认启用 `LlmsTxtPlugin` 和 `MarkdownMirrorPlugin`。详见 [AI 上下文文件](./llms)。

## 离线 / PWA

```toml
[pwa]
enabled = false
name = "My Docs"
short_name = "Docs"
theme_color = "#3b82f6"
background_color = "#ffffff"
display = "standalone"
cache_search_index = true
```

详见[离线 / PWA](./offline)。

## 页面反馈

```toml
[feedback]
enabled = false
question = "这个页面有帮助吗?"
positive_text = "有"
negative_text = "没有"
thanks_text = "感谢反馈。"
positive_link = "https://github.com/user/repo/discussions"
negative_link = "https://github.com/user/repo/issues/new"
```

详见[页面反馈](./feedback)。

## 自定义模板

Novel 会优先在项目根目录下的 `templates/` 文件夹中查找模板,然后才回退到内置的默认模板。你只需覆盖那些你想改动的文件;缺失的模板仍会使用默认版本。

```text
templates/
  base.html
  doc.html
  home.html
  404.html
```

完整说明见[主题与外观](./theming.md)。

## 完整示例

```toml title="novel.toml"
title = "My Project"
description = "Documentation for My Project"
root = "docs"
out_dir = "dist"
base = "/"
lang = "zh"
site_url = "https://my-project.dev"
logo = "/logo.svg"
icon = "/favicon.ico"

[markdown]
show_line_numbers = false
check_dead_links = true

[theme]
dark_mode = true
footer = "Copyright 2025 My Project"
last_updated = true
edit_link = "https://github.com/user/my-project/edit/main/docs/"
source_link = "https://github.com/user/my-project"

[[theme.nav]]
text = "指南"
link = "/zh/guide/"

[[theme.nav]]
text = "API"
link = "/api/"

[theme.banner]
text = "v1.0 正式发布!"
link = "/guide/getting-started"
dismissible = true
```
