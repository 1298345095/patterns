# 传递字符串

## 描述

向 FFI 函数传递字符串时，应遵循四个原则：

1. 让拥有所有权的字符串存活得尽可能久。
2. 尽量减少转换过程中的 `unsafe` 代码。
3. 如果 C 代码可能修改字符串数据，使用 `Vec`，而不是 `CString`。
4. 除非外部函数 API 明确要求，否则不应把字符串的所有权转移给被调用方。

## 动机

Rust 通过 `CString` 和 `CStr` 类型内置了对 C 风格字符串的支持。不过，在 Rust
函数中向外部函数调用传递字符串时，可以采用不同的处理方式。

最佳实践很简单：使用 `CString`，并尽可能减少 `unsafe`
代码。不过，还要留意另一点：*对象必须存活足够长的时间*，也就是说，应尽可能延长其生命周期。此外，文档说明，对
`CString` 进行修改之后再“往返转换”会导致未定义行为，因此这种情况需要额外处理。

> **译注（原文疑点）**：原文对“修改之后再往返转换”的说法过于宽泛。`CString::into_raw`
> / `from_raw`
> 要求恢复时的字符串长度与原长度一致；在满足其他安全契约的前提下，并非任何内容修改都被禁止。另一方面，通过
> `CString::as_ptr`
> 得到的是只读指针，不能通过它修改底层字节。还应把“尽可能久”理解为覆盖外部代码的全部使用期间：若外部函数保存指针供以后使用，仅让局部
> `CString` 活到本次调用结束仍然不够。参见
> [`CString::from_raw`](https://doc.rust-lang.org/std/ffi/struct.CString.html#method.from_raw)
> 和
> [`CStr::as_ptr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.as_ptr)。

## 代码示例

```rust,ignore
pub mod unsafe_module {

    // 模块中的其他内容

    extern "C" {
        fn seterr(message: *const libc::c_char);
        fn geterr(buffer: *mut libc::c_char, size: libc::c_int) -> libc::c_int;
    }

    fn report_error_to_ffi<S: Into<String>>(err: S) -> Result<(), std::ffi::NulError> {
        let c_err = std::ffi::CString::new(err.into())?;

        unsafe {
            // SAFETY: 调用的 FFI 在文档中说明该指针为 const，
            // 因此不应发生修改。
            seterr(c_err.as_ptr());
        }

        Ok(())
        // c_err 的生命周期一直持续到这里
    }

    fn get_error_from_ffi() -> Result<String, std::ffi::IntoStringError> {
        let mut buffer = vec![0u8; 1024];
        unsafe {
            // SAFETY: 调用的 FFI 在文档中暗示，
            // 输入只需在本次调用期间保持有效。
            let written: usize = geterr(buffer.as_mut_ptr(), 1023).into();

            buffer.truncate(written + 1);
        }

        std::ffi::CString::new(buffer).unwrap().into_string()
    }
}
```

> **译注（原文疑点）**：`get_error_from_ffi` 原例不能直接作为可靠实现使用。通常
> `c_int` 是 `i32`，不能通过 `.into()` 无条件转成 `usize`；`*mut u8` 与平台上的
> `*mut c_char` 也不一定是同一类型。更关键的是，如果 `written`
> 不含终止符，`truncate(written + 1)` 会保留末尾零字节，而 `CString::new`
> 要求输入中没有任何零字节并自行追加终止符，于是这里的 `unwrap()`
> 会失败。真实接口还必须明确负返回值的含义、写入长度是否包含终止符、最大写入范围，并先验证这些条件；已有末尾零字节的缓冲区应按相应契约考虑
> `CString::from_vec_with_nul`。以上代码按原文保留。参见
> [`CString::new`](https://doc.rust-lang.org/std/ffi/struct.CString.html#method.new)。

### Rust 1.99.1 / Edition 2024 对照：声明外部函数与有界缓冲区

> **译注（版本说明）**：上方保留的 `extern "C" { ... }` 属于旧 edition 语法。
> 在 Rust 2024 Edition，外部声明块必须写成 `unsafe extern "C" { ... }`。
> 这里的 `unsafe` 是**声明者**对外部符号 ABI、参数及返回值签名正确性的承诺；
> 具体函数仍默认为不安全调用，只有确认对所有有效参数均安全时，
> 才能在声明中单独使用 `safe fn`。参见
> [Rust 2024 外部声明块](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)。

```rust,ignore
// 假设有匹配的 C 库导出 seterr；此片段本身不包含 C 端实现。
use std::ffi::{c_char, CString, NulError};

unsafe extern "C" {
    fn seterr(message: *const c_char);
}

fn report_error(err: &str) -> Result<(), NulError> {
    let owned = CString::new(err)?;
    // SAFETY: 假设 C 函数只在调用期间读取指针，不保存、不修改它。
    unsafe { seterr(owned.as_ptr()) };
    Ok(())
}
```

> **译注（学习提示）**：`CString` 活到调用结束，仅足以应对“同步借用”契约。
> 若 C 方会保存指针供以后使用，必须另外定义所有权转移、失效通知及释放方式，
> 不能把 `into_raw` 当成无条件的生命周期修复。对返回的数据，
> 原例 `geterr` 的返回值究竟是字节数、缓冲区大小还是错误码并不清楚，
> 所以不存在无需先了解 C 契约的普适修补版。

如果 C 方在**已初始化的固定大小缓冲区内**写入一个 NUL 结尾的字符串，
可以在确认写入范围与终止符后，用安全的 `from_bytes_until_nul`
进行有界解析，而不用将未验证的 `c_int` 直接转换成 `usize`：

```rust
use std::ffi::CStr;

fn decode_c_buffer(bytes: &[u8]) -> Result<&str, Box<dyn std::error::Error>> {
    let cstr = CStr::from_bytes_until_nul(bytes)?;
    Ok(cstr.to_str()?)
}

fn main() {
    assert_eq!(decode_c_buffer(b"hello\0unused").unwrap(), "hello");
}
```

> 这段安全的解码函数**不负责**保证外部函数写入时没有越界；
> 调用 C 之前仍要核对容量单位、负错误码、返回长度是否包含终止符等约定。

## 优点

这个示例的写法旨在保证：

1. `unsafe` 块尽可能小。
2. `CString` 存活足够长的时间。
3. 类型转换出错时，只要可能，就将错误向上传播。

一种常见错误是在第一个代码块中不使用局部变量；这种错误常见到连文档都专门提及：

```rust,ignore
pub mod unsafe_module {

    // 模块中的其他内容

    fn report_error<S: Into<String>>(err: S) -> Result<(), std::ffi::NulError> {
        unsafe {
            // SAFETY: 糟糕，这里包含一个悬垂指针！
            seterr(std::ffi::CString::new(err.into())?.as_ptr());
        }
        Ok(())
    }
}
```

这段代码会产生悬垂指针，因为创建指针并不会延长 `CString`
的生命周期，这与创建引用的情况不同。

> **译注（原文疑点）**：对于这里实际写出的表达式，上述结论并不成立：临时
> `CString` 会活到整条 `seterr(...);` 语句结束，所以若 `seterr`
> 只在本次调用中读取数据，调用期间的指针仍然有效。标准库文档展示的典型错误是先执行
> `let ptr = CString::new(...).unwrap().as_ptr();`，再在后续语句使用
> `ptr`；第一条语句结束时，临时值已经销毁。外部函数若保留指针，则本例也需要更长的所有者生命周期。此外，并非任意“创建引用”都会触发临时值生命周期延长，它遵循具体语法规则。参见
> [Rust Reference 的临时值作用域规则](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes)。

另一个经常被提起的问题是：初始化一个包含 1k 个零的向量“很慢”。不过，较新版本的
Rust 实际上会把这一特定的宏用法优化为对 `zmalloc`
的调用，因此它的速度等同于操作系统提供清零内存的速度，而这个速度相当快。

> **译注（原文疑点）**：这里对 `zmalloc`
> 的具体调用和性能保证不宜照单全收。`vec![0u8; n]`
> 可能使用分配器的清零分配路径，但 Rust
> 不承诺固定调用名，也不承诺其成本只取决于操作系统。分配器、平台、缓冲区大小和是否实际触碰内存都会影响性能。对这个
> FFI
> 场景，先用已初始化的缓冲区建立清晰的不变量，再根据实际测量判断是否需要优化，更容易正确实现。参见
> [`GlobalAlloc::alloc_zeroed`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html#method.alloc_zeroed)。

## 缺点

没有？
