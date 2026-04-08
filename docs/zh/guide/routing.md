# 路由

Novel 使用基于文件的路由。`docs/` 目录下的每个 `.md` 文件都会自动成为一个页面。

## 约定

| 文件路径 | 路由 URL |
|-----------|-----------|
| `docs/index.md` | `/` |
| `docs/guide/index.md` | `/guide/` |
| `docs/guide/getting-started.md` | `/guide/getting-started` |
| `docs/api/reference.md` | `/api/reference` |

映射规则如下:

- `index.md` 文件映射为所在目录的路径(带尾随斜杠)
- 其他 `.md` 文件映射为其文件名(不带扩展名)
- 嵌套目录产生嵌套的 URL 路径
- 默认按字母顺序排序;可以用 `_meta.json` 自定义

## 目录结构

一个典型的文档目录如下:

```
docs/
├── index.md              # → /
├── guide/
│   ├── _meta.json        # /guide/ 的侧边栏顺序
│   ├── index.md          # → /guide/
│   ├── getting-started.md # → /guide/getting-started
│   └── configuration.md   # → /guide/configuration
└── api/
    ├── _meta.json        # /api/ 的侧边栏顺序
    └── reference.md      # → /api/reference
```

## 使用 `_meta.json` 控制侧边栏顺序

默认情况下,侧边栏项按字母顺序排序。在任意目录中创建 `_meta.json` 文件可以控制顺序:

```json title="docs/guide/_meta.json"
["getting-started", "configuration", "markdown", "deploy"]
```

每一项是不带 `.md` 扩展名的文件名。越靠前的页面在侧边栏中越靠前。

### 进阶 `_meta.json`

你也可以使用对象项来自定义标签和分组:

```json title="docs/guide/_meta.json"
[
  "getting-started",
  {
    "text": "编写内容",
    "collapsed": false,
    "items": ["markdown", "file-embed"]
  },
  {
    "text": "进阶",
    "collapsed": true,
    "items": ["configuration", "deploy"]
  }
]
```

## 导航

顶级目录会自动成为导航栏中的导航链接。例如,`guide/` 和 `api/` 两个目录会生成 "Guide" 和 "API" 导航项。

如需自定义导航,参见[配置](/zh/guide/configuration) 指南。

## 首页

根目录的 `index.md` 文件在 frontmatter 中设置 `page_type: home` 时会被当作首页。详情见[首页](/zh/guide/home-page)。

## 静态资源

`docs/` 目录下的非 Markdown 文件(图片、PDF 等)会原样复制到输出目录,保持目录结构。详情见[静态资源](/zh/guide/static-assets)。
