# Novel 项目修复与复核统一报告

审计与修复日期：2026-07-07  
工作区：`D:\Works\taidge\novel`  
报告目标：根据上一版审计报告修复可落地问题，并对每个 item 标记是否已 fixed。  
PR 状态：基础修复已提交到 draft PR #9 `Fix static site security boundaries`，URL: https://github.com/taidge/novel/pull/9。未完成 checklist item 会拆成独立 stacked PR，base 指向 #9 分支，避免重复包含 #9 的大 diff。
当前结论：项目已从“可运行 MVP / 内测 beta”推进到“更接近公开 beta”的状态。核心构建、测试、clippy、文档站点 build/check 已通过；crates.io 安装链路仍未完全闭环，因为 `novel-core`/`novel-cli` 的完整 package verify 需要先在 crates.io 发布内部依赖。

## 1. 当前完成度

综合完成度：约 78%。

已明显改善：

- 自带文档站点 `novel check` 已清零。
- 默认构建不再静默跳过读取/处理失败的页面。
- `summary_separator`、`list_layout`、重复 heading id、静态内部文件过滤已接通或修复。
- 默认 MiniJinja 模板和主要插件已加入 base URL helper，内部静态配置文件不再复制到 `dist/`。
- 发布 metadata 和内部 path dependency version 已补齐。

仍未完成：

- `cargo package -p novel-core` / `cargo package -p novel-cli` 仍不能完整通过，因为依赖 crate 尚未发布到 crates.io。
- i18n + general SSG 的 collection/taxonomy/archive 组合模型仍未完整修复。
- Tera/Handlebars 模板引擎 parity、dev watcher 覆盖、日期解析校验、静态资产链接检查仍是 gaps。
- CI 配置本身尚未更新为执行本报告中的完整验证矩阵。

## 2. 可审核维度

| 维度 | 关注点 | 当前状态 |
| --- | --- | --- |
| 产品范围与完成度 | README/docs 承诺是否闭环 | 部分改善，安装发布仍未完全闭环 |
| 构建与 CLI 工作流 | build/dev/check/clean/init/new | build/check 已通过，dev watcher 仍有缺口 |
| 核心 SSG 正确性 | 路由、base、输出路径、资产、sitemap/feed | 主线改善，base 缺少完整 E2E fixture |
| 内容模型 | frontmatter、collections、taxonomy、archive、series | 单语主线可用，i18n 组合仍未完整 |
| i18n / versioning | locale/version 路由、alternate、跳转 | base root redirect 已修，collection 组合未完 |
| 模板与主题 | MiniJinja/Tera/Handlebars/helper parity | MiniJinja 改善，替代引擎仍落后 |
| 文档质量 | description、安装说明、配置说明 | description 已清零，安装说明仍受发布链路影响 |
| 安全与信任边界 | 路径穿越、HTML/JS/XML 注入、内部文件泄露 | 内部文件泄露和 home inline JS 已修，trust boundary 需文档化 |
| 测试质量 | 单元、集成、端到端、跨 feature | 单元增加并通过，E2E fixture 仍不足 |
| CI / Release / Packaging | package、publish、release、安装脚本 | shared 可 package，core/cli 等待依赖发布顺序 |
| 性能与可维护性 | 并行构建、失败策略、依赖体量 | 失败策略改善，依赖治理未做 |
| 开发者体验 | live reload、错误信息、示例 | 错误失败更明确，dev watcher 未修 |

## 3. 修复状态 Checklist

- [ ] 5.1 `cargo install novel-cli` / crates.io 安装链路完全闭环  
  部分修复：manifest metadata、workspace repository/homepage、内部依赖 `version = "0.1.0"` 已补齐。未勾选原因：`cargo package -p novel-core --allow-dirty` 仍因 crates.io 上没有 `novel-shared 0.1.0` 失败；`novel-cli` 同理等待 `novel-core 0.1.0`。

- [x] 5.2 自带文档站点通过自己的 `novel check`  
  已为 42 个缺少 description 的真实页面补齐 frontmatter；复扫结果 `missing 0`，`cargo run -p novel-cli -- check` exit 0。

- [ ] 5.3 `base` 子路径部署端到端闭环  
  部分修复：新增 URL helper，默认 MiniJinja 模板、sitemap/feed/llms/PWA/robots/markdown mirror、i18n 根跳转、render-time absolute internal links 已处理 base。未勾选原因：尚未建立 `base + i18n + collections + alternate template engine` 的端到端 fixture。

