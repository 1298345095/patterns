# 引言

## 参与编写

如果你有兴趣为本书做贡献，请阅读
[贡献指南](https://github.com/rust-unofficial/patterns/blob/master/CONTRIBUTING.md)。

## 动态

- **2025-12-14**：新增模式：
  [用自定义 trait 简化复杂的类型约束](./patterns/structural/trait-for-bounds.md)。
- **2024-03-17**：现在可以下载
  [本书的 PDF 版本](https://rust-unofficial.github.io/patterns/rust-design-patterns.pdf)。

## 设计模式

在软件开发中，我们常会遇到一些相似的问题，即使它们出现的环境各不相同。
要解决手头的具体问题，实现细节固然很重要；但我们也可以抽离这些特殊之处，
找出普遍适用的共同做法。

设计模式汇集了工程实践中针对反复出现的问题而形成的、经过验证且可以复用的解决方案。
它们使软件更易于模块化、维护和扩展。此外，设计模式还为开发者提供了一套共同语言，
因此，在团队协作解决问题时，它们也是促进有效沟通的有力工具。

请记住：每种模式都有自己的取舍。比起只关注如何实现一种模式，
更重要的是弄清楚为什么要选择它。[^1]

## Rust 中的设计模式

Rust 不是面向对象语言。函数式编程元素、强大的类型系统、借用检查器等诸多特性
结合在一起，使 Rust 独具特色。因此，Rust 中的设计模式与传统面向对象语言中的
设计模式有所不同。这正是我们决定编写本书的原因。希望你读得愉快！
本书分为三个主要章节：

> **译注（原文疑点）**：“Rust 不是面向对象语言”是原文的概括，取决于如何定义
> “面向对象”。Rust 支持封装，也能通过泛型和 trait 对象实现不同形式的多态，
> 但不提供传统的结构体实现继承。阅读这些模式时，更有用的问题是：它需要哪些
> 语言机制，Rust 用什么机制表达同样的约束？参见
> [《Rust 程序设计语言》对面向对象特征的讨论](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html)。

- [惯用法](./idioms/index.md)：编写代码时应遵循的指导原则，也是社区约定俗成的规范。
  只有在理由充分时，才应打破这些惯例。
- [设计模式](./patterns/index.md)：解决编程中常见问题的方法。
- [反模式](./anti_patterns/index.md)：同样是解决编程中常见问题的方法。
  不过，设计模式能够带来益处，而反模式却会造成更多问题。

> **译注（原文疑点）**：这里的“三个主要章节”沿用原文。
> 当前版本还包含“函数式编程”和“补充资源”等内容，请以[目录](./SUMMARY.md)为准。
> 本译本的阅读约定与术语见[译者说明](./translation-notes.md)。
> 对照当前 stable 的技术更新见[2026 年 Rust 兼容性审校](./stable-audit-1.99.md)。

[^1]: <https://www.infoq.com/podcasts/software-architecture-hard-parts/>
    （[存档](https://web.archive.org/web/20240124025806/https://www.infoq.com/podcasts/software-architecture-hard-parts/)）
