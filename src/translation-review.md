# 翻译核验记录

本页为译者新增内容，用于说明翻译范围、核验方法与已知的原文问题。

## 版本与交付范围

- 仓库：`1298345095/patterns`，中文分支：`zh-cn`。
- 原文基线：[36fdd17cdebfffd7af94479a4caa9375e40bde80](https://github.com/1298345095/patterns/tree/36fdd17cdebfffd7af94479a4caa9375e40bde80)。
- 核验日期：2026-10-17（UTC）。
- 原书 `src/` 中全部 **51 个内容页**及目录已翻译，包含原目录未收录的重构页。
- 配套的 README、贡献指南和文章模板已翻译。
- 新增[译者说明与术语表](./translation-notes.md)以及本核验记录。
- 初译提交 `ce489e4` 的原有内容页共有 **127 条就地译注**；后续针对 Rust 1.99.1 的增补另计，详见[stable 兼容性审校](./stable-audit-1.99.md)。

原文与新增内容的区分方式见[译者说明](./translation-notes.md)。翻译时未搜索或参考现成中文译本，也未使用仓库内其他语言的译文；技术疑点根据原例和相关原始技术资料独立核对。

## 核验结果

**版本说明**：下表记录的是初译提交 `ce489e4` 的测试与保真结果。
2026-10-17 的 stable 审校**没有复用**这些测试结果来宣称新增代码已经通过编译。
新增的自检文件见[独立 smoke tests](../checks/rust_2024_smoke.rs)，
本次会话执行环境未安装 `rustc` / `mdbook`，所以新测试**尚未实际运行**。


| 检查           | 结果与范围                                                                          |
| -------------- | ----------------------------------------------------------------------------------- |
| 覆盖范围       | 51/51 个原有内容页完成翻译；原有标题层级完整保留                                    |
| 示例保真       | 100/100 个原有代码、命令和输出块通过比对，其中正文 96 个、配套文档 4 个             |
| 比对方法       | 保留围栏种类与标记，按词法排除可翻译的自然语言注释后，核对代码 token 与字符串字面量 |
| 原有链接       | 所有原有链接目标均保留；本地文件链接及引用的标题锚点检查通过                        |
| 语言检查       | 逐段校阅，并检查代码块外的残留英文段落；类型、API、专名与必要原始输出按约定保留     |
| 交叉校阅       | 惯用法、模式、FFI、函数式编程及引言和反模式均完成作者以外的交叉校阅，并落实修订     |
| Markdown 格式  | 使用原仓库指定的 Markdown 插件 `markdown-0.20.0`，按原有 dprint 选项检查通过        |
| Markdown 结构  | markdownlint 结构检查通过；长命令、表格和不可拆分文本的 80 列限制单独排除           |
| 空白与补丁     | `git diff --check` 通过                                                             |
| 英文示例基线   | `mdbook test` 通过                                                                  |
| 中文示例       | 同一工具链、同一 edition 下的 `mdbook test` 通过                                    |
| 本地解析与构建 | 全书 `mdbook build` 通过                                                            |

代码比对不把注释中的讲解文字当作程序行为；被注释掉的示例代码另做内容复核。没有为使测试通过而改写原书程序。

### 示例测试的具体范围

使用 **mdBook 0.5.0**、**rustc/rustdoc
1.99.1**（`58cf8f9c3`，2026-10-13），保留原仓库的 **Rust 2024 edition** 设置。
英文和中文各自在独立的验证副本中测试；副本保留全部章节文本，采用只配置书籍源目录、语言和
Rust edition 的最小 mdBook 配置，以便检查正文而不引入 gettext、PDF 或发布流程。

测试遵循原书的代码块标记：`compile_fail` 按预期编译失败检查，`ignore`
保留为说明性示例。C、Java、shell
和原始输出块做保真比对。因此，测试通过表示本次翻译没有引入已测试示例的回归，并不把原书的说明性片段认定为可直接运行的完整程序。

## 原文疑点的阅读入口

以下是已就近标注的一部分问题，便于优先阅读；完整说明位于对应段落旁。

| 章节                                                  | 已保留原文并说明的问题                                                            |
| ----------------------------------------------------- | --------------------------------------------------------------------------------- |
| [引言](./intro.md)                                    | “Rust 不是面向对象语言”取决于定义；“三个主要章节”与当前目录范围不同               |
| [析构时收尾](./idioms/dtor-finally.md)                | 析构期间再次 panic 的行为、清理执行条件与绑定方式                                 |
| [可扩展性](./idioms/priv-extend.md)                   | `non_exhaustive` 的限制主要作用于定义它的 crate 之外，原例位于同一 crate          |
| [命令模式](./patterns/behavioural/command.md)         | 函数指针、可转换的闭包和动态分发之间的区别                                        |
| [构建器](./patterns/creational/builder.md)            | `Default` 与 `new()` 的初始值不一致；复用说明与消费 `self` 的接口之间的差别       |
| [折叠](./patterns/creational/fold.md)                 | 共享所有权不等于绝对不可变；`Iterator::fold` 的结果也可以是集合或树               |
| [小型 crate](./patterns/structural/small-crates.md)   | 有条件删除与 yank 的区别；锁文件、可复现构建和默认 LTO 的适用范围                 |
| [FFI 字符串传递](./idioms/ffi/passing-strings.md)     | 同语句调用与保存裸指针之后使用的生命周期差别，以及原例中的类型和 NUL 处理问题     |
| [基于对象的 FFI API](./patterns/ffi/export.md)        | 作者建议与 POSIX DBM 的真实所有权契约之间的差别，以及 `void *`、NULL 和长度的约束 |
| [FFI 包装器](./patterns/ffi/wrappers.md)              | 地址稳定性与别名规则不同；重建迭代状态前必须结束相关借用                          |
| [泛型与类型类](./functional/generics-type-classes.md) | 类型错误与语法错误、不同语言泛型机制的边界，以及脚注类型不一致                    |
| [Optics](./functional/optics.md)                      | Iso/Prism 的标准含义、关联类型与类型擦除、Serde 的实际驱动方向和访问层次          |
| [重构](./refactoring/index.md)                        | 原页未收录在目录中，且“小步修改”只有标题；译本保留并标明未完成状态                |

## 2026-10-17 Rust stable 兼容性复核

审校对象为 Rust 1.99.1 和 Rust 2024 Edition，
结果列于[独立审校报告](./stable-audit-1.99.md)。
本次保持全部原始代码和围栏标记不变，
增补 FFI 的 2024 语法对照、独立分配／释放示例、
lint 现代用法、Edition 2024 析构范围、trait upcasting 与 GAT 等学习笔记。
若要实测新示例，可在仓库根目录执行：

```sh
rustc --edition=2024 --test checks/rust_2024_smoke.rs -o /tmp/rust_2024_smoke
/tmp/rust_2024_smoke
```

此命令是交给读者在安装 Rust stable 的机器上运行的**待执行检查**，
不是本次会话已获得的测试结果。

## 逐页覆盖清单

下表按本译本目录的阅读顺序列出全部 51 个原有内容页。每一行均已全文翻译。
“示例块”包含代码、命令及输出，不等同于可执行测试数；短导言不强行添加译注。
本表的“译注数／示例块数”也是**初译时的快照**，不包含后续 stable 增补的译注和对照代码，
不应解释为 `zh-cn` 分支当前所有代码块的实时数量。

| 章节                                                                                    | 译注数 | 示例块数 |
| --------------------------------------------------------------------------------------- | -----: | -------: |
| [引言](./intro.md)                                                                      |      2 |        0 |
| [翻译](./translations.md)                                                               |      1 |        0 |
| [惯用法](./idioms/index.md)                                                             |      0 |        0 |
| [使用借用类型作为参数](./idioms/coercion-arguments.md)                                  |      3 |        5 |
| [使用 format! 拼接字符串](./idioms/concat-format.md)                                    |      1 |        1 |
| [构造函数](./idioms/ctor.md)                                                            |      2 |        3 |
| [Default trait](./idioms/default.md)                                                    |      2 |        1 |
| [集合也是智能指针](./idioms/deref.md)                                                   |      2 |        1 |
| [在析构函数中执行收尾工作](./idioms/dtor-finally.md)                                    |      3 |        1 |
| [使用 mem::{take(*), replace(*)} 在切换枚举变体时保留拥有的值](./idioms/mem-replace.md) |      2 |        2 |
| [在栈上进行动态分发](./idioms/on-stack-dyn-dispatch.md)                                 |      4 |        2 |
| [FFI 惯用法](./idioms/ffi/intro.md)                                                     |      0 |        0 |
| [FFI 中的错误处理](./idioms/ffi/errors.md)                                              |      3 |        3 |
| [接收字符串](./idioms/ffi/accepting-strings.md)                                         |      3 |        2 |
| [传递字符串](./idioms/ffi/passing-strings.md)                                           |      4 |        2 |
| [遍历 Option](./idioms/option-iter.md)                                                  |      2 |        2 |
| [向闭包传递变量](./idioms/pass-var-to-closure.md)                                       |      3 |        2 |
| [使用 #[non_exhaustive] 和私有字段保留扩展空间](./idioms/priv-extend.md)                |      3 |        2 |
| [简化文档示例中的初始化](./idioms/rustdoc-init.md)                                      |      3 |        2 |
| [临时可变性](./idioms/temporary-mutability.md)                                          |      2 |        2 |
| [在出错时返还已消耗的参数](./idioms/return-consumed-arg-on-error.md)                    |      2 |        1 |
| [设计模式](./patterns/index.md)                                                         |      1 |        0 |
| [行为型模式](./patterns/behavioural/intro.md)                                           |      0 |        0 |
| [命令](./patterns/behavioural/command.md)                                               |      3 |        3 |
| [解释器](./patterns/behavioural/interpreter.md)                                         |      3 |        3 |
| [新类型（newtype）](./patterns/behavioural/newtype.md)                                  |      3 |        3 |
| [借助守卫实现 RAII](./patterns/behavioural/RAII.md)                                     |      3 |        2 |
| [策略（Strategy，也称 Policy）](./patterns/behavioural/strategy.md)                     |      4 |        3 |
| [访问者](./patterns/behavioural/visitor.md)                                             |      3 |        2 |
| [创建型模式](./patterns/creational/intro.md)                                            |      0 |        0 |
| [构建器](./patterns/creational/builder.md)                                              |      3 |        2 |
| [折叠（fold）](./patterns/creational/fold.md)                                           |      4 |        1 |
| [结构型模式](./patterns/structural/intro.md)                                            |      0 |        0 |
| [拆分结构体以分别借用](./patterns/structural/compose-structs.md)                        |      2 |        3 |
| [优先使用小型 crate](./patterns/structural/small-crates.md)                             |      3 |        0 |
| [将 unsafe 代码限制在小模块中](./patterns/structural/unsafe-mods.md)                    |      3 |        0 |
| [用自定义 trait 简化复杂的类型约束](./patterns/structural/trait-for-bounds.md)          |      3 |        2 |
| [FFI 模式](./patterns/ffi/intro.md)                                                     |      0 |        0 |
| [基于对象的 API](./patterns/ffi/export.md)                                              |      6 |        5 |
| [将类型整合到包装器中](./patterns/ffi/wrappers.md)                                      |      3 |        3 |
| [反模式](./anti_patterns/index.md)                                                      |      0 |        0 |
| [为了通过借用检查而克隆](./anti_patterns/borrow_clone.md)                               |      3 |        1 |
| [#![deny(warnings)]](./anti_patterns/deny-warnings.md)                                  |      3 |        3 |
| [Deref 多态](./anti_patterns/deref.md)                                                  |      3 |        3 |
| [Rust 的函数式用法](./functional/index.md)                                              |      0 |        0 |
| [编程范式](./functional/paradigms.md)                                                   |      2 |        2 |
| [将泛型用作类型类](./functional/generics-type-classes.md)                               |      7 |        4 |
| [函数式语言中的光学抽象](./functional/optics.md)                                        |     10 |       17 |
| [补充资源](./additional_resources/index.md)                                             |      0 |        0 |
| [设计原则](./additional_resources/design-principles.md)                                 |      3 |        0 |
| [重构](./refactoring/index.md)                                                          |      2 |        0 |

目录文件本身、README、贡献指南和文章模板也已翻译，但不计入上面的 51 个内容页。
许可证原文、其他语言翻译目录、第三方资源和发布工作流保持原状。阅读中文正文可直接使用本分支的
Markdown 文件。
