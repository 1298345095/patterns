# `#![deny(warnings)]`

## 描述

一位出于好意的 crate 作者希望代码构建时不出现警告，于是在 crate 根模块中加上：

## 示例

```rust
#![deny(warnings)]

// 一切正常。
```

## 优点

写法简短，只要有任何不妥之处，就会停止构建。

## 缺点

禁止编译器在出现警告时继续构建，相当于主动放弃 Rust 广受称道的稳定性保障。
有时，新特性或旧特性中的设计问题会要求改变现有做法，因此会先引入相应的 lint，
在一段过渡期内发出 `warn` 级别的警告，之后再改为 `deny`。

例如，人们曾发现，同一个类型的两个 `impl` 可以定义相同的方法。
这种情况被认为不合理，但为了平稳过渡，Rust 引入了 `overlapping-inherent-impls`
这个 lint，先对遇到这种情况的代码发出警告，再在未来版本中将其升级为硬错误。

此外，API 有时会被弃用，之前没有警告的调用也就会开始触发警告。

这些因素共同导致：只要环境有所变化，构建就可能失败。

还有，除非移除这个属性，否则就无法再使用提供额外 lint 的 crate，例如
[rust-clippy]。 可以用 [--cap-lints] 缓解这个问题：命令行参数 `--cap-lints=warn`
会把所有 `deny` 级别的 lint 错误降为警告。

> **译注（原文疑点）**：这里的“无法再使用”说得过于绝对。问题在于， 新启用的 lint
> 一旦发出警告，`deny(warnings)` 就会把它升级为错误； 并非 Clippy
> 与该属性在机制上不能同时使用。Cargo 编译依赖时通常还会使用
> `--cap-lints allow`，因此依赖源码中的 `deny` 也并非总会导致下游构建失败。
> 还应区分 lint 级别变化和语言硬错误： `--cap-lints` 只约束
> lint，不能把类型错误等任意编译错误降为警告。 参见
> [rustc 的 lint 级别说明](https://doc.rust-lang.org/rustc/lints/levels.html#capping-lints)。

## 替代方案

解决这个问题有两种思路：一是把构建设置与代码分离，二是显式列出希望禁止的 lint。

以下命令会将所有警告设为 `deny`，然后构建项目：

`RUSTFLAGS="-D warnings" cargo build`

任何开发者都可以自行使用这个设置，也可以在 Travis 等 CI 工具中配置，
而无须修改代码。但请记住，环境变化时，它仍可能导致构建失败。

另一种做法是在代码中逐项指定要设为 `deny` 的 lint。
下面列出了一组可以相对放心地禁止的警告 lint——至少我们希望如此 （以 rustc 1.48.0
为准）：

```rust,ignore
#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
```

> **译注（版本说明）**：上面的列表是原文针对 rustc 1.48.0 给出的历史建议，
> 不是本译本对当前工具链的推荐配置。`const_err` 和 `private_in_public` 等 lint
> 后来已被移除或由其他检查替代。请保留对“逐项选择 lint”这一思路的理解，
> 并用当前工具链的 `rustc -W help` 检查具体名称；可核对
> [rustc 对已移除 lint 的登记](https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_lint/lib.rs.html)。

此外，以下默认设为 `allow` 的 lint，也可能值得改为 `deny`：

```rust,ignore
#![deny(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
```

有些人可能还想把 `missing-copy-implementations` 加入列表。

注意，我们有意没有加入 `deprecated`，因为未来几乎肯定还会有更多 API 被弃用。

> **译注（学习提示）**：可以分别考虑“发布给下游使用的源码应有多稳定”和
> “自己项目的 CI 应有多严格”。把严格警告策略放在自己控制的 CI 中，
> 有助于及时发现问题，又不会把同一策略写死在供别人编译的库源码里。
> 即使如此，升级 CI 的 Rust 版本时，也应预期新增 lint 需要处理。

## 另请参阅

- [全部 Clippy lint 的索引](https://rust-lang.github.io/rust-clippy/master)
- [`deprecated` 属性文档][deprecate attribute]
- 输入 `rustc -W help`，可查看当前系统支持的 lint 列表； 输入
  `rustc --help`，可查看常用选项。
- [rust-clippy] 提供了一组帮助改进 Rust 代码的 lint。

[rust-clippy]: https://github.com/rust-lang/rust-clippy
[deprecate attribute]: https://doc.rust-lang.org/reference/attributes.html#deprecation
[--cap-lints]: https://doc.rust-lang.org/rustc/lints/levels.html#capping-lints
