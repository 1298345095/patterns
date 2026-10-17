# 使用 `format!` 拼接字符串

## 描述

要构造字符串，可以对可变的 `String` 使用 `push` 和 `push_str`
方法，也可以使用它的 `+` 运算符。不过，使用 `format!`
往往更方便，尤其是在需要混合字符串字面量与其他字符串时。

## 示例

```rust
fn say_hello(name: &str) -> String {
    // 我们可以手动构造结果字符串。
    // let mut result = "Hello ".to_owned();
    // result.push_str(name);
    // result.push('!');
    // result

    // 不过，使用 format! 更好。
    format!("Hello {name}!")
}
```

## 优点

通常，`format!` 是组合字符串时最简洁、最易读的方式。

## 缺点

它通常不是组合字符串的最高效方式。在一个可变字符串上连续执行 `push`
操作，往往才是最高效的做法，尤其是在已经按预期大小预先分配了字符串容量的情况下。

> **译注（学习提示）**：这里的性能取舍主要在于“是否创建新的结果缓冲区”和“能否复用已有容量”。若结果本来就需要一个新的
> `String`，例子中的 `format!` 十分直接；若是在循环中不断追加片段，反复用
> `format!`
> 重建已有前缀则会产生额外复制。应结合实际数据规模比较，而不能仅凭宏或方法的名称判断快慢。参见
> [`String` 的容量说明](https://doc.rust-lang.org/std/string/struct.String.html#representation)。
