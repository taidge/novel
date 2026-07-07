# Data Structure Audit Todos

## Audit Summary

The current public data model mixes several competing field vocabularies:

- File-system paths use both `root` and `out_dir`, while internal code calls the same concepts docs root and output directory.
- Href-like destinations use both `link` and `url` (`edit_url`, `markdown_url`, `prev_url`, `next_url` already use URL terminology).
- Page layout is split between legacy `page_type` and `layout`.
- Dates are split across `date`, `updated`, `expiry_date`, and generated `last_updated`.
- Taxonomies are configured generically, but frontmatter still has hard-coded `tags` and `categories`.
- KDL config and `.well-known/llms*.txt` outputs are compatibility surfaces not used by the documented TOML-first model.

## Tasks

- [x] T1. Rename site path fields: `root` -> `docs_dir`, `out_dir` -> `output_dir`.
- [x] T2. Normalize href fields from `link` to `url` across shared types, config, templates, plugins, docs, examples, and generated scaffold.
- [x] T3. Remove `page_type` and `PageType`; use `layout` as the single page layout selector.
- [x] T4. Rename date fields: `date` -> `published_at`, `updated` -> `updated_at`, `expiry_date` -> `expires_at`, generated `last_updated` -> `git_updated_at`.
- [x] T5. Replace fixed `tags` / `categories` frontmatter fields with generic `taxonomies`.
- [x] T6. Tighten collection and taxonomy pagination/sorting names: `paginate_by` -> `per_page`, typed `sort_by` / `order`, no sentinel `0` compatibility.
- [x] T7. Delete historical compatibility surfaces: KDL config loading and `.well-known/llms*.txt` duplicate outputs.
- [x] T8. Update documentation, scaffold templates, and tests to the new field names.
- [x] T9. Run formatting and test/build verification; fix any failures.
