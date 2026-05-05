# Novel — 待办与本轮改进

> 调研对象: VitePress、Docusaurus、Material for MkDocs、Mintlify、mdBook 等文档站/静态站工具。只列当前项目尚未完整覆盖、且对 Novel 有实际价值的功能。

## 本轮完成

- [x] **F12 — AI 友好的 `llms.txt` / `llms-full.txt` 输出**
  - 参考: Mintlify 自动生成 `/llms.txt` 与 `/llms-full.txt`, 用 Markdown 结构列出文档页并提供完整上下文; llms.txt reference site 也建议在站点根路径提供一份机器可读的文档地图。
  - 选择理由: Novel 已经有 sitemap、feed、search index、robots 等构建期文件生成插件, 新增 LLM 文档索引可以复用同一插件机制, 低风险且符合 2026 年文档站的实际使用方式。
  - 完成范围: 默认 CLI 构建生成 `llms.txt`、`llms-full.txt`、`.well-known/llms.txt`、`.well-known/llms-full.txt`; 跳过 `noindex`、404 和 redirect 页面; `llms-full.txt` 优先使用源 Markdown 并去掉 frontmatter。

## 后续候选

- [x] **F13 — 文档版本化**
  - 参考: Docusaurus 提供 docs versioning, 让发布版本对应冻结的文档树。
  - 完成范围: 新增 `[versions]` 配置,支持从 `docs/<dir>/` 构建多个版本;当前版本保持非前缀路由,归档版本默认输出到 `/<code>/...`;同路径页面自动获得导航栏版本选择器;归档版本内的绝对内部链接自动加版本前缀。

- [x] **F14 — 每页 Markdown 镜像输出**
  - 参考: Mintlify 的 `llms.txt` 页面链接到可供 AI 工具直接读取的 `.md` 版本。
  - 完成范围: 新增 `MarkdownMirrorPlugin`,CLI 默认写出每个公共页面的 `.md` 镜像;默认移除 frontmatter;文档页脚显示 `Markdown` 链接;支持 `[markdown_mirror] enabled/strip_frontmatter` 配置。

- [x] **F15 — 离线/PWA 包**
  - 参考: Material for MkDocs 有 offline 相关能力。
  - 完成范围: 新增 `[pwa]` 配置和 `PwaPlugin`;启用后输出 `manifest.webmanifest`、`service-worker.js`、`offline.html`;默认模板自动注册 service worker 并链接 manifest;支持缓存搜索索引。

- [x] **F16 — 文档评论/反馈组件**
  - 参考: Material for MkDocs、Mintlify 类产品常见页面反馈入口。
  - 完成范围: 新增 `[feedback]` 配置;文档页可显示静态反馈组件;无后端时用 `localStorage` 记录选择,也可配置正/负反馈跳转链接到表单、issue 或讨论区。
