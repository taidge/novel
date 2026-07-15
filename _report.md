# Novel 主线升级与安全审计任务报告

审计日期：2026-07-15

工作区：`D:\Works\taidge\novel`

目标分支：`main`

升级目标：Salvo `0.94.0`，Rust MSRV `1.94`

## 1. 执行摘要

本轮工作先把 2026-07-07 已完成但分散在多个分支的审计成果合入本地
`main`，再以合并后的主线为基线重新执行代码、依赖、安全、性能和功能审计。

当前基线能够通过格式化、全特性 Clippy 和全特性测试，但依赖锁中仍有 8 个
`cargo audit` 漏洞、1 个 `cargo deny` 报告的 unsound advisory、1 个被 yank
的传递依赖，以及若干输入校验和替代模板引擎安全边界问题。本轮应优先完成依赖
链升级和“解析失败即停止”的正确性修复，然后再做性能、提示和文档改进。

本报告是本轮实现的任务清单。报告先独立提交，后续改动按阶段提交。

## 2. 近期更新合入确认

开始时 `main` 与 `origin/main` 都位于 `1af39c1`。以下近期成果已合入本地
`main`：

| 来源 | 主要内容 | 状态 |
| --- | --- | --- |
| `chris/docs-implementation-drift` | 静态站点安全边界、lint 修复、审计文档校正 | 已合入 |
| `chris/security-audit-fixes` | 安全复核反馈 | 已合入 |
| `chris/dev-watcher-coverage` | dev watcher 覆盖模板、样式、数据和静态资源 | 已合入 |
| `chris/static-asset-link-checks` | 静态资产链接检查 | 已合入 |
| `chris/frontmatter-date-validation` | frontmatter 日期校验 | 已合入 |
| `chris/ci-audit-gates` | 全特性 CI 检查矩阵 | 已合入 |
| `chris/head-tag-validation` | 自定义 head 名称初步校验 | 已合入并保留日期校验 |

合并后的主线为 `2fdd03e`，相对 `origin/main` 超前 15 个提交。原工作区的 6 个
未提交文档修正已保存在 Git stash 中，待本报告提交后恢复并纳入文档阶段。

## 3. 基线验证结果

| 检查 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 通过 |
| `cargo test --workspace --all-features` | 通过；测试清单共 78 项 |
| `cargo run -p novel-cli -- check` | 通过 |
| 当前本机工具链 | `rustc 1.97.0` / `cargo 1.97.0` |
| Manifest MSRV | 未声明 |
| 当前 Salvo | Manifest `0.77`，锁定 `0.77.1` |
| `cargo audit` | 失败；8 个漏洞，另有未维护依赖警告 |
| `cargo deny check advisories` | 失败；另检出 `anyhow 1.0.102` unsound advisory |

Salvo `0.94.0` 的 crate 元数据声明 `rust-version = 1.94`，与本轮要求一致。
Rust 1.94.0 已于 2026-03-05 发布，因此将 `rust-version = "1.94"` 作为 workspace
MSRV，并在 CI 中加入 1.94 工具链验证。

## 4. 依赖与漏洞审计

### P0：必须在本轮封堵

| Advisory | 依赖链/影响 | 计划 |
| --- | --- | --- |
| RUSTSEC-2026-0190 | `anyhow 1.0.102` 的 `downcast_mut` unsoundness | 更新到已修复版本 |
| RUSTSEC-2026-0204 | `crossbeam-epoch 0.9.18` 无效指针解引用 | 刷新锁文件到 `>=0.9.20` |
| RUSTSEC-2026-0194 / 0195 | `syntect -> plist -> quick-xml 0.38.4` CPU/内存 DoS | 关闭 Syntect 不需要的 plist/yaml loader 特性 |
| RUSTSEC-2026-0185 | Salvo 旧依赖链中的 `quinn-proto 0.11.14` 远程内存耗尽 | 升级 Salvo，并关闭本地 HTTP 服务不需要的默认特性 |
| RUSTSEC-2026-0049 / 0098 / 0099 / 0104 | Salvo 旧 TLS 链中的 `rustls-webpki 0.103.9` | 升级 Salvo并缩小默认特性 |
| yanked `spin 0.9.8` | `salvo_core -> multer` | 由 Salvo/锁文件升级移除 |

### 残余信息性风险

- `syntect` 的预编译默认 syntax/theme 数据仍需要 `bincode 1.3.3`。
  RUSTSEC-2025-0141 将其标为“停止维护”，但没有已知漏洞或安全升级版本。
  本轮先移除可避免的 `yaml-rust`、`plist` 和 `quick-xml` 链；保留
  `bincode` 并在最终结果中明确记录。
