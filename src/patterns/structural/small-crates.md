# 优先使用小型 crate

## 描述

优先使用只专注于做好一件事的小型 crate。

Cargo 和 crates.io 让引入第三方库变得很容易，远比在 C 或 C++ 等语言中方便。
此外，由于 crates.io
上的包发布后不能修改或删除，任何现在能够成功的构建，在将来也应当继续成功。
我们应该充分利用这些工具，采用更小、粒度更细的依赖。

> **译注（原文疑点）**：原文“发布后不能删除”是历史表述，不能作为当前的绝对保证：crates.io
> 已支持在特定条件下真正删除 crate，参见
> [2025 年的官方开发更新](https://blog.rust-lang.org/2025/02/05/crates-io-development-update/#crate-deletions)。删除与
> `cargo yank` 是不同操作；仅仅 yank
> 不会删除代码，已有锁文件仍可使用该版本，但新的解析通常不会再选择它，参见
> [Cargo 关于 `yank` 的说明](https://doc.rust-lang.org/cargo/reference/publishing.html#cargo-yank)。锁文件固定的是解析结果，并不是源码的永久备份。即使源码仍可取得，编译器、目标平台和系统库的变化，也可能让未来的构建失败。

### crates.io 的当前行为与长期可复现性

> **译注（版本说明）**：这里“发布后不能删除”的断言不再绝对成立。
> crates.io 在 **2025 年**引入有条件删除 crate 的功能：
> 官方发布说明列出的情形包括发布不足 72 小时，或在满足所有权、
> 下载量及无下游依赖等要求时删除。此处是对该次政策变化的说明，
> 未来具体限制应以 crates.io 当时的管理界面和规则为准。
> 删除与 `cargo yank`（仅影响新解析）不是同一操作。
> 参见 [crates.io 2025 开发更新](https://blog.rust-lang.org/2025/02/05/crates-io-development-update/#crate-deletions)。

> **译注（学习提示）**：即便 crate 没有被删除，`Cargo.lock` 也只固定
> 依赖的版本解析结果，不能固定 rustc、目标平台、系统库、编译脚本
> 或操作系统。长期重复构建还应保存源码与依赖缓存，明确工具链，
> 并根据需要记录目标平台及原生依赖。反过来，拆分为大量小 crate
> 本身也不自动带来更可复现或更快的编译：仍要评估构建脚本、
> feature 统一及公开类型随版本传播等代价。

## 优点

- 小型 crate 更容易理解，也会鼓励更模块化的代码组织。
- crate 便于跨项目复用代码。例如，`url` crate 最初作为 Servo
  浏览器引擎的一部分开发，后来在项目之外也得到了广泛使用。
- Rust 的编译单元是 crate，因此把一个项目拆成多个
  crate，可以让更多代码并行编译。

## 缺点

- 这可能导致“依赖地狱”：一个项目同时依赖某个 crate 的多个不兼容版本。
  例如，`url` crate 存在 1.0 和 0.5 两个版本。由于 `url:1.0` 的 `Url` 与
  `url:0.5` 的 `Url` 是不同类型， 一个使用 `url:0.5` 的 HTTP
  客户端，就无法接收一个使用 `url:1.0` 的网页抓取器所提供的 `Url` 值。
- crates.io 上的包并不经过统一的质量筛选。一个 crate
  可能实现质量很差、文档帮助不大，甚至包含恶意行为。
- 两个小型 crate 的优化效果可能不如一个大型
  crate，因为编译器默认不执行链接时优化（LTO）。

> **译注（原文疑点）**：这里关于 LTO 的说法应限定为默认不做跨 crate 的
> LTO。Cargo 的 `lto = false`
> 在满足相应优化等级和代码生成单元条件时，仍可能在当前 crate 内执行局部
> ThinLTO；`lto = "off"` 才是明确禁用。crate
> 边界对性能的实际影响还取决于泛型、内联和具体构建配置。参见
> [Cargo 的 `lto` 配置](https://doc.rust-lang.org/cargo/reference/profiles.html#lto)。
>
> **译注（学习提示）**：小型 crate
> 的收益来自边界明确，而不是数量多。同一个依赖的不同版本通常可以共存，但一旦把其中的类型暴露在公开
> API 中，版本差异就会传播给调用方。划分 crate
> 时，可以同时检查：这部分功能是否有独立的职责与依赖，以及公开接口是否把内部依赖类型带到了边界之外。

## 示例

[`url`](https://crates.io/crates/url) crate 提供处理 URL 的工具。

[`num_cpus`](https://crates.io/crates/num_cpus) crate 提供查询机器 CPU
数量的函数。

[`ref_slice`](https://crates.io/crates/ref_slice) crate 提供将 `&T` 转换成
`&[T]` 的函数。（历史示例）

## 另请参阅

- [crates.io：Rust 社区的 crate 托管平台](https://crates.io/)
