# 库 API

Novel 可以作为 CLI 独立运行,也可以作为库嵌入到你自己的 Rust 应用中 —— 无论是 Web 服务器、构建工具还是其他任何地方。

## 添加依赖

```toml title="Cargo.toml"
[dependencies]
novel-core = { path = "path/to/novel/crates/novel-core" }
```

## 快速开始

三行代码即可构建一个文档站点并写入磁盘:

```rust
let site = novel_core::Novel::new("docs").build()?;
site.write_to("dist")?;
```

## 从 `novel.toml` 加载

如果你有一个带 `novel.toml` 配置文件的项目:

```rust
let site = novel_core::Novel::load(".")?.build()?;
site.write_to_default_output()?;
```

## Builder API

通过 builder 模式以编程方式自定义站点:

```rust
use novel_core::Novel;

let site = Novel::new("docs")
    .title("My API Reference")
    .description("Generated docs for my-crate")
    .base("/docs/")
    .site_url("https://example.com")
    .with_theme(|t| {
        t.dark_mode = true;
        t.footer = Some("Built with Novel".into());
        t.last_updated = true;
    })
    .build()?;

site.write_to("./output")?;
```

可用的 builder 方法:

| 方法 | 描述 |
|------|------|
| `title()` | 站点标题 |
| `description()` | 站点描述 |
| `base()` | 基础 URL 路径(例如 `"/docs/"`) |
| `lang()` | 语言代码(默认 `"en"`) |
| `out_dir()` | 输出目录名 |
| `site_url()` | 用于 sitemap / RSS 的完整 URL |
| `theme()` | 替换主题配置 |
| `with_theme()` | 通过闭包修改主题配置 |
| `config()` | 替换整个 `SiteConfig` |
| `project_root()` | 覆盖用于文件嵌入的项目根目录 |

## `BuiltSite`

调用 `.build()` 会返回一个 `BuiltSite`,它持有所有已处理的页面,并可以按需渲染 HTML。

### 访问页面

```rust
let site = Novel::new("docs").build()?;

// 遍历所有页面
for page in site.pages() {
    println!("{}: {}", page.route.route_path, page.title);
}

// 查找某个具体页面
if let Some(page) = site.page("/guide/intro") {
    println!("Found: {}", page.title);
}
```

### 渲染单个页面

```rust
let site = Novel::new("docs").build()?;

// 将一个页面渲染为完整的 HTML 字符串
let page = site.page("/guide/intro").unwrap();
let html = site.render_page(page)?;

// 渲染 404 页面
let not_found_html = site.render_404()?;
```

### 静态资源

```rust
let css: &str = site.css();   // 样式表
let js: &str  = site.js();    // 客户端脚本
```

### 生成的数据

```rust
// 搜索索引(JSON)
let json: String = site.search_index_json()?;

// sitemap XML(未设置 site_url 时为 None)
if let Some(xml) = site.sitemap_xml() {
    std::fs::write("sitemap.xml", xml)?;
}

// Atom/RSS 订阅源(未设置 site_url 时为 None)
if let Some(xml) = site.feed_xml() {
    std::fs::write("feed.xml", xml)?;
}
```

## 嵌入到 Web 服务器

以下是一个使用 Axum 的最小示例:

```rust title="server.rs"
use axum::{Router, routing::get, extract::Path, response::Html};

#[tokio::main]
async fn main() {
    // 在启动时构建一次
    let site = novel_core::Novel::new("docs")
        .title("My App Docs")
        .build()
        .expect("failed to build docs");

    // 提供页面
    let app = Router::new()
        .route("/docs/*path", get(move |Path(path): Path<String>| {
            let route = format!("/{}", path);
            let html = site.page(&route)
                .and_then(|p| site.render_page(p).ok())
                .unwrap_or_else(|| site.render_404().unwrap());
            async move { Html(html) }
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

::: tip
`BuiltSite` 结构体是 `Send` 但不是 `Sync`(由于模板引擎的限制)。如果你需要在多个处理器之间共享访问,请将它包装在 `Arc<BuiltSite>` 中,或者在热更新场景下按请求重新构建。
:::

## 工作流对比

::: tabs
== CLI(独立)

```bash
novel build
novel dev
novel preview
```

== 库(嵌入)

```rust
let site = Novel::new("docs").build()?;
site.write_to("dist")?;

// 或者直接从内存中提供服务
let html = site.render_page(page)?;
```
:::