- 不加入无依据的 advisory ignore；CI 使用 `cargo audit` 阻断真实漏洞，
  信息性 warning 保持可见。

## 5. 代码审计发现

### P0：发布保护可能静默失效

Markdown frontmatter 的 YAML 解析或类型反序列化失败时，当前代码会静默使用
`FrontMatter::default()`。Typst frontmatter 也有相同行为。这可能让
`draft`、`noindex`、`expires_at`、redirect 等字段在配置错误时被忽略，
最终发布本应隐藏或跳转的页面。

计划：

- Markdown 和 Typst frontmatter 解析/反序列化失败时返回带文件路径的错误。
- Typst 检测到起始分隔符但缺少结束分隔符时失败。
- 增加 malformed YAML、错误字段类型和未闭合 frontmatter 测试。

### P1：自定义 head 仍可执行任意脚本

现有校验只检查标签/属性名称格式并拒绝 `on*` 属性，但仍允许 `script`、
`style`、`base`、`meta http-equiv=refresh` 等高风险组合。页面内容若来自
非完全可信贡献者，可借此执行脚本、改变 URL 基准或强制跳转。

计划：

- 默认仅允许 SEO/资源提示所需的 `meta`、`link`、`title`。
- 为每类标签设置属性 allowlist，拒绝 `http-equiv`、事件属性和不匹配属性。
- 限制 void 标签内容并校验 `href` 的危险 scheme。
- 更新中英文文档，明确 Markdown 原始 HTML和自定义模板仍属于 trusted input。

### P1：Tera/Handlebars 首页卡片存在内联 JavaScript 注入面

MiniJinja 已用普通 `<a href>` 渲染可点击 feature card，但 Tera 和 Handlebars
仍把 frontmatter URL 拼入 `onclick="window.location='...'"`。HTML 转义不能
可靠地构成 JavaScript 上下文转义。

计划：

- 三套内置模板统一使用语义化链接，不把数据拼入内联 JavaScript。
- 增加替代模板引擎渲染测试，覆盖恶意引号/脚本样例。

### P1：合并后的重复校验

日期校验与 head 校验合并冲突解决后，Markdown 和 Typst 入口各调用了两次日期
校验。结果正确但浪费工作并掩盖组合逻辑，需去重并用组合测试固定。

### P1：CI 缺少 MSRV 与持续漏洞门禁

当前 CI 使用 stable，不能证明 Rust 1.94 可编译；也不会在新 advisory 发布后
自动阻断。

计划：

- 所有 workspace package 继承 `rust-version = "1.94"`。
- CI 加入 Rust 1.94 的 `cargo check --locked --workspace --all-targets --all-features`。
- 使用 RustSec 官方 `rustsec/audit-check@v2.0.0` 检查 `Cargo.lock`。

### P2：本地服务器安全提示和路径错误处理

- `dev` / `preview` 允许绑定非 loopback 地址，但没有提示生成站点会暴露到
  局域网/公网。
- 非 UTF-8 输出目录会静默回退到相对 `dist`，存在服务错误目录的风险。

计划：

- 绑定非 loopback host 时输出醒目的安全 warning。
- 非 UTF-8 路径显式报错，不再回退。
- 增加 host 分类和路径错误测试；保持默认 `127.0.0.1`。

### P2：数据和嵌入式服务错误被吞掉

- `TemplateEngine::new` 加载 `docs/data` 失败时静默返回空对象，可能让站点
  在数据损坏后仍“成功”构建。
- `BuiltSite::into_salvo_router` 忽略写临时目录错误，且临时目录只按 PID 命名，
  同一进程多站点可能互相覆盖。

计划：

- 数据文件解析失败时让构建失败并报告具体文件。
- 为 Salvo 集成增加 fallible API和进程内唯一临时目录；保留兼容入口时明确其
  行为，避免无提示失败。

### P3：小型性能与维护改进

- `parse_file_embed` 每次解析代码围栏都会重新编译同一正则。
- Salvo 默认特性远超本地静态 HTTP/SSE 所需，增加编译时间和供应链面积。

计划：

- 将 file embed 正则移入 `LazyLock`。
- Salvo 使用 `default-features = false`，只启用 HTTP/1 server、静态文件和 SSE
  所需特性；以实际编译结果校正最小特性集。

## 6. 分阶段实施与提交计划

1. **报告提交**：仅提交本文件。
2. **工具链、依赖和 CI**：Salvo 0.94.0、Rust MSRV 1.94、最小依赖特性、锁文件、
   MSRV/audit CI。
3. **输入安全和正确性**：frontmatter fail-closed、head allowlist、重复校验去除、
   数据加载错误传播。
