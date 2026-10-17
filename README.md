# Rust 设计模式

> **译者说明：中文阅读入口**
>
> 这是 `1298345095/patterns` 的 `zh-cn` 中文学习分支，依据英文原书独立翻译。
> 从[全书目录](./src/SUMMARY.md)或[引言](./src/intro.md)开始阅读即可。
> [译者说明与术语表](./src/translation-notes.md)介绍翻译约定、原文对照方式和阅读建议；
> [翻译核验记录](./src/translation-review.md)列出覆盖范围与校验结果。
> 正文中的“译注”用于区分原书内容、学习补充和原文疑点。 以下保留原 README
> 的项目介绍与构建说明；其中在线站点和 PDF 链接指向上游内容。

这是一本介绍 Rust 编程语言中设计模式与惯用法的开源书，
你可以在[这里](https://rust-unofficial.github.io/patterns/)阅读。

也可以通过
[这个链接](https://rust-unofficial.github.io/patterns/rust-design-patterns.pdf)
下载本书的 PDF 版本。

## 参与贡献

如果你发现本仓库还缺少某些有助于他人的内容，并且愿意动手讲解，欢迎参与！
我们一直欢迎为本项目贡献新内容，例如对某些主题展开说明，或纠正其中的问题。

可以查看
[主题汇总 issue](https://github.com/rust-unofficial/patterns/issues/116)，
了解哪些模式、反模式和惯用法还可以补充到书中。

建议先阅读[贡献指南](./CONTRIBUTING.md)，了解如何参与本仓库的工作。

## 使用 mdBook 构建

本书使用 [mdBook](https://rust-lang.github.io/mdBook/) 构建。 可以运行
`cargo install mdbook` 安装它。

### 其他依赖

- `cargo install mdbook-last-changed`：在页脚显示修改日期。
- `cargo install mdbook-pandoc`：将本书渲染为 PDF。
- `cargo install mdbook-i18n-helpers`：提供翻译与国际化支持。

#### Texlive

```sh
# 加载 .env 文件，取得 PANDOC_VERSION
. ./.env

sudo apt-get update

sudo apt-get install -y texlive texlive-latex-extra texlive-luatex texlive-lang-cjk librsvg2-bin fonts-noto

curl -LsSf https://github.com/jgm/pandoc/releases/download/$PANDOC_VERSION/pandoc-$PANDOC_VERSION-linux-amd64.tar.gz | tar zxf -
```

### 构建本书

如果想在本地构建，可以在仓库根目录运行以下两条命令之一：

- `mdbook build`

  构建静态 HTML 页面，默认将输出放在 `/book` 目录中。

- `mdbook serve`

  在 `http://localhost:3000`
  提供本书的本地浏览服务；内容变化时会重新加载浏览器。
  端口可以改变，请以终端输出为准。

> **译注（阅读说明）**：本中文分支的日常阅读只需要 Markdown 阅读器或 GitHub。
> 上述构建步骤是原项目的说明，包含 PDF 和国际化相关依赖，并非阅读译文的前提。

## 许可证

本仓库内容采用 **MPL-2.0** 许可证，详见 [LICENSE](./LICENSE)。