- [ ] 5.4 i18n 与 general SSG 组合完整修复  
  未完成：locale 内 collection/taxonomy/archive/series 的模型仍需设计与 E2E 验证。

- [x] 5.5 构建过程不再静默跳过失败页面  
  已将读取/处理失败聚合为 build error 返回，不再 warn 后继续成功。

- [ ] 5.6 Tera/Handlebars 替代模板引擎 parity  
  未完成：缺少 list/terms 模板和 helper parity 的系统性补齐。

- [x] 6.1 内部数据和控制文件不再作为静态资产发布  
  已排除 `data/`、`_collection.toml`、`_*.toml/json/yaml/yml` 等控制文件；重建后 `dist/` 未发现这类文件。

- [x] 6.2 `summary_separator` 与 `list_layout` 配置接通  
  已将 `content.summary_separator` 传入 MarkdownProcessor；ListPage 已携带 template name 并按 `list_layout` 渲染。

- [x] 6.3 dev server watcher 覆盖模板/CSS/Sass/数据等变更
  已在独立 PR 中修复：dev watcher 会收集 docs、config、project templates、theme pack、custom CSS、Sass entries/load paths，并扩展 rebuild 触发文件类型以覆盖模板、CSS/SCSS、JS、图片、字体和 PDF 等静态资产。

- [ ] 6.4 文档与实现漂移全面清理  
  部分修复：description 缺口已清理。未勾选原因：安装说明、alternate engine 限制、部分配置说明仍需与发布策略同步。

- [x] 6.5 Markdown heading ID 去重  
  已按页面维护 slug counter，重复标题生成 `intro`、`intro-2` 等唯一 id，并新增测试。

- [ ] 6.6 日期字段真实解析与校验  
  未完成。

- [x] 6.7 `check_dead_links` 与静态资产存在性检查语义统一
  已在独立 PR 中修复：Markdown `href`/`src` 内部引用都会被收集；路由引用按页面路由表检查，静态资产引用按公开 assets 集合检查；CLI `novel check` 会报告缺失静态资产，`markdown.check_dead_links = true` 的 build 会在缺失路由或资产时失败。

- [x] 安全项：home feature card 内联 JS 字符串注入面  
  已改为普通 `<a href>` 渲染，不再把 frontmatter URL 写入 inline JS 字符串。

- [ ] 安全项：trusted content/config 边界文档化与 head tag/attr allowlist  
  未完成。

- [ ] CI 项：把 all-features test/clippy、`novel check`、package 验证加入 CI  
  未完成。本次已本地验证，但未修改 CI。

## 4. 本次已完成改进

### 4.1 发布 metadata 与内部依赖

- Workspace 增加 `repository`、`homepage`。
- `novel-shared`、`novel-core`、`novel-cli` 增加 `description`、`license.workspace`、`repository.workspace`、`homepage.workspace`。
- `novel-core -> novel-shared`、`novel-cli -> novel-core/novel-shared` 增加 `version = "0.1.0"`，保留 `path` 用于本地 workspace 开发。

### 4.2 文档质量

- 为英文/中文首页、guide 页面、posts 页面补齐 frontmatter `description`。
- 文档 description 扫描结果从 42 个缺口降为 0。

### 4.3 构建失败策略

- 页面读取失败现在聚合后返回 `NovelError::Build`。
- Markdown/Typst 处理失败现在聚合后返回 `NovelError::Build`。
- 构建不再出现“成功但页面缺失”的静默降级。

### 4.4 Base URL 与 URL helper

- 新增 `join_base_path`、`join_site_url`、特殊 URL 判断。
- MiniJinja 注册 `route_url`、`absolute_url` helper。
- 默认模板、sitemap、feed、llms、PWA、robots、markdown mirror 已改用统一 helper。
- i18n 根跳转使用 base-aware URL。
- 渲染输出阶段对 HTML 中 `href="/..."` / `src="/..."` 做 base rewrite，并避免重复加前缀。

### 4.5 静态资产过滤

- 静态复制规则排除 Markdown/Typst 之外，也排除内部数据与控制文件。
- 重建后 `dist/` 没有 `data/`、`_collection.toml`、`_meta.json`、`_*.toml/json/yaml/yml`。

