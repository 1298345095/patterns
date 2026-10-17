# 在出错时返还已消耗的参数

## 描述

如果一个可能失败的函数会消耗，也就是移走某个参数，就在出错时把这个参数放进错误值中返回。

## 示例

```rust
pub fn send(value: String) -> Result<(), SendError> {
    println!("using {value} in a meaningful way");
    // 模拟一个结果不确定、可能失败的操作。
    use std::time::SystemTime;
    let period = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    if period.subsec_nanos() % 2 == 1 {
        Ok(())
    } else {
        Err(SendError(value))
    }
}

pub struct SendError(String);

fn main() {
    let mut value = "imagine this is very long string".to_string();

    let success = 's: {
        // 尝试发送 value 两次。
        for _ in 0..2 {
            value = match send(value) {
                Ok(()) => break 's true,
                Err(SendError(value)) => value,
            }
        }
        false
    };

    println!("success: {success}");
}
```

## 动机

发生错误时，你可能希望尝试其他办法；如果函数的结果具有不确定性，也可能希望重试。但如果参数无论成功失败都会被消耗，你就不得不在每次调用前克隆它，效率并不理想。

标准库也采用了这种方式，例如 `String::from_utf8`
方法。传入的向量如果不包含有效的 UTF-8 数据，就会返回 `FromUtf8Error`。调用
`FromUtf8Error::into_bytes` 方法，便可以取回原来的向量。

> **译注（学习提示）**：返回参数的所有权，只保证调用者有机会复用这份数据，并不自动保证失败前没有副作用。本例在决定返回
> `Ok` 还是 `Err` 之前就已经打印了一行内容。将这种 API
> 用于实际发送或写入操作时，需要另外明确：失败表示“完全未处理”，还是“可能已部分处理”；能取回参数与能安全重试是两个不同的契约。标准库的
> [`FromUtf8Error::into_bytes`](https://doc.rust-lang.org/std/string/struct.FromUtf8Error.html#method.into_bytes)
> 则明确允许无复制地取回原始字节。

## 优点

尽可能通过移动传递参数，性能更好。

## 缺点

错误类型会稍微复杂一些。

> **译注（学习提示）**：如果把示例中的错误类型做成库的公开
> API，还需要为外部调用者提供取回数据的途径，例如消耗错误值的
> `into_inner(self) -> String` 方法。`pub struct SendError(String)`
> 只公开了类型，元组字段仍是私有的；示例的 `main`
> 能解构它，是因为两者位于同一模块。这样的设计也有利于以后扩充错误信息，而不把存储布局暴露给调用者。参见
> [可见性与私有性](https://doc.rust-lang.org/reference/visibility-and-privacy.html)。
