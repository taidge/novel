# AI 上下文文件

Novel 会在 `novel build` 期间生成适合 AI 工具读取的文档上下文文件。

## 生成的文件

CLI 默认启用 `LlmsTxtPlugin` 和 `MarkdownMirrorPlugin`,并写出:

- `llms.txt` — 公共文档页面的精简 Markdown 地图
- `llms-full.txt` — 单文件形式的完整公共文档内容
- `.well-known/llms.txt` — 兼容性副本
- `.well-known/llms-full.txt` — 兼容性副本
- 每个公共页面对应的 `.md` 镜像,例如 `/guide/getting-started.md`

当用户把你的文档加入 AI 编程工具的上下文,或爬虫需要避开站点外壳、JavaScript 和渲染 HTML 时,这些文件能提供更干净的入口。

## 页面过滤

Novel 会排除不应作为公共文档暴露的页面:

- 设置了 `noindex: true` 的页面
- 跳转到其他地址的页面
- 生成的 404 页面

草稿页和未来日期页面仍由正常内容管线过滤;只有使用 `--drafts` 或 `--future` 构建时才会包含。

## 完整上下文来源

对于基于文件系统的构建,`llms-full.txt` 会优先读取原始 Markdown,并在写入正文前移除 frontmatter。这样可以保留代码围栏和作者编写的 Markdown。如果无法取得源 Markdown,Novel 会退回到从渲染 HTML 中提取纯文本。

## 自定义文件

如果你想覆盖生成结果,可以在文档目录中放置自己的 `llms.txt`、`llms-full.txt` 或 `.well-known/llms.txt` 文件。静态资源复制发生在插件输出之后,因此你的自定义文件会覆盖自动生成文件。

## Markdown 镜像

Markdown 镜像默认启用:

```toml
[markdown_mirror]
enabled = true
strip_frontmatter = true
```

如果只需要聚合的 `llms.txt` 文件,可以设置 `enabled = false`。启用后,文档页脚也会显示 `Markdown` 链接。

## 库用法

把 Novel 作为库使用时,注册插件即可:

```rust
use novel_core::plugins::LlmsTxtPlugin;
use novel_core::plugins::MarkdownMirrorPlugin;

let site = novel_core::DirNovel::new("docs")
    .plugin(LlmsTxtPlugin)
    .plugin(MarkdownMirrorPlugin)
    .build()?;
```

也可以直接从已构建站点生成字符串:

```rust
let llms = site.llms_txt();
let full = site.llms_full_txt();
```