### 4.6 配置接线

- `content.summary_separator` 已真正影响摘要截断。
- `CollectionConfig.list_layout` 已用于选择 list template。

### 4.7 Markdown 与模板安全

- 重复标题 id 去重。
- home feature card 从 `onclick="window.location='...'"` 改为普通链接。

## 5. 验证结果

| 命令 / 检查 | 结果 |
| --- | --- |
| description 缺口扫描 | 通过：`missing 0` |
| `cargo fmt --all` | 通过 |
| `cargo test --workspace --all-features` | 通过：CLI 2、core 61、shared 2、doctest 2 passed / 2 ignored |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 通过 |
| `cargo run -p novel-cli -- build` | 通过 |
| `cargo run -p novel-cli -- check` | 通过 |
| `cargo package -p novel-shared --allow-dirty` | 通过 |
| `cargo package -p novel-core --allow-dirty` | 未通过：crates.io 上没有 `novel-shared 0.1.0` |
| `cargo package -p novel-cli --allow-dirty` | 未通过：crates.io 上没有 `novel-core 0.1.0` |
| `dist/` 内部控制文件扫描 | 通过：未发现 `data/`、`_collection.toml`、`_*.toml/json/yaml/yml` |

说明：`novel-core`/`novel-cli` 的 package 失败原因已经从“manifest 缺 metadata / path dependency 缺 version”变为“依赖 crate 尚未发布到 crates.io”。这属于发布顺序问题，不再是同一个 manifest 缺陷，但仍会阻止 `cargo install novel-cli` 对真实用户成立。

## 6. 当前仍存在的缺陷与 gaps

1. 发布链路仍需决策：若坚持 crates.io 安装，需要先发布 `novel-shared`，再发布 `novel-core`，最后发布 `novel-cli`；若主推 GitHub release binary，需要同步修改 README/deploy/getting-started。
2. i18n + general SSG 尚未完整：locale 内 posts collection、taxonomy、archive、series 应生成 locale-scoped 页面，还是全站混合，需要明确产品模型。
3. base URL 缺少端到端 fixture：本次修了主线 helper 和模板，但仍需要测试 `base="/repo/"` 下 HTML、search、llms、sitemap、feed、PWA、i18n redirect 全部正确。
4. Tera/Handlebars 仍不是完整替代引擎：list/terms 与 helper parity 没有补齐。
5. dev watcher 仍未覆盖 templates、custom CSS、Sass inputs/load paths、theme pack、数据文件。
6. 日期字段仍是字符串语义，缺少 `YYYY-MM-DD` / RFC3339 解析与错误提示。
7. dead link 检查仍未覆盖静态资产存在性，build/check 的 fail 语义仍需统一。
8. CI 未加入本次本地验证矩阵。

## 7. 建议下一步

### 发布前必须

1. 明确 release 策略，并让 `cargo install novel-cli` 或替代安装路径与文档一致。
2. 增加发布顺序脚本或 release checklist：`novel-shared` -> `novel-core` -> `novel-cli`。
3. 增加 `fixtures/base-path` 和 `fixtures/i18n-collections`，把 base/i18n/general SSG 加入 CI。
4. 决定 Tera/Handlebars 是完整支持还是 experimental，并更新 docs/代码。

### Beta 稳定

1. 修复 dev watcher 覆盖面。
2. 日期字段类型化并在 frontmatter 解析阶段校验。
3. 增加 HTML link crawl 和静态资产存在性检查。
4. 在安全文档中明确 Markdown/config/template 是 trusted author input。

### CI 建议命令

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p novel-cli -- build
cargo run -p novel-cli -- check
cargo package -p novel-shared --allow-dirty
```

`cargo package -p novel-core` 和 `cargo package -p novel-cli` 应在对应依赖发布到 crates.io 后加入硬门禁，或使用专门的 publish workflow 分阶段执行。

## 8. 最终结论

本次已修复多个会直接影响真实使用的缺陷：文档质量门禁、构建失败策略、内部文件泄露、已暴露配置未接线、重复 heading id、默认模板中的 base URL 和 inline JS 风险。项目现在更适合作为公开 beta 继续推进。

但它还不应宣称 crates.io 安装链路已经可用。公开发布前，必须完成发布顺序和文档策略闭环，并补上 base/i18n/general SSG 的端到端回归。
