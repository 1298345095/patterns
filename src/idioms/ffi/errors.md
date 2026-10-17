# FFI 中的错误处理

## 描述

在 C 等外部语言中，错误通常由返回码表示。不过，Rust
的类型系统可以通过完整的类型，记录并传播丰富得多的错误信息。

这项最佳实践介绍不同种类的错误码，以及如何以便于使用的方式将它们暴露出去：

1. 无数据枚举应转换为整数，作为错误码返回。
2. 带数据的枚举应转换为整数错误码，并附带字符串错误消息来说明细节。
3. 自定义错误类型应采用 C 表示，成为“透明”类型。

## 代码示例

### 无数据枚举

```rust,ignore
enum DatabaseError {
    IsReadOnly = 1,    // 用户尝试执行写操作
    IOError = 2,       // 用户应读取 C 的 errno()，了解具体错误
    FileCorrupted = 3, // 用户应运行修复工具来恢复文件
}

impl From<DatabaseError> for libc::c_int {
    fn from(e: DatabaseError) -> libc::c_int {
        (e as i8).into()
    }
}
```

### 带数据的枚举

```rust,ignore
pub mod errors {
    enum DatabaseError {
        IsReadOnly,
        IOError(std::io::Error),
        FileCorrupted(String), // 描述问题的消息
    }

    impl From<DatabaseError> for libc::c_int {
        fn from(e: DatabaseError) -> libc::c_int {
            match e {
                DatabaseError::IsReadOnly => 1,
                DatabaseError::IOError(_) => 2,
                DatabaseError::FileCorrupted(_) => 3,
            }
        }
    }
}

pub mod c_api {
    use super::errors::DatabaseError;
    use core::ptr;

    #[no_mangle]
    pub extern "C" fn db_error_description(
        e: Option<ptr::NonNull<DatabaseError>>,
    ) -> Option<ptr::NonNull<libc::c_char>> {
        // SAFETY: 假定 `e` 的生命周期长于
        // 当前栈帧的存续时间。
        let error = unsafe { e?.as_ref() };

        let error_str: String = match error {
            DatabaseError::IsReadOnly => {
                format!("cannot write to read-only database")
            }
            DatabaseError::IOError(e) => {
                format!("I/O Error: {e}")
            }
            DatabaseError::FileCorrupted(s) => {
                format!("File corrupted, run repair: {}", &s)
            }
        };

        let error_bytes = error_str.as_bytes();

        let c_error = unsafe {
            // SAFETY: 将 error_bytes 复制到已分配的缓冲区，
            // 并在末尾写入一个 '\0' 字节。
            let buffer = ptr::NonNull::<u8>::new(libc::malloc(error_bytes.len() + 1).cast())?;

            buffer
                .as_ptr()
                .copy_from_nonoverlapping(error_bytes.as_ptr(), error_bytes.len());
            buffer.as_ptr().add(error_bytes.len()).write(0_u8);
            buffer
        };

        Some(c_error.cast())
    }
}
```

