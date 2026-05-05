# Page Feedback

Novel includes a static page feedback widget. It works without a backend by storing the user's response in `localStorage`, and it can optionally link to an external issue tracker or form.

## Enable Feedback

```toml title="novel.toml"
[feedback]
enabled = true
question = "Was this page helpful?"
positive_text = "Yes"
negative_text = "No"
thanks_text = "Thanks for the feedback."
```

When enabled, doc pages show the feedback widget below the previous/next links.

## External Links

Use links when you want feedback to open a form, issue template, or email flow:

```toml
[feedback]
enabled = true
positive_link = "https://github.com/user/repo/discussions"
negative_link = "https://github.com/user/repo/issues/new"
```

Novel still records that the user clicked a feedback button locally, so the widget is not shown again on that page in the same browser.
