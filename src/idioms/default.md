# `Default` trait

## 描述

Rust
中的许多类型都有[构造函数][constructor]。不过，构造函数是各个类型*专有*的；Rust
无法直接对“所有具有 `new()` 方法的类型”进行抽象。为了支持这样的抽象，Rust 引入了
[`Default`] trait，可供容器和其他泛型类型使用，例如
[`Option::unwrap_or_default()`]。值得一提的是，一些容器已经在适用时实现了这个
trait。

不仅 `Cow`、`Box`、`Arc` 这类只容纳一个元素的容器会在所容纳的类型实现 `Default`
时实现 `Default`，所有字段都实现了 `Default` 的结构体也可以通过
`#[derive(Default)]` 自动派生这个实现。因此，实现 `Default`
的类型越多，它就越有用。

> **译注（原文疑点）**：对 `Cow` 而言，上述概括不够精确。`Cow<'_, B>` 的
> `Default` 实现要求的是 `B::Owned: Default`，返回的也是
> `Cow::Owned`，并不要求借用目标 `B` 本身实现 `Default`。例如 `Cow<'_, str>`
> 可以默认构造为空的拥有型字符串。参见
> [`Cow` 的 `Default` 实现](https://doc.rust-lang.org/std/borrow/enum.Cow.html#impl-Default-for-Cow%3C'_,+B%3E)。

另一方面，构造函数可以接收多个参数，而 `default()`
方法不接收参数。一个类型甚至可以有多个名称不同的构造函数，但只能有一个 `Default`
实现。

## 示例

```rust
use std::{path::PathBuf, time::Duration};

// 注意，这里可以直接自动派生 Default。
#[derive(Default, Debug, PartialEq)]
struct MyConfiguration {
    // Option 的默认值是 None
    output: Option<PathBuf>,
    // Vec 的默认值是空向量
    search_path: Vec<PathBuf>,
    // Duration 的默认值是零时长
    timeout: Duration,
    // bool 的默认值是 false
    check: bool,
}

impl MyConfiguration {
    // 在这里添加设置字段值的方法
}

fn main() {
    // 使用默认值构造一个新实例
    let mut conf = MyConfiguration::default();
    // 在这里使用 conf 做些事情
    conf.check = true;
    println!("conf = {conf:#?}");

    // 用默认值补全初始化，创建与上面相同的实例
    let conf1 = MyConfiguration {
        check: true,
        ..Default::default()
    };
    assert_eq!(conf, conf1);
}
```

> **译注（学习提示）**：派生实现会逐字段调用各自的
> `Default::default()`，因此它只组合字段的默认值，不会替你验证字段之间的关系。假设配置要求“启用某项检查时必须指定输出路径”，字段分别有默认值也不代表组合后满足这个约束。此时应考虑手动实现
> `Default`，或者通过单独的校验或构造步骤建立不变量。参见
> [`Default` 的派生规则](https://doc.rust-lang.org/stable/std/default/trait.Default.html#derivable)。

## 另请参阅

- [构造函数][constructor]惯用法提供了另一种创建实例的方式，创建出的实例可以是“默认的”，也可以不是。
- [`Default`] 的文档，向下滚动可以查看实现它的类型列表。
- [`Option::unwrap_or_default()`]
- [`derive(new)`]

[constructor]: ctor.md
[`Default`]: https://doc.rust-lang.org/stable/std/default/trait.Default.html
[`Option::unwrap_or_default()`]: https://doc.rust-lang.org/stable/std/option/enum.Option.html#method.unwrap_or_default
[`derive(new)`]: https://crates.io/crates/derive-new/
