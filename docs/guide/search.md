# Search

Sapid includes built-in client-side search with no external dependencies or services required.

## How It Works

During `sapid build`, a search index (`search-index.json`) is generated from all pages. The search is fully client-side — no server or third-party service needed.

The search index includes:
- Page titles
- Section headings
- Page content (with HTML stripped)
- Page descriptions

## Using Search

The search box is displayed in the sidebar on every documentation page. You can also press `/` on your keyboard to focus the search input from anywhere on the page.

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` | Focus the search input |
| `Escape` | Close search results |
| `Arrow Up/Down` | Navigate between results |
| `Enter` | Go to the selected result |

## Search Ranking

Results are ranked by relevance:

1. **Title matches** score highest (10 points)
2. **Heading matches** score medium (5 points)
3. **Content matches** score lowest (1 point)

Results are sorted by score and limited to 8 results.

## Search Preview

Each search result shows:
- The page title (with the matched term highlighted)
- A content preview showing the surrounding context of the match

## Performance

The search index is loaded lazily — it's only fetched when the search input is first focused. This means the search index does not impact initial page load performance.
