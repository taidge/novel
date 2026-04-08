# 部署

运行 `novel build` 之后,`dist/` 目录包含一个完全静态的站点,可以部署到任何地方。

## Netlify

### 通过 Git 集成

1. 将你的项目推送到一个 Git 仓库(GitHub、GitLab 或 Bitbucket)。
2. 登录 [Netlify](https://app.netlify.com) 并点击 **Add new site** > **Import an existing project**。
3. 选择你的仓库并配置:
   - **Build command**: `novel build`
   - **Publish directory**: `dist`
4. 点击 **Deploy site**。

Netlify 会在每次推送时自动重新构建。

### 通过 CLI

安装 Netlify CLI 并手动部署:

```bash
# 安装 Netlify CLI
npm install -g netlify-cli

# 构建站点
novel build

# 部署到 Netlify
netlify deploy --prod --dir dist
```

### 配置文件

在项目根目录创建 `netlify.toml` 以持久化配置:

```toml title="netlify.toml"
[build]
  command = "novel build"
  publish = "dist"

# 为干净 URL 提供 SPA 风格的回退
[[redirects]]
  from = "/*"
  to = "/404.html"
  status = 404
```

::: tip
如果 Netlify 的构建环境上没有安装 Novel,请在构建脚本中先安装它:

```toml title="netlify.toml"
[build]
  command = "cargo install novel-cli && novel build"
  publish = "dist"
```
:::

## GitHub Pages

### 使用 GitHub Actions

创建 `.github/workflows/deploy.yml`:

```yaml title=".github/workflows/deploy.yml"
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Novel
        run: cargo install novel-cli

      - name: Build
        run: novel build

      - name: Setup Pages
        uses: actions/configure-pages@v4

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: dist

      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

::: warning
如果你的站点部署在子路径下(例如 `https://user.github.io/repo/`),请在 `novel.toml` 中设置 `base` 选项:

```toml
base = "/repo/"
```
:::

## Vercel

1. 将你的项目推送到一个 Git 仓库。
2. 在 [Vercel](https://vercel.com) 上导入项目。
3. 配置构建设置:
   - **Build command**: `cargo install novel-cli && novel build`
   - **Output directory**: `dist`
4. 部署。

或者使用 Vercel CLI:

```bash
npm install -g vercel
novel build
cd dist && vercel --prod
```

## Cloudflare Pages

1. 登录 [Cloudflare 控制台](https://dash.cloudflare.com),进入 **Workers & Pages**。
2. 点击 **Create application** > **Pages** > **Connect to Git**。
3. 选择你的仓库并配置:
   - **Build command**: `cargo install novel-cli && novel build`
   - **Build output directory**: `dist`
4. 部署。

## 任意静态主机

由于 `novel build` 输出的是普通的 HTML、CSS 和 JS 文件,你可以部署到任何静态文件主机:

```bash
# 构建站点
novel build

# 将 dist/ 目录上传到你的服务器
rsync -avz dist/ user@server:/var/www/docs/

# 或在本地使用任意静态文件服务器
npx serve dist
python -m http.server -d dist 8080
```

## Docker

使用轻量级 Nginx 容器提供构建好的站点:

```dockerfile title="Dockerfile"
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo install novel-cli && novel build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
```

构建并运行:

```bash
docker build -t my-docs .
docker run -p 8080:80 my-docs
```

## Base 路径

当部署到子路径(例如 `https://example.com/docs/`)时,请配置 `base` 选项:

```toml title="novel.toml"
base = "/docs/"
```

这能确保所有资源链接和导航都使用正确的前缀。
