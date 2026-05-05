# 文档版本化

Novel 可以从文档根目录下的子目录构建冻结的文档版本。

## 目录结构

```text
docs/
  next/
    index.md
    guide/getting-started.md
  v1/
    index.md
    guide/getting-started.md
```

## 配置

```toml title="novel.toml"
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

`current` 版本会保留正常的非前缀路由。在上面的例子中,`docs/next/guide/getting-started.md` 会构建到 `/guide/getting-started`,而 `docs/v1/guide/getting-started.md` 会构建到 `/v1/guide/getting-started`。

也可以为任意版本指定路由前缀:

```toml
[[versions.items]]
code = "v1"
label = "1.0"
dir = "v1"
path = "/docs/1.0"
```

## 版本选择器

当多个版本中存在相同相对路径的文件时,Novel 会在导航栏添加版本选择器。选择其他版本会跳转到该版本的对应页面。

## 链接

对于归档版本,Markdown 中的绝对内部链接会自动改写到对应版本前缀。例如 `v1` 源文件里的 `/guide/configuration` 会变成 `/v1/guide/configuration`。
