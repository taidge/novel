# 页面反馈

Novel 内置一个静态页面反馈组件。它不需要后端,会把用户的选择存到 `localStorage`,也可以跳转到外部 issue tracker 或表单。

## 启用反馈

```toml title="novel.toml"
[feedback]
enabled = true
question = "这个页面有帮助吗?"
positive_text = "有"
negative_text = "没有"
thanks_text = "感谢反馈。"
```

启用后,文档页会在上一页/下一页链接下方显示反馈组件。

## 外部链接

如果希望反馈打开表单、issue 模板或邮件流程,可以配置链接:

```toml
[feedback]
enabled = true
positive_link = "https://github.com/user/repo/discussions"
negative_link = "https://github.com/user/repo/issues/new"
```

Novel 仍会在本地记录用户已经点击过反馈按钮,因此同一浏览器中该页面不会重复显示反馈按钮。
