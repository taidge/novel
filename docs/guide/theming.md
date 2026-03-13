# Theming

Sapid comes with a clean, responsive theme that supports dark mode out of the box.

## Dark Mode

Dark mode is enabled by default. Users can toggle between light and dark mode using the button in the navbar.

The preference is saved to `localStorage` and persisted across page loads. If no preference is saved, Sapid follows the system preference (`prefers-color-scheme`).

To disable the dark mode toggle:

```toml title="sapid.toml"
[theme]
dark_mode = false
```

## Footer

Add a footer to every page:

```toml title="sapid.toml"
[theme]
footer = "Copyright 2025 My Project | Built with Sapid"
```

HTML is supported in the footer text.

## Edit Link

Show an "Edit this page" link on every doc page that links to your repository:

```toml title="sapid.toml"
[theme]
edit_link = "https://github.com/user/repo/edit/main/docs/"
edit_link_text = "Edit this page on GitHub"
```

The page's relative file path is appended automatically.

## Last Updated

Show the last git commit date for each page:

```toml title="sapid.toml"
[theme]
last_updated = true
last_updated_text = "Last updated"
```

This uses `git log` to determine the date, so git must be available in the build environment.

## Source Link

Display a GitHub (or other) repository link in the navbar:

```toml title="sapid.toml"
[theme]
source_link = "https://github.com/user/repo"
```

## Banner

Show an announcement banner at the top of every page:

```toml title="sapid.toml"
[theme.banner]
text = "We just released v2.0!"
link = "/guide/changelog"
dismissible = true
```

When `dismissible` is `true`, the user can close the banner. The dismissed state is stored in `sessionStorage`.

## Responsive Layout

The theme is fully responsive:

- **Desktop**: Three-column layout (sidebar, content, table of contents)
- **Tablet**: Two-column layout (content, table of contents)
- **Mobile**: Single-column layout with a hamburger menu for the sidebar

## Built-in Components

The theme includes several visual components:

| Component | Description |
|-----------|-------------|
| **Container directives** | Tip, warning, danger, info, note, details |
| **Tabs** | Tabbed content blocks |
| **Steps** | Numbered step-by-step guides |
| **Badges** | Inline status badges |
| **Code blocks** | Syntax highlighting, line numbers, titles, copy button |
| **Image zoom** | Click to zoom on images |
| **Back to top** | Floating button when scrolled down |
| **Table of contents** | Auto-generated from page headings |
