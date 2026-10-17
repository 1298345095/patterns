# 用自定义 trait 简化复杂的类型约束

## 描述

trait 约束可能变得难以掌握，尤其是涉及某个 `Fn` 系列 trait[^fn-traits]，
并且对输出类型有特定要求的时候。这时，引入一个新的 trait，可以减少冗长的表达、
消除部分类型参数，从而增强表达力。还可以为所有满足原有约束的类型提供一个泛型
`impl`， 让它们自动实现这个新 trait。

## 示例

假设有一个监控或信息收集系统。系统从不同来源获取不同类型的值，
并根据这些值计算出反映问题的状态。例如，可用内存总量应当高于某个阈值， 而 ID 为
`0` 的用户应当始终名为“root”。

为了便于统一管理，我们可能希望在最外层进行类型擦除。
不过，针对特定类型的数据源，又需要提供特定的、可由用户配置的评估规则，
例如为数值类型设置阈值和范围。由于这些值来自不同来源，
我们可能会把数据源表示成闭包，调用闭包时就返回一个值。
这些值很可能需要从操作系统获取，因此也会涉及可能失败的操作。

于是，我们可能会采用下面这些类型与 trait 约束来处理具体的值：

```rust
use std::fmt::Display;

struct Value<G: FnMut() -> Result<T, Error>, S: Fn(&T) -> Status, T: Display> {
    value: Option<T>,
    getter: G,
    status: S,
}

impl<G: FnMut() -> Result<T, Error>, S: Fn(&T) -> Status, T: Display> Value<G, S, T> {
    pub fn update(&mut self) -> Result<(), Error> {
        (self.getter)().map(|v| self.value = Some(v))
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn status(&self) -> Option<Status> {
        self.value().map(&self.status)
    }
}

// ...

enum Status {
    // ...
}

struct Error {
    // ...
}
```

采用这些类型后，我们至少需要在几个地方重复 `G` 的 trait 约束。
这会影响可读性，其中一个原因是获取值的函数返回了 `Result`。
为“获取器”（getter）引入一个专用的约束，可以更清晰地表达意图，并消除一个类型参数：

```rust
# use std::fmt::Display;
trait Getter {
    type Output: Display;

    fn get_value(&mut self) -> Result<Self::Output, Error>;
}

impl<F: FnMut() -> Result<T, Error>, T: Display> Getter for F {
    type Output = T;

    fn get_value(&mut self) -> Result<Self::Output, Error> {
        self()
    }
}

struct Value<G: Getter, S: Fn(&G::Output) -> Status> {
    value: Option<G::Output>,
    getter: G,
    status: S,
}

// ...
# enum Status {}
# struct Error;
```

> **译注（学习提示）**：关联类型 `Output`
> 表达了“每一种获取器都有一个由自身决定的输出类型”这一关系。于是调用方只需写
> `G::Output`，不必再单独携带 `T`。这里的整体设计仍然可以是泛型和静态分发；引入
> `Getter` 这个名字，并不会自动引入装箱或动态分发。
>
> **译注（学习提示）**：示例中的泛型实现会让所有符合签名的闭包和函数自动实现
> `Getter`，所以普通调用者无须自己包装它们。不过，这个实现也占用了相应的 trait
> 实现空间：对已经匹配该泛型实现的类型，再写一个重叠的 `Getter`
> 实现会产生冲突。若某个获取器需要特殊行为，可以用一个独立的结构体或新类型表达，而不是假定总能覆盖现有实现。

## 优点

引入新 trait 有助于简化类型约束，尤其是通过消除类型参数来实现简化。 为新 trait
取一个好名字，还能让约束更有表达力。 新 trait
本身也是一种抽象，因此还带来了进一步扩展的可能，包括：

- 增加实现这个 trait 的专用类型，例如表示某种身份标识的类型；这些类型还可以实现
  `Default` 等其他有用的 trait。
- 增加方法，前提是所有相关类型都能够实现这些方法。

## 缺点

引入 trait 等新项，就需要为它们找到合适的名称和位置。
这也意味着，原有功能的使用者需要额外了解一个新的项[^read-docs]。
取决于文档如何呈现，使用者可能无法立刻看出：上例中的 `Getter`
可以直接使用一个简单闭包。

> **译注（学习提示）**：开头提到的“在最外层做类型擦除”并没有在这段代码中完成。若改用
> `dyn Getter`，仍需要指定关联类型，例如
> `dyn Getter<Output = u64>`，因此不能仅靠这个 trait
> 就把输出类型任意不同的获取器放在同一集合里。一个可行的分层思路是：内部保留具体的
> `Output`
> 和对应评估规则，外层再暴露只返回统一状态或格式化结果的接口。这样既能在内部保持类型关系，也能在外部统一管理。

[^fn-traits]: 即 `Fn`、`FnOnce` 和 `FnMut`。

[^read-docs]: 也就是说，他们可能需要阅读更多文档。
