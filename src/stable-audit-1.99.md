# Rust 1.99.1 / 2024 Edition 兼容性审校

> **译者新增审校记录**，不属于英文原书。检查日期：2026-10-17（UTC）。
> 对照 Rust stable **1.99.1**（2026-10-15 发布）和 Rust **2024 Edition**。
> 原书基线为 `36fdd17`，中文初译基线为 `ce489e4`；本次仅补充、标记和并列展示现代写法，
> 不把原文历史代码改成另一套实现。

## 先区分：语言版本、Edition 与代码是否完整

Rust **1.99.1** 是编译器工具链版本；**2024 Edition** 是选择部分语言规则的兼容性边界。
旧代码是否需要改动取决于实际 edition。例如 `#[no_mangle]` 在旧 edition
中仍可使用，但在 2024 Edition 下要求 `#[unsafe(no_mangle)]`。
本仓库的 `book.toml` 已设定 `edition = "2024"`，
因此书中故意标记为 `rust,ignore` 的旧 FFI 片段尤其值得注意：
它们不会自动进入 mdBook 的编译验证范围。

原书的绝大多数模式（RAII、Newtype、Builder、Visitor、
`mem::take` / `mem::replace`、`#[non_exhaustive]`、trait 对象与泛型等）
**没有被 stable 废弃**。下面列出的是需要保留历史语境、更新写法或
明确 soundness 条件的地方，不是建议机械替换全书。

## 审校结果：按优先级阅读

| 类别 | 位置 | stable / Edition 2024 结论 |
| --- | --- | --- |
| 必须更新语法（针对 2024） | [FFI 接收字符串](./idioms/ffi/accepting-strings.md) | 原例的 `#[no_mangle]` 需改用 `#[unsafe(no_mangle)]`；`unsafe fn` 内的 `CStr::from_ptr` 需要局部 `unsafe {}` |
| 必须更新语法（针对 2024） | [FFI 返回错误](./idioms/ffi/errors.md) | 原例 `#[no_mangle]` 不合 2024 属性写法；另补成对的 `CString` 所有权转移／释放示例 |
| 必须更新语法（针对 2024） | [基于对象的 C API](./patterns/ffi/export.md) | 原例三个 `#[no_mangle]` 不合 2024 属性写法，且原始示例自身未补全；另补独立、不透明的句柄创建／释放对照 |
| 必须更新语法（针对 2024） | [向 FFI 传递字符串](./idioms/ffi/passing-strings.md) | `extern "C" {}` 外部声明块需要 `unsafe extern "C" {}`；原例返回长度及 NUL 处理还存在独立问题 |
| 历史建议 | [`deny(warnings)`](./anti_patterns/deny-warnings.md) | rustc 1.48.0 的旧 lint 白名单不适合作为 1.99.1 的配置；补充 CI、Clippy、`#[expect]` 与升级策略 |
| 改变作用域 / 析构行为 | [析构与 finally](./idioms/dtor-finally.md)、[栈上动态分发](./idioms/on-stack-dyn-dispatch.md) | Rust 2024 对尾表达式和部分 `if let` 临时值的销毁时机作了调整；不能随意推广 Rust 1.79 的临时生命周期延长 |
| 标准库示意与历史链接 | [RAII](./patterns/behavioural/RAII.md) | 原例不是实际 `Mutex` 实现；补标准库 `MutexGuard` 及 poisoned lock 的区别 |
| 已变化的生态事实 | [小型 crate](./patterns/structural/small-crates.md) | crates.io 在 2025 年已支持有条件删除 crate；`yank` 仍不等于删除 |
| 已稳定的新能力 | [`Deref` 多态](./anti_patterns/deref.md) | Rust 1.86 稳定 trait object upcasting，但不等于结构体继承，也不能证明 `Deref` 适合模拟子类 |
| 已稳定的新能力 | [泛型与类型类](./functional/generics-type-classes.md)、[Optics](./functional/optics.md) | GAT（1.65）、trait 方法返回 `impl Trait`（1.75）及 2024 `impl Trait` 捕获规则扩大了表达空间；原文泛型限制不能当作当前语言能力结论 |

## FFI：语法更新与 soundness 是两项工作

### 1. 声明端：`unsafe extern` 不意味着可以随意调用

在 Rust 2024 中，`unsafe extern "C" { fn f(...); }`
表明**声明者**承担 ABI 与签名正确的责任；被声明函数默认为 `unsafe`，
除非明确标为 `safe fn`。若 C 接口要求有效指针、活跃对象或特定缓冲区长度，
不能只凭声明正确就将其标为安全函数。参见
[Edition Guide：unsafe extern](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)。

### 2. 导出端：unsafe 属性负责符号，不负责指针安全

