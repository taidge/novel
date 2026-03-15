# Deployment

After running `novel build`, the `dist/` directory contains a fully static site that can be deployed anywhere.

## Netlify

### Via Git Integration

1. Push your project to a Git repository (GitHub, GitLab, or Bitbucket).
2. Log in to [Netlify](https://app.netlify.com) and click **Add new site** > **Import an existing project**.
3. Select your repository and configure:
   - **Build command**: `novel build`
   - **Publish directory**: `dist`
4. Click **Deploy site**.

Netlify will automatically rebuild on every push.

### Via CLI

Install the Netlify CLI and deploy manually:

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Build the site
novel build

# Deploy to Netlify
netlify deploy --prod --dir dist
```

### Configuration File

Create a `netlify.toml` in your project root for persistent config:

```toml title="netlify.toml"
[build]
  command = "novel build"
  publish = "dist"

# SPA-style fallback for clean URLs
[[redirects]]
  from = "/*"
  to = "/404.html"
  status = 404
```

::: tip
If Novel is not installed on the Netlify build environment, add a build script that installs it first:

```toml title="netlify.toml"
[build]
  command = "cargo install novel-cli && novel build"
  publish = "dist"
```
:::

## GitHub Pages

### Using GitHub Actions

Create `.github/workflows/deploy.yml`:

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
If your site is deployed to a subpath (e.g. `https://user.github.io/repo/`), set the `base` option in `novel.toml`:

```toml
base = "/repo/"
```
:::

## Vercel

1. Push your project to a Git repository.
2. Import the project on [Vercel](https://vercel.com).
3. Configure the build settings:
   - **Build command**: `cargo install novel-cli && novel build`
   - **Output directory**: `dist`
4. Deploy.

Or use the Vercel CLI:

```bash
npm install -g vercel
novel build
cd dist && vercel --prod
```

## Cloudflare Pages

1. Log in to the [Cloudflare dashboard](https://dash.cloudflare.com) and go to **Workers & Pages**.
2. Click **Create application** > **Pages** > **Connect to Git**.
3. Select your repository and configure:
   - **Build command**: `cargo install novel-cli && novel build`
   - **Build output directory**: `dist`
4. Deploy.

## Any Static Host

Since `novel build` outputs plain HTML, CSS, and JS files, you can deploy to any static file host:

```bash
# Build the site
novel build

# Upload the dist/ directory to your server
rsync -avz dist/ user@server:/var/www/docs/

# Or use any static file server locally
npx serve dist
python -m http.server -d dist 8080
```

## Docker

Serve the built site with a lightweight Nginx container:

```dockerfile title="Dockerfile"
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo install novel-cli && novel build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
```

Build and run:

```bash
docker build -t my-docs .
docker run -p 8080:80 my-docs
```

## Base Path

When deploying to a subpath (e.g. `https://example.com/docs/`), configure the `base` option:

```toml title="novel.toml"
base = "/docs/"
```

This ensures all asset links and navigation use the correct prefix.
