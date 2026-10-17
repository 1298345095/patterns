# 贡献指南

## 简介

本书汇集了 Rust 编程技巧、设计模式与反模式、惯用法，以及其他相关讲解。
这些内容来自协作实践中逐渐积累的共同知识——其中有些原本只是隐含的经验——
以及参与者的亲身经历。

这里介绍的模式**不是规则**，而是帮助你编写符合 Rust 惯用法的代码的指导原则。
我们把这些 Rust 模式收集到书中，是为了让读者理解不同惯用法之间的取舍，
并在自己的代码中恰当地使用它们。

如果你愿意参与这项工作，可以通过以下方式加入。

## 讨论区

如果你对某些内容有疑问或想法，希望获得社区其他成员的反馈， 但又觉得不适合提交
issue，可以在我们的
[讨论区](https://github.com/rust-unofficial/patterns/discussions)发起讨论。

## 撰写新文章

开始撰写新文章前，请先查看以下资源，确认是否已有相关讨论，
或者已经有人在处理这个主题：

- [主题汇总 issue](https://github.com/rust-unofficial/patterns/issues/116)
- [所有 issue](https://github.com/rust-unofficial/patterns/issues)
- [Pull Request 列表](https://github.com/rust-unofficial/patterns/pulls)

如果没有找到相关 issue，而且你确信这件事不更适合放在
[讨论区](https://github.com/rust-unofficial/patterns/discussions)， 请新建一个
issue。这样，我们就能一起讨论文章的想法与后续内容， 也可能提供一些反馈或建议。

撰写新文章时，建议把
[模式模板](https://github.com/rust-unofficial/patterns/blob/master/template.md)
复制到合适的目录，再开始编辑。你不必填写每个小节，可以删掉不需要的小节，
也可以添加新的小节。

请尽量降低文章的理解门槛，让正在学习
[Rustlings](https://github.com/rust-lang/rustlings)
的读者也能跟上并理解其中的思路。 这样，我们就能鼓励人们尽早使用这些模式。

我们鼓励你编写符合 Rust 惯用法、且能在 [Playground](https://play.rust-lang.org/)
中构建的代码。

如果引用博客文章，或者引用其他无法确定几年后是否仍然存在的内容，例如 PDF， 请用
[Wayback Machine](https://web.archive.org/) 保存快照， 并在文章中使用快照链接。

别忘了把新文章加入 `SUMMARY.md`，这样它才会被渲染到书中。

请尽早创建草稿 Pull Request，方便我们跟踪进展并及时反馈，详见后文。

## 风格指南

为了让全书风格保持一致，我们建议：

- 遵循 Rust 官方书籍的
  [风格指南](https://github.com/rust-lang/book/blob/master/style-guide.md)。
- 遵循
  [RFC 1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text)。
  主要包括：
  - 优先写出完整的类型名称，例如使用 `Option<T>`，而不是 `Option`。
  - 在适用的情况下，优先使用行注释 `//`，而不是块注释 `/* */`。

## 在本地检查文章

提交 Pull Request 前，请运行 `mdbook build`，确认本书能正常构建； 再运行
`mdbook test`，确认代码示例正确。

### Markdown 检查

我们使用 [markdownlint-cli](https://github.com/igorshubovych/markdownlint-cli)
检查 Markdown 文件是否符合格式要求。 为了减少手工修正格式、通过 CI
检查的工作量， 可以使用以下命令自动修复编写 Markdown 时出现的大多数格式问题。

- 安装：

  ```sh
  npm install -g markdownlint-cli
  ```

- 检查所有 Markdown 文件：
  - Unix：`markdownlint '**/*.md'`
  - Windows：`markdownlint **/*.md`

- 自动修复基本格式错误：
  - Unix：`markdownlint -f '**/*.md'`
  - Windows：`markdownlint -f **/*.md`

## 创建 Pull Request

“尽早发布，经常发布！”也同样适用于 Pull Request。

当文章已经有一些可以展示的内容时，就创建一个带 `[WIP]` 标记的草稿 Pull Request，
并说明已经完成以及打算完成的工作。社区的早期评审不是针对个人的批评，
而是为了提供反馈。

一个值得遵循的原则是：“一起工作，分享想法，帮助他人学习。”

### 重要说明

请**不要强制推送**分支中的提交，以便保留提交历史，
也方便我们查看不同轮次评审之间的变化。

请勾选文本框下方的 `Allow edits of maintainers`，允许维护者编辑，
这样大家才能真正协作，或者直接修正一些小问题。

## 维护者

本仓库由以下人员维护：

- [simonsan](https://github.com/simonsan)。
- [marcoieni](https://github.com/marcoieni)。Marco 只负责本书的 CI，
  不评审正文内容，也不会收到新 issue 或 Pull Request 的通知。 如果希望他查看 CI
  问题，请直接提及他。

> **译注（版本说明）**：本页是原仓库贡献指南的译文；其中的协作流程、维护者介绍
> 和上游链接均保留原意。本中文学习分支没有向上述人员发送通知或提交 Pull
> Request。
