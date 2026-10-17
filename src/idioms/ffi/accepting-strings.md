# 接收字符串

## 描述

通过 FFI 以指针形式接收字符串时，应遵循两个原则：

1. 让外部字符串保持“借用”状态，而不是直接复制它们。
2. 将 C 风格字符串转换为 Rust 原生字符串时，尽量降低复杂度，减少所需的 `unsafe`
   代码。

## 动机

C 使用的字符串与 Rust 字符串有一些不同：

- C 字符串以空字节结尾，而 Rust 字符串会保存长度。
- C 字符串可以包含任意非零字节，而 Rust 字符串必须是 UTF-8。
- C 字符串通过 `unsafe` 指针操作来访问和修改，而 Rust
  字符串通过安全的方法进行操作。

Rust 标准库提供了与 `String` 和 `&str` 对应的 C 字符串类型，分别是 `CString` 和
`&CStr`。借助它们，我们可以避免在 C 字符串与 Rust
字符串之间转换时涉及的许多复杂细节和 `unsafe` 代码。

`&CStr` 类型还允许我们操作借用的数据，这意味着在 Rust 与 C
之间传递字符串是一项零开销操作。

> **译注（原文疑点）**：这里的“零开销”更适合理解为不必分配或复制字符串内容。`CStr::from_ptr`
> 目前仍需扫描终止空字节，`to_str` 还会验证
> UTF-8；因此，整个转换流程并不保证是常数时间。参见
> [`CStr::from_ptr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.from_ptr)
> 和
> [`CStr::to_str`](https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.to_str)。

## 代码示例

```rust,ignore
pub mod unsafe_module {

    // 模块中的其他内容

    /// 以指定级别记录一条消息。
    ///
    /// # 安全性
    ///
    /// 调用者必须保证 `msg`：
    ///
    /// - 不是空指针
    /// - 指向有效且已初始化的数据
    /// - 指向以空字节结尾的内存区域
    /// - 在本次函数调用期间不会被修改
    #[no_mangle]
    pub unsafe extern "C" fn mylib_log(msg: *const libc::c_char, level: libc::c_int) {
        let level: crate::LogLevel = match level { /* ... */ };

        // SAFETY: 调用者已保证此操作满足安全要求（参见文档注释中的
        // “安全性”一节）。
        let msg_str: &str = match std::ffi::CStr::from_ptr(msg).to_str() {
            Ok(s) => s,
            Err(e) => {
                crate::log_error("FFI string conversion failed");
                return;
            }
        };

        crate::log(msg_str, level);
    }
}
```

> **译注（学习提示）**：从裸指针构造引用，是把人工维护的安全契约交给 Rust
> 类型系统继续跟踪，而不是让编译器替你验证外部内存。调用者还必须保证从指针到终止空字节的整个区域可读、位于同一分配中，并满足长度上限；返回引用的生命周期不能超过该区域实际有效且不被修改的时间。如果日志系统要在函数返回后异步保存消息，就需要复制到自己拥有的存储中。参见
> [`CStr::from_ptr` 的安全要求](https://doc.rust-lang.org/std/ffi/struct.CStr.html#safety)。

### Rust 1.99.1 / Edition 2024 对照：局部证明与导出符号

> **译注（版本说明）**：上面的 `#[no_mangle]` 保留原文，但在 Rust 2024
> Edition 中必须写为 `#[unsafe(no_mangle)]`；被导出的名称属于全局符号命名空间，
> 需要确认不会与其他库的符号冲突。`unsafe fn` 也不再隐式允许函数体内任意不安全操作：
> `unsafe_op_in_unsafe_fn` 现在默认报警，应把 `CStr::from_ptr` 放进单独的
> `unsafe {}`，明确局部证明义务。仅添加语法标记**不会**修复无效指针、非 NUL
> 终止字符串或违反别名规则等问题。参考 [unsafe 属性](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html)、
> [unsafe 函数内的操作](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html)。

以下是可独立使用标准库编译的最小对照例子；返回 `bool` 只演示 UTF-8 检查，
不是一个完整的日志接口：

```rust
use std::ffi::{c_char, CStr, CString};

/// # Safety
/// `msg` 必须非空，指向同一分配中有效且以 NUL 结尾的可读字节；
/// 在本次调用中不得修改这些字节。
// SAFETY: 集成到最终程序前须确认此导出符号在链接产物中唯一。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_patterns_zh_validate_utf8(msg: *const c_char) -> bool {
    // SAFETY: 由本函数的调用者保证上述指针与内存契约。
    let message = unsafe { CStr::from_ptr(msg) };
    message.to_str().is_ok()
}

fn main() {
    let msg = CString::new("你好，Rust").unwrap();
    // SAFETY: CString 在调用结束前一直存活，且不会被修改。
    assert!(unsafe { rust_patterns_zh_validate_utf8(msg.as_ptr()) });
}
```

> **译注（学习提示）**：这里把责任划成三层：导出符号的全局唯一性、
> 外部调用者对裸指针的保证、函数体中具体的不安全操作。
> `CStr::from_ptr` 必须扫描终止 NUL；若 C 方已经给出可信长度，
> 应优先建立**有界**缓冲区协议，并校验长度而非盲目从指针扫描。
> 若回调或日志系统会在函数返回后保存消息，必须复制到拥有型存储。

## 优点

这个示例的写法保证了：

1. `unsafe` 块尽可能小。
2. 生命周期“不受跟踪”的指针转换成了生命周期“受到跟踪”的共享引用。

再看另一种确实会复制字符串的实现：

```rust,ignore
pub mod unsafe_module {

    // 模块中的其他内容

    pub extern "C" fn mylib_log(msg: *const libc::c_char, level: libc::c_int) {
        // 不要使用这段代码。
        // 它既难看又冗长，还藏着一个不易察觉的错误。

        let level: crate::LogLevel = match level { /* ... */ };

        let msg_len = unsafe { /* SAFETY: strlen 就是这么回事吧，我猜？ */
            libc::strlen(msg)
        };

        let mut msg_data = Vec::with_capacity(msg_len + 1);

        let msg_cstr: std::ffi::CString = unsafe {
            // SAFETY: 从外部指针复制到拥有所有权的内存中，假定该指针
            // 在当前栈帧的整个存续期间都有效。
            std::ptr::copy_nonoverlapping(msg, msg_data.as_mut(), msg_len);

            msg_data.set_len(msg_len + 1);

            std::ffi::CString::from_vec_with_nul(msg_data).unwrap()
        }

        let msg_str: String = unsafe {
            match msg_cstr.into_string() {
                Ok(s) => s,
                Err(e) => {
                    crate::log_error("FFI string conversion failed");
                    return;
                }
            }
        };

        crate::log(&msg_str, level);
    }
}
```

这段代码在两个方面不如前一个版本：

1. 它包含更多 `unsafe` 代码；更重要的是，它必须维护更多不变量。
2. 由于需要进行大量算术计算，这个版本中出现了一个会导致 Rust
   未定义行为（`undefined behaviour`）的错误。

这里的错误很简单：指针算术出了差错。字符串的全部 `msg_len`
个字节都复制了，但末尾的 `NUL` 终止符没有复制。

随后，代码把向量的长度直接*设置*成了*包含末尾补零的字符串*的长度，而不是通过*调整长度并初始化新增元素*的方式扩展它；后者本可以在末尾补上一个零。因此，向量的最后一个字节仍是未初始化的内存。在这个块的末尾创建
`CString` 时，对向量的读取就会引发未定义行为！

和许多类似问题一样，这个问题也很难定位。有时它会因为字符串不是 `UTF-8` 而
panic，有时会在字符串末尾出现一个奇怪的字符，有时则会直接彻底崩溃。

> **译注（原文疑点）**：原例还存在与核心错误无关的代码问题，例如 `msg_cstr`
> 初始化之后缺少分号，`as_mut()` 也不等同于取得裸指针的
> `as_mut_ptr()`；这里均按原文保留。就作者要说明的问题而言，`set_len`
> 本身便要求新增长度覆盖的元素都已初始化，违反这个前提后，行为不再有保证，不能依靠上面列举的
> panic、乱码或崩溃来判断是否发生了错误。参见
> [`Vec::set_len` 的安全要求](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.set_len)。

## 缺点

没有？
