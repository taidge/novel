# 离线 / PWA

Novel 可以生成一个轻量的 Progressive Web App 外壳,用于离线阅读。

## 启用 PWA 输出

```toml title="novel.toml"
[pwa]
enabled = true
name = "My Docs"
short_name = "Docs"
theme_color = "#3b82f6"
background_color = "#ffffff"
display = "standalone"
cache_search_index = true
```

启用后,Novel 会写出:

- `manifest.webmanifest`
- `service-worker.js`
- `offline.html`

默认模板会自动注册 service worker,并在 `<head>` 中链接 manifest。

## 缓存策略

Service worker 会预缓存站点根路径、公共页面路由、构建后的 CSS/JS、`offline.html`,以及在 `cache_search_index = true` 时缓存搜索索引。HTML 导航使用 network-first 策略,失败时回退到缓存页面或 `offline.html`。
