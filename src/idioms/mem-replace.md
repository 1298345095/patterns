# 使用 `mem::{take(_), replace(_)}` 在切换枚举变体时保留拥有的值

## 描述

假设我们有一个 `&mut MyEnum`，而 `MyEnum`
至少包含两个变体：`A { name: String, x: u8 }` 和
`B { name: String }`。现在，我们希望在 `x` 为零时将 `MyEnum::A` 改为
`B`，同时保持原本就是 `MyEnum::B` 的值不变。

我们可以在不克隆 `name` 的情况下做到这一点。

## 示例

```rust
use std::mem;

enum MyEnum {
    A { name: String, x: u8 },
    B { name: String },
}

fn a_to_b(e: &mut MyEnum) {
    if let MyEnum::A { name, x: 0 } = e {
        // 取出 `name`，并用一个空 String 替换它
        // （注意，空字符串不需要分配内存）。
        // 然后构造新的枚举变体，
        // 并将它赋给 `*e`。
        *e = MyEnum::B {
            name: mem::take(name),
        }
    }
}
```

有更多变体时，这种方法同样适用：

```rust
use std::mem;

enum MultiVariateEnum {
    A { name: String },
    B { name: String },
    C,
    D,
}

fn swizzle(e: &mut MultiVariateEnum) {
    use MultiVariateEnum::*;
    *e = match e {
        // 所有权规则不允许按值取出 `name`；我们无法
        // 从可变引用中取出值，除非用另一个值替换它：
        A { name } => B {
            name: mem::take(name),
        },
        B { name } => A {
            name: mem::take(name),
        },
        C => D,
        D => C,
    }
}
```

## 动机

使用枚举时，我们可能希望就地修改枚举值，甚至切换到另一个变体。为了满足借用检查器的要求，这通常分为两个阶段。第一阶段观察现有值及其组成部分，决定接下来要做什么；第二阶段则根据条件修改这个值，如上例所示。

借用检查器不允许我们从枚举中直接取走
`name`，因为那里必须留有*某个值*。当然，我们可以对 `name` 调用
`.clone()`，把克隆出的值放进
`MyEnum::B`，但这就是[为满足借用检查器而克隆](../anti_patterns/borrow_clone.md)反模式的一个实例。其实，只通过可变借用来修改
`e`，就能避免额外的内存分配。

`mem::take` 可以将原来的值取出，用该类型的默认值替代它，并返回旧值。对于
`String`，默认值是一个无需分配内存的空
`String`。这样，我们就能以*拥有所有权的值*的形式取得原本的
`name`，再将它包装到另一个枚举变体中。

**注意：** `mem::replace`
与之非常相似，只是允许我们自行指定替换进去的值。示例中的 `mem::take` 调用等价于
`mem::replace(name, String::new())`。

不过，如果使用的是 `Option`，并且希望用 `None` 替换当前值，那么 `Option` 自带的
`take()` 方法会更加简洁，也更符合惯用法。

> **译注（学习提示）**：关键不变量是：只要通过 `&mut T`
> 访问原位置，该位置就必须保持为一个有效的 `T`。`take`／`replace`
> 把“交出旧值”和“留下有效替代值”合并起来，因此不需要用 `clone`
> 同时保留两份所有权。对本例的
> `String`，移动只转交字符串缓冲区的所有权，不复制其中的字符数据。参见
> [`mem::take`](https://doc.rust-lang.org/std/mem/fn.take.html) 和
> [`mem::replace`](https://doc.rust-lang.org/std/mem/fn.replace.html)。

## 优点

看吧，老妈，完全不用分配内存！而且这样做时，你可能还会觉得自己就是印第安纳·琼斯。

## 缺点

写起来稍显啰嗦。如果反复出错，你可能会开始讨厌借用检查器。编译器有时无法优化掉两次写入操作，因此，与在不保证内存安全的语言中采用的做法相比，性能可能有所下降。

此外，要取出的值的类型需要实现
[`Default` trait](./default.md)。如果所用类型没有实现这个 trait，也可以改用
`mem::replace`。

> **译注（学习提示）**：这种写法保证内存中的值始终有效，却不自动保证业务上的“要么全部成功，要么完全没变化”。如果成功
> `take` 之后、赋入最终结果之前新增了可能失败或 panic
> 的处理，旧对象可能暂时甚至最终保留默认占位值。推广这个模式时，应检查这个中间状态是否满足你的业务不变量；例子中的取出与构造之间没有额外处理，因此容易分析。

## 讨论

这种模式只在 Rust
中特别值得关注。在有垃圾回收的语言中，通常直接取得值的引用，由垃圾回收器追踪这些引用；在
C 等其他底层语言中，则可以先给指针建立别名，再回头处理后续问题。

不过，在 Rust
中，我们需要多做一点工作。一个拥有型值只能有一个所有者，因此，取出它时必须放回另一个值，就像印第安纳·琼斯用一袋沙子替换文物那样。

## 另请参阅

这种模式能在特定场景下消除[为满足借用检查器而克隆](../anti_patterns/borrow_clone.md)这一反模式。
