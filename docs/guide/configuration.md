# Configuration

Sapid is configured via a `sapid.toml` file in your project root. All fields are optional and have sensible defaults.

## Basic Options

```toml title="sapid.toml"
# Site title displayed in the navbar and page titles
title = "My Docs"

# Site description used in meta tags
description = "Documentation for my project"

# Documentation root directory (relative to project root)
root = "docs"

# Output directory for the built site
out_dir = "dist"

# Base URL path — set this when deploying to a subpath
# e.g. "/docs/" for https://example.com/docs/
base = "/"

# Default language (used in <html lang="...">)
lang = "en"

# Full site URL — enables sitemap.xml and feed.xml generation
site_url = "https://example.com"

# Path to logo image (displayed in the navbar)
logo = "/logo.svg"

# Path to favicon
icon = "/favicon.ico"

# Remove .html extensions from URLs
clean_urls = false
```

## Markdown Options

```toml title="sapid.toml"
[markdown]
# Show line numbers on all code blocks by default
show_line_numbers = false

# Wrap long code lines by default
default_wrap_code = false

# Check for dead internal links during build
check_dead_links = false
```

## Theme Options

```toml title="sapid.toml"
[theme]
# Enable the dark/light mode toggle
dark_mode = true

# Footer text (HTML is supported)
footer = "Built with Sapid | MIT License"

# Show git last-updated timestamps on pages
last_updated = true

# Custom text for the last-updated label
last_updated_text = "Last updated"

# "Edit this page" link pattern
# The page's relative file path is appended to this URL
edit_link = "https://github.com/user/repo/edit/main/docs/"

# Custom text for the edit link
edit_link_text = "Edit this page"

# Source code repository link (shows GitHub icon in navbar)
source_link = "https://github.com/user/repo"
```

### Navigation

By default, navigation is auto-generated from top-level directories in your docs folder. To customise:

```toml title="sapid.toml"
[[theme.nav]]
text = "Guide"
link = "/guide/"

[[theme.nav]]
text = "API"
link = "/api/"

[[theme.nav]]
text = "Blog"
link = "https://blog.example.com"
```

### Sidebar

By default, sidebar is auto-generated from the directory structure and `_meta.json` files. To customise:

```toml title="sapid.toml"
# Sidebar for pages under /guide/
[[theme.sidebar."/guide/"]]
type = "link"
text = "Getting Started"
link = "/guide/getting-started"

[[theme.sidebar."/guide/"]]
type = "link"
text = "Configuration"
link = "/guide/configuration"
```

### Social Links

```toml title="sapid.toml"
[[theme.social_links]]
icon = "GitHub"
link = "https://github.com/user/repo"

[[theme.social_links]]
icon = "Twitter"
link = "https://twitter.com/user"
```

### Banner

Display an announcement banner at the top of every page:

```toml title="sapid.toml"
[theme.banner]
text = "Sapid v1.0 is released!"
link = "/guide/getting-started"
dismissible = true
```

## Custom Templates

Sapid checks a `templates/` folder in your project root before falling back to the built-in embedded templates. Override only the files you need; any missing template still uses the default version.

```text
templates/
  base.html
  doc.html
  home.html
  404.html
```

## Full Example

```toml title="sapid.toml"
title = "My Project"
description = "Documentation for My Project"
root = "docs"
out_dir = "dist"
base = "/"
lang = "en"
site_url = "https://my-project.dev"
logo = "/logo.svg"
icon = "/favicon.ico"

[markdown]
show_line_numbers = false
check_dead_links = true

[theme]
dark_mode = true
footer = "Copyright 2025 My Project"
last_updated = true
edit_link = "https://github.com/user/my-project/edit/main/docs/"
source_link = "https://github.com/user/my-project"

[[theme.nav]]
text = "Guide"
link = "/guide/"

[[theme.nav]]
text = "API"
link = "/api/"

[theme.banner]
text = "v1.0 is out!"
link = "/guide/getting-started"
dismissible = true
```