`#[unsafe(no_mangle)]` 还要求集成者验证全局符号没有冲突；
它**不证明**导出函数收到的裸指针有效，也不保证与 C 内存布局兼容。
对象的裸指针是否指向正确的分配、是否还存活、是否正被其他线程读取，
仍需要单独设计协议。参见
[Edition Guide：unsafe 属性](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html)。

### 3. 生命周期、别名、字符编码和分配器

读入 C 字符串时，需要保证非空、NUL 终止、完整字节区域可读且有效，
并确保外部代码在 Rust 借用期间不修改它。
`CStr::from_ptr` 要扫描到 NUL，单次调用同步完成和“C 函数保留裸指针供以后使用”
是不同协议；如果调用方可以给出有界缓冲区，先验证长度并考虑
`CStr::from_bytes_until_nul`。错误码必须区分负值、字节数、
含／不含终止符的长度；不能用无条件 `into::<usize>()` 推断 C 接口。

返回给 C 的内存应约定**谁分配、谁释放、哪个函数回收、是否允许修改、重复释放怎么办**。
`CString::into_raw` 需要与 Rust 的 `CString::from_raw` 配对，
不能直接交给 C 的 `free`；由 C 的 `malloc` 创建的内存也不能
交给 `CString::from_raw`。即使有空指针检查，也无法自动发现悬垂指针。
跨普通 `extern "C"` 边界发生 Rust panic 时通常会终止进程；
允许跨语言栈展开还需要检查 `C-unwind` ABI 及双方异常模型。
参见 [CStr 安全要求](https://doc.rust-lang.org/std/ffi/struct.CStr.html#safety)、
[CString 的所有权规则](https://doc.rust-lang.org/std/ffi/struct.CString.html#method.from_raw)、
[Rust Reference：FFI unwind](https://doc.rust-lang.org/reference/panic.html#unwinding-across-ffi-boundaries)。

## Edition 2024 的临时值析构顺序

Rust 2024 调整了块／函数**尾表达式**及部分 `if let` 中临时值的作用域。
这既可能让原本借用检查不通过的代码通过，也可能让依赖旧延长规则的
表达式不再通过。它并未改变 RAII 的基本思想：
作用域正常离开以及栈展开时，活跃守卫依然按析构规则清理；
但对精确释放顺序敏感时应显式绑定守卫。
参见 [2024 尾表达式](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html)、
[2024 `if let` 临时值](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html)。

## 泛型、动态分发与现代编写方式

泛型单态化不表示“任何情况都更快”或“二进制一定变大”。
GAT 适合建模有借用生命周期的关联视图；trait 方法返回 `impl Trait`
则允许返回静态分发的抽象具体类型，仍需评估对象安全性和公开 API
的可表达边界。Rust 1.86 的 trait object upcasting 使
`&dyn Sub` 可以转为 `&dyn Super`，但并没有引入类继承。
Rust 2024 返回位置 `impl Trait` 的自动捕获规则还可能改变
复杂生命周期 API 的类型推断；必要时可使用精确捕获语法 `use<...>`。

## 代码示例核验说明

- **完整保留**英文基线的 100 个原始代码、命令和输出块；原版 `rust,ignore` 与
  `compile_fail` 标记不改动。新写的现代示例是并列示范，
  不代表原文示例已变成可直接编译的版本。
- 新增一个不依赖第三方 crate 的 [Rust 2024 独立自检文件](../checks/rust_2024_smoke.rs)，
  包括 `#[unsafe(no_mangle)]`、局部 `unsafe`、
  `unsafe extern` 声明、FFI 所有权配对、GAT、trait upcasting
  和临时值析构场景。运行方式：

```sh
rustup update stable
rustc --version
rustc --edition=2024 --test checks/rust_2024_smoke.rs -o /tmp/rust_2024_smoke
/tmp/rust_2024_smoke
```

- **验证状态**：原译本 `ce489e4` 已记录过 mdBook 及 rustdoc 测试通过。
  本次会话的执行环境**没有安装 rustc 或 mdbook，无法实跑新自检文件**；
  新示例仅完成静态检查与原文保留检查。不能把先前的测试结果
  当作本次新增代码已通过编译的证据。今后若测试失败，应
  根据编译器诊断修正**新增示例**，而不是悄悄篡改原书代码。
- 本次校阅不替代 Miri、ASan、Valgrind、跨平台 ABI 验证或
  完整 C 调用方集成测试；原书中不完整的 FFI / AST 示意仍不适合作生产实现。

## 版本来源与继续跟进

- [Rust 1.99.1 正式发布（2026-10-15）](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/)
- [Rust 2024 Edition 指南](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [Rust 1.86：trait upcasting](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/#trait-upcasting)
- [GAT 稳定说明](https://blog.rust-lang.org/2022/10/28/gats-stabilization/)
- [trait 中返回 `impl Trait` 稳定说明](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/)
- [crates.io 有条件删除（2025-02）](https://blog.rust-lang.org/2025/02/05/crates-io-development-update/#crate-deletions)