> **译注（原文疑点）**：原例中的 `DatabaseError` 没有公开，兄弟模块 `c_api`
> 无法直接导入它，代码按原文保留。安全性也不能仅由“生命周期足够长”推出：`NonNull`
> 只保证非空，不保证指针对齐、指向有效的 `DatabaseError`，或不存在冲突访问。若
> Rust 调用者需要承担这些前提，公开函数也应以 `unsafe`
> 契约表达，不能仅在内部注释中假定成立。第一段注释中的 `errno()` 也应理解为读取
> C 的 `errno`；它通常是宏提供的可修改左值，而非函数。把枚举转换成整数 `2`
> 不会自动设置或保存 `errno`；它是否对应本次失败，需要接口另行保证。
>
> **译注（学习提示）**：错误信息跨越 FFI
> 边界时，要把“错误类别”和“错误对象的生命周期”分开设计。此例用 `malloc`
> 分配描述文本，因此接口文档必须说明由谁、在何时、用哪个匹配的释放函数回收它；不能直接套用
> `CString::from_raw` 接管这块内存。如果消息里有内嵌零字节，C
> 调用者还会在第一个零处截断读取，需要在契约中明确处理方式。`Option<NonNull<T>>`
> 利用空指针表示 `None`，其布局保证可见
> [`Option` 的表示说明](https://doc.rust-lang.org/std/option/index.html#representation)，但这种布局保证不会证明指针所指对象有效。

### 自定义错误类型

```rust,ignore
struct ParseError {
    expected: char,
    line: u32,
    ch: u16,
}

impl ParseError {
    /* ... */
}

/* 创建第二个版本，以 C 结构体的形式公开 */
#[repr(C)]
pub struct parse_error {
    pub expected: libc::c_char,
    pub line: u32,
    pub ch: u16,
}

impl From<ParseError> for parse_error {
    fn from(e: ParseError) -> parse_error {
        let ParseError { expected, line, ch } = e;
        parse_error { expected, line, ch }
    }
}
```

> **译注（原文疑点）**：这里直接将 `expected: char` 填入
> `expected: libc::c_char`，存在类型不匹配。Rust 的 `char` 表示 Unicode
> 标量值，C 的 `char` 通常只容纳一个字节，不能无损对应所有 Unicode 字符。应先为
> C 接口决定编码契约，例如用固定宽度整数传递码点，或用 UTF-8
> 字节序列传递字符；不能用一次强制转换掩盖这个设计问题。另一个容易混淆之处是，上文的“透明”指调用者看得见字段的
> API 设计，代码实际使用的是 `#[repr(C)]`，并非 `#[repr(transparent)]`。参见
> [Rust `char` 文档](https://doc.rust-lang.org/std/primitive.char.html)。

### Rust 1.99.1 / Edition 2024 对照：错误消息的分配与释放协议

> **译注（版本说明）**：原例 `#[no_mangle]` 在 2024 Edition 中要写成
> `#[unsafe(no_mangle)]`。仅在 Rust 侧使用 `malloc` 并不能自动解决所有权问题；
> C 调用者必须知道该使用哪个释放函数，且不能二次释放。下面给出另一种
> **成对 API** 的最小实现，便于比较。它不是对上面 `DatabaseError` 例子的暗中修复。

```rust
use std::ffi::{c_char, CString};

/// 分配一条 UTF-8、NUL 结尾的错误消息；返回值由调用者负责释放。
// SAFETY: 集成时须确认此导出符号不与其他全局符号冲突。
#[unsafe(no_mangle)]
pub extern "C" fn rust_patterns_zh_error_text() -> *mut c_char {
    CString::new("database is read-only")
        .expect("静态文本不含内嵌 NUL")
        .into_raw()
}

/// # Safety
/// `ptr` 必须为空，或恰为本 API 返回且尚未释放的原始指针；
/// 指针对应的 C 字符串长度不能被 C 方修改，不能重复释放。
// SAFETY: 集成时须确认此导出符号不与其他全局符号冲突。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_patterns_zh_error_text_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        // SAFETY: 调用者负责遵守来源、有效性和唯一释放的约定。
        drop(unsafe { CString::from_raw(ptr) });
    }
}

fn main() {
    let text = rust_patterns_zh_error_text();
    // SAFETY: 正好释放上一行获取的指针一次。
    unsafe { rust_patterns_zh_error_text_free(text) };
}
```

> **译注（学习提示）**：这组 API 的内存由 Rust 的 `CString` 创建并回收，
> 不能让 C 侧直接调用 `free`。反过来，若内存由 C 的 `malloc` 分配，
> 也不能用 `CString::from_raw` 回收。`into_raw` / `from_raw`
> 配对的另一关键前提是 C 方不能改变 NUL 终止字符串的长度；
> 否则 Rust 重新接管时可能违反分配元数据要求。
> 真实 FFI 还应制定错误码与空指针语义、并处理跨 ABI 的 panic
> （通常不应让 Rust panic 穿过普通 `extern "C"` 边界）。参见
> [`CString::into_raw`](https://doc.rust-lang.org/std/ffi/struct.CString.html#method.into_raw)、
> [FFI unwind 规则](https://doc.rust-lang.org/reference/panic.html#unwinding-across-ffi-boundaries)。

## 优点

这样既能让外部语言清晰地访问错误信息，又完全不必牺牲 Rust 代码自身的 API 设计。

## 缺点

需要编写大量代码，而且有些类型可能不容易转换成 C 类型。
