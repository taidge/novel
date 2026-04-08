# 静态资源

Novel 会自动处理文档目录中的静态资源(图片、字体、PDF 等)。

## 工作原理

在 `novel build` 期间,你 `docs/` 目录中任何非 Markdown 文件都会被原样复制到输出中,目录结构会被保留。

```
docs/
├── index.md
├── logo.png              # → dist/logo.png
├── guide/
│   ├── getting-started.md
│   └── screenshot.png    # → dist/guide/screenshot.png
└── assets/
    ├── diagram.svg       # → dist/assets/diagram.svg
    └── sample.pdf        # → dist/assets/sample.pdf
```

## 引用资源

### 从 Markdown 中引用

使用标准的 Markdown 图片语法,配合相对路径:

```markdown
![Screenshot](./screenshot.png)
```

或者从文档根目录引用:

```markdown
![Logo](/logo.png)
```

### 图片

Markdown 中的图片会自动:
- **懒加载** —— 带上 `loading="lazy"` 属性以提升性能
- **可缩放** —— 点击后以覆盖层形式查看全尺寸

### Favicon 和 Logo

在 `novel.toml` 中设置 favicon 和 logo:

```toml title="novel.toml"
icon = "/favicon.ico"
logo = "/logo.svg"
```

将这些文件放到你的 `docs/` 目录中。

## 内置资源

Novel 会自动在 `dist/assets/` 下生成以下资源:

| 文件 | 描述 |
|------|------|
| `style.css` | 站点样式表 |
| `main.js` | 客户端 JavaScript(搜索、主题切换等) |
| `search-index.json` | 客户端搜索的搜索索引 |

## 被排除的文件

以下文件不会被复制:
- `.md` 文件(作为页面处理) 
- `_meta.json` 文件(用于侧边栏排序,不会被发布)
