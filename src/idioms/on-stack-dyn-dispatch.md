# 在栈上进行动态分发

## 描述

我们可以针对多个值进行动态分发。不过，为此需要声明多个变量，分别绑定不同类型的对象。要根据需要延长它们的生命周期，可以使用延迟的条件初始化，如下所示：

## 示例

```rust
use std::io;
use std::fs;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let arg = "-";

// 需要明确写出类型，才能使用动态分发。
let readable: &mut dyn io::Read = if arg == "-" {
    &mut io::stdin()
} else {
    &mut fs::File::open(arg)?
};

// 在这里从 `readable` 读取数据。

# Ok(())
# }
```

## 动机

Rust
默认会对代码进行单态化：针对每一种实际使用的类型，生成一份代码并分别优化。这能够让热点路径上的代码运行得很快，但在性能并非关键的地方，也会导致代码膨胀，从而增加编译时间和缓存占用。

好在 Rust 允许使用动态分发，只是需要我们明确提出这一要求。

> **译注（学习提示）**：这里的单态化主要针对以具体类型实例化的泛型代码。统一成
> `&mut dyn io::Read` 后，后续调用通过 trait
> 对象提供的分发信息选择实现，而不必让后续整段处理流程都以 `R: Read`
> 参数化。代价包括间接调用，以及编译器通常更难跨越这种调用进行内联优化；是否值得取决于实际热点，而不是“动态”一词本身。参见
> [trait 对象](https://doc.rust-lang.org/reference/types/trait-object.html)。

## 优点

我们无需在堆上分配任何内容，也不必初始化之后不会使用的对象，更不需要为同时支持
`File` 与 `Stdin`，对后续的整段代码进行单态化。

> **译注（原文疑点）**：“无需在堆上分配任何内容”应限定为不必为了保存这个 trait
> 对象而额外使用 `Box`。具体的 I/O
> 类型及其内部操作是否分配内存，是另一个问题。这个例子展示的是“动态分发不要求拥有型的堆分配”，并不保证整段程序完全没有堆分配。

## 缺点

在 Rust 1.79.0 之前，这段代码需要两个延迟初始化的 `let` 绑定，因此比下面基于
`Box` 的版本多出一些需要处理的细节：

```rust,ignore
// 仍然需要显式标注类型，以使用动态分发。
let readable: Box<dyn io::Read> = if arg == "-" {
    Box::new(io::stdin())
} else {
    Box::new(fs::File::open(arg)?)
};
// 在这里从 `readable` 读取数据。
```

好在如今这个缺点已经不存在了。太好了！

## 讨论

从 Rust 1.79.0 开始，编译器会自动将 `&` 或 `&mut`
中临时值的生命周期尽可能延长到函数作用域允许的范围。

> **译注（原文疑点）**：这不是“编译器会按引用需求任意延长所有临时值”的一般规则。Rust
> 1.79.0 扩展的是特定语法位置上的临时值生命周期延长，使上述 `let` 初始化器内
> `if`／`match` 分支末尾直接借用临时值的写法也能适用；通常延长到包含该 `let`
> 的代码块结束，而非必然到整个函数结束。通过普通函数调用间接返回借用，不会自动获得相同待遇。导言所说的“多个变量、延迟初始化”描述的是旧做法，如今的主示例已经不需要那些显式绑定。参见
> [1.79.0 发布说明](https://blog.rust-lang.org/2024/06/13/Rust-1.79.0/#extending-automatic-temporary-lifetime-extension)
> 和
> [临时值生命周期延长规则](https://doc.rust-lang.org/reference/destructors.html#temporary-lifetime-extension)。

这意味着，我们可以直接在这里使用 `&mut` 值，无需操心把引用指向的内容先保存到某个
`let` 绑定中。在这一变化之前，解决办法正是使用这样的绑定来进行延迟初始化。

每个值仍然有各自的存储位置，即使这个位置是临时的。编译器知道每个值的大小，而且每个被借用的值都会比借用它的所有引用存活得更久。

## 另请参阅

- [在析构函数中执行收尾工作](dtor-finally.md)和
  [RAII 守卫](../patterns/behavioural/RAII.md)都能从精确控制生命周期中受益。
- 如果需要按条件构造一个装有可变或不可变引用的 `Option<&T>`，可以先直接初始化
  `Option<T>`，然后使用它的 [`.as_ref()`] 方法取得可选的引用。

> **译注（原文疑点）**：这里提到了可变引用，但 `as_ref()` 返回的是
> `Option<&T>`，其中只有共享引用。如果需要 `Option<&mut T>`，应对可变的
> `Option<T>` 使用
> [`as_mut()`](https://doc.rust-lang.org/std/option/enum.Option.html#method.as_mut)。两种方法都借用原有内容，不会把其中的
> `T` 移走。

[`.as_ref()`]: https://doc.rust-lang.org/std/option/enum.Option.html#method.as_ref
