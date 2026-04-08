# Home Page

The home page is configured through frontmatter in your root `docs/index.md` file.

## Setup

Set `page_type: home` in the frontmatter to activate the home page layout:

```yaml title="docs/index.md"
---
page_type: home
hero:
  name: My Project
  text: Build Amazing Things
  tagline: A fast and modern tool for developers
  actions:
    - text: Get Started
      link: /guide/getting-started
      theme: brand
    - text: GitHub
      link: https://github.com/user/repo
      theme: alt
features:
  - title: Fast
    icon: "\u26A1"
    details: Built for speed with Rust at its core.
  - title: Simple
    icon: "\U0001F4DD"
    details: Write Markdown, get a beautiful site.
  - title: Flexible
    icon: "\U0001F527"
    details: Customise everything to your needs.
---
```

## Hero Section

The hero section is the large banner at the top of the home page.

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Large heading text |
| `text` | string | Secondary heading |
| `tagline` | string | Description text below the heading |
| `actions` | array | Call-to-action buttons |
| `image` | object | Optional hero image |

### Actions

Each action is a button in the hero section:

```yaml
actions:
  - text: Get Started       # Button label
    link: /guide/intro       # URL to navigate to
    theme: brand             # "brand" (primary) or "alt" (secondary)
```

### Hero Image

```yaml
hero:
  image:
    src: /hero-image.png
    alt: My Project Logo
```

## Features Grid

The features section displays a grid of feature cards below the hero:

```yaml
features:
  - title: Feature Name
    icon: "\U0001F680"        # Emoji or text icon
    details: Description of this feature.
    link: /guide/feature       # Optional: makes the card clickable
```

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Feature card heading |
| `icon` | string | Emoji or text displayed above the title |
| `details` | string | Feature description |
| `link` | string | Optional URL — makes the card clickable |

## Body Content

Any Markdown content below the frontmatter `---` is rendered below the features grid. This is useful for adding additional sections to your home page.
