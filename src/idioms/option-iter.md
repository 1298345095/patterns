# 遍历 `Option`

## 描述

`Option` 可以看作一个包含零个或一个元素的容器。尤其是，它实现了 `IntoIterator`
trait，因此可以用于要求这类类型的泛型代码。

## 示例

由于 `Option` 实现了 `IntoIterator`，可以把它作为参数传给
[`.extend()`](https://doc.rust-lang.org/std/iter/trait.Extend.html#tymethod.extend)：

```rust
let turing = Some("Turing");
let mut logicians = vec!["Curry", "Kleene", "Markov"];

logicians.extend(turing);

// 等价于
if let Some(turing_inner) = turing {
    logicians.push(turing_inner);
}
```

> **译注（原文疑点）**：这里的 `extend` 和 `if let`
> 是两种可相互替代的写法。若把整个代码块原样执行，就会加入两次
> `"Turing"`。例子中的 `Option<&str>` 实现了 `Copy`，所以 `turing`
> 还能再次使用；若改成 `Option<String>`，按值传给 `extend` 就会移走它，后面的
> `if let` 也不能再使用原变量。

如果要把一个 `Option` 接在已有迭代器的末尾，可以把它传给
[`.chain()`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain)：

```rust
let turing = Some("Turing");
let logicians = vec!["Curry", "Kleene", "Markov"];

for logician in logicians.iter().chain(turing.iter()) {
    println!("{logician} is a logician");
}
```

> **译注（学习提示）**：`Option<T>`
> 的三种遍历方式分别保留不同的所有权语义：`into_iter()` 产出 `T`，`iter()` 产出
> `&T`，`iter_mut()` 产出 `&mut T`。`chain`
> 要求两边产出的元素类型相同，因此应先决定后续处理需要拥有元素还是借用元素，再选择遍历方式。本例两边都用
> `iter()`，实际产出的是 `&&str`。参见
> [`Option` 的迭代说明](https://doc.rust-lang.org/std/option/enum.Option.html#iterating-over-option)。

注意，如果这个 `Option` 总是 `Some`，那么直接对其中的元素使用
[`std::iter::once`](https://doc.rust-lang.org/std/iter/fn.once.html)
更符合惯用法。

此外，由于 `Option` 实现了 `IntoIterator`，也可以用 `for` 循环遍历它。这等价于用
`if let Some(..)` 进行匹配，大多数情况下应当优先选择后者。

## 另请参阅

- [`std::iter::once`](https://doc.rust-lang.org/std/iter/fn.once.html)
  返回一个恰好产出一个元素的迭代器。与 `Some(foo).into_iter()` 相比，它更易读。

- [`Iterator::filter_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter_map)
  是
  [`Iterator::map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.map)
  的一个专门版本，用于映射函数返回 `Option` 的情况。

- [`ref_slice`](https://crates.io/crates/ref_slice) crate 提供了将 `Option`
  转换为零元素或单元素切片的函数。

- [`Option<T>` 文档](https://doc.rust-lang.org/std/option/enum.Option.html)