4. **模板、服务器和性能**：替代模板引擎注入修复、公开绑定提示、路径错误处理、
   Salvo fallible 集成、正则缓存。
5. **文档与最终验证**：恢复已保护的文档修正，更新中英文安全/MSRV说明，执行
   完整验证后提交。

若实现过程中发现某项需要破坏公开 API或依赖上游未提供能力，将在最终报告中
保留为明确的 residual item，不以静默绕过代替修复。

## 7. 验收标准

- `cargo fmt --all -- --check`
- `cargo check --locked --workspace --all-targets --all-features`
- Rust 1.94 工具链执行同等 workspace check
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo run -p novel-cli -- check`
- `cargo audit` 不报告已知 vulnerability
- `cargo tree` 不再包含 Salvo 0.77、`yaml-rust`、`quick-xml 0.38`、
  `rustls-webpki 0.103.9` 或 `spin 0.9.8`
- 工作区无未解释的未提交改动；每阶段形成独立 Git 提交

## 8. 实施结果（2026-07-15）

### 已完成

- [x] 近期 7 条审计/修复分支全部合入本地 `main`。
- [x] Salvo 升级到 `0.94.0`，关闭默认特性，仅保留 HTTP/1 server、静态文件和
  SSE 所需能力。
- [x] workspace 及全部 package 声明 `rust-version = "1.94"`，Rust 1.94.0
  本地全特性检查通过。
- [x] CI 增加 Rust 1.94 MSRV job 和 RustSec `audit-check@v2.0.0` 门禁。
- [x] 8 个 `cargo audit` 漏洞、`anyhow` unsound advisory 和 yanked
  `spin 0.9.8` 均已从锁文件移除。
- [x] Syntect 关闭不需要的 YAML/plist loader，移除 `yaml-rust`、
  `plist` 和有漏洞的 `quick-xml 0.38`。
- [x] Markdown/Typst frontmatter 改为 fail-closed；错误 YAML、错误字段类型和
  未闭合 Typst frontmatter 均带来源文件报错。
- [x] 自定义 head 改为 `meta` / `link` / `title` 及标签专属属性
  allowlist；结构化 frontmatter URL 拒绝危险 scheme。
- [x] 数据目录遍历和 TOML/JSON 解析错误不再被静默吞掉。
- [x] Tera/Handlebars feature card 不再拼接内联 JavaScript；Handlebars icon
  不再使用未转义输出。
- [x] 修复 Tera 子模板可能先于 `base.html` 注册而导致引擎无法启动的问题，
  并保留完整的嵌套错误原因。
- [x] `dev` / `preview` 对非 loopback 绑定输出安全警告，正确处理 IPv6，
  非 UTF-8 静态目录不再回退到错误的 `dist`。
- [x] Salvo 嵌入 API 改为返回 `Result`，写入失败不再被忽略；同进程多个站点
  使用唯一临时目录。
- [x] file embed 正则和 syntax/theme 初始化改用 `LazyLock`，移除直接
  `once_cell` 依赖；Tokio 从 `full` 缩减为实际使用特性。
- [x] README、部署文档和 `novel init` 脚手架改用 GitHub `--locked` 安装。

### 审计中新增的重要发现

crates.io 上的 `novel-cli 0.17.1` 属于另一个小说下载项目。因此
`cargo install novel-cli` 会安装错误软件，而且本项目无法直接以同名 package
发布。本轮已删除所有该安装指令并加入明确提示；若要发布到 crates.io，后续需要
产品层决定新的 package 名称，二进制名仍可保持 `novel`。

### 最终验证

| 检查 | 最终结果 |
| --- | --- |
| Rust 1.94.0 `cargo check --locked --workspace --all-targets --all-features` | 通过 |
| stable `cargo check --locked --workspace --all-targets --all-features` | 通过 |
| `cargo fmt --all -- --check` | 通过 |
| 全特性 Clippy `-D warnings` | 通过 |
| 全特性测试 | CLI 10、core 76、shared 2、doctest 2 全部通过；2 个示例 doctest ignored |
| `cargo run -p novel-cli -- check` | 通过 |
| `cargo audit` | 0 vulnerability；1 个允许的信息性 warning |

### 保留项

- `syntect 5.3.0` 的预编译默认 syntax/theme 仍通过 `bincode 1.3.3`
  加载。RUSTSEC-2025-0141 仅标记其停止维护，目前没有已知漏洞或可直接替换的
  安全升级；`cargo audit` 保持该 warning 可见。
- crates.io 发布需要先决定不冲突的新 package 名称，并按
  `novel-shared -> novel-core -> CLI` 顺序建立发布链。本轮不擅自更名。
