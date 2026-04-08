# Markdown 功能

Novel 支持标准 Markdown,并带有 GitHub Flavored Markdown (GFM) 扩展。

## 标题

使用 `#` 到 `######` 来定义标题。每个标题都会自动获得一个锚点链接。

## 强调

- **粗体** 用 `**text**`
- *斜体* 用 `*text*`
- ~~删除线~~ 用 `~~text~~`

## 链接

- [内部链接](/guide/getting-started)
- [外部链接](https://github.com)(自动在新标签页打开)

## 列表

无序列表:
- 项目一
- 项目二
  - 嵌套项

有序列表:
1. 第一
2. 第二
3. 第三

## 任务列表

- [x] Markdown 解析
- [x] 语法高亮
- [x] 文件嵌入
- [ ] 统治世界

## 表格

| 功能 | 描述 | 状态 |
|---------|-------------|--------|
| GFM | GitHub Flavored Markdown | 完成 |
| 高亮 | 代码块语法高亮 | 完成 |
| 容器 | tip / warning / danger / info / note / details | 完成 |
| 文件嵌入 | 在代码块中嵌入外部文件 | 完成 |
| 选项卡 | 选项卡式内容块 | 完成 |
| 步骤 | 带编号的步骤指南 | 完成 |
| 徽章 | 行内徽章 | 完成 |

## 引用块

> 文档是你写给未来自己的情书。
> —— Damian Conway

## 代码块

行内代码: `let x = 42;`

带语法高亮的代码块:

```rust title="example.rs"
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### 行号

在代码围栏上添加 `showLineNumbers` 以显示行号:

````markdown
```rust showLineNumbers
fn main() {
    println!("Hello!");
}
```
````

或者在 `novel.toml` 中全局启用:

```toml
[markdown]
show_line_numbers = true
```

### 行高亮

用 `{1,3-5}` 高亮指定行:

````markdown
```rust {1,4-5}
use std::io;

fn main() {
    println!("Hello!");
    println!("World!");
}
```
````

### Diff 显示

使用 `diff` 语言来展示新增和删除的行:

```diff
- let old_value = 1;
+ let new_value = 2;
  let unchanged = 3;
```

## 容器指令

::: tip
这是给读者的一条有用提示。
:::

::: warning
请留意这条警告。
:::

::: danger
该操作不可逆!
:::

::: info
这里是一些补充信息。
:::

::: note
一条备注以供参考。
:::

::: details 点击展开
默认隐藏的内容,点击后可展开。

你可以在这里放任意 Markdown 内容,例如:
- 列表
- **粗体**
- 代码: `let x = 1;`
:::

## 选项卡

使用 `::: tabs` 和 `== 选项卡标题` 来分组相关内容:

::: tabs
== npm

```bash
npm install my-package
```

== yarn

```bash
yarn add my-package
```

== pnpm

```bash
pnpm add my-package
```
:::

语法:

````markdown
::: tabs
== 第一个选项卡

第一个选项卡的内容。

== 第二个选项卡

第二个选项卡的内容。
:::
````

## 步骤

使用 `::: steps` 创建带编号的分步指南:

::: steps

### 安装 Rust

从 [rustup.rs](https://rustup.rs) 下载并安装 Rust。

### 创建项目

```bash
novel init my-docs
```

### 开始写作

在 `docs/` 目录中添加 `.md` 文件。

:::

语法:

````markdown
::: steps

### 第一步

描述内容。

### 第二步

描述内容。

:::
````

## 徽章

使用 `{badge:TYPE|TEXT}` 添加行内徽章:

- 这是 v0.2 中的 {badge:tip|新增}
- 该功能当前为 {badge:warning|实验性}
- 此 API 已 {badge:danger|废弃}
- 状态: {badge:info|稳定}

语法: `{badge:tip|文本}` —— 支持的类型: `tip`、`info`、`warning`、`danger`、`note`。

## 图片

图片会被懒加载,并支持点击放大:

```markdown
![替代文本](./image.png)
```

## 水平分隔线

---

## HTML

Markdown 无法满足需求时,支持直接使用行内 HTML。
