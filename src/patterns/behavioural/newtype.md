# 新类型（newtype）

有些情况下，我们希望一个类型的行为与另一个类型相似，或者希望在编译期强制满足某种行为约束，
而仅靠类型别名又不够用，该怎么办？

例如，出于安全考虑，我们可能希望为 `String` 自定义 `Display`
实现（密码就是一个例子）。

在这种情况下，可以用新类型（newtype）模式提供**类型安全**和**封装**。

## 描述

用一个只包含单个字段的元组结构体，为某个类型创建不透明的包装。
这会创建一个新类型，而不是该类型的别名（`type` 项）。

## 示例

```rust
use std::fmt::Display;

// 创建新类型 Password，以覆盖 String 的 Display trait 实现
struct Password(String);

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "****************")
    }
}

fn main() {
    let unsecured_password: String = "ThisIsMyPassword".to_string();
    let secured_password: Password = Password(unsecured_password.clone());
    println!("unsecured_password: {unsecured_password}");
    println!("secured_password: {secured_password}");
}
```

```shell
unsecured_password: ThisIsMyPassword
secured_password: ****************
```

> **译注（学习提示）**：这里实现的是 `Password` 自己的 `Display`，并没有修改
> `String`
> 的实现。包装类型为同一份底层表示赋予了新的接口与语义。这个示例只控制通过
> `Display`
> 进行的输出；原始字符串和复制出来的字符串仍然存在，是否开放取出明文的方法、是否实现
> `Debug`，也需要由这个类型的接口设计决定。

## 动机

新类型最主要的作用是建立抽象。它让不同类型能够共享实现细节，同时精确控制各自暴露的接口。
如果 API
使用新类型，而不是直接暴露底层实现类型，就可以在保持向后兼容的前提下更换内部实现。

新类型还可以用来区分计量单位。例如，将 `f64` 分别包装成 `Miles` 和
`Kilometres`，就可以区分英里与千米。

## 优点

被包装的类型与包装类型并不类型兼容（这一点与使用 `type` 不同），
因此新类型的使用者不会把两者“混淆”。

新类型是一种零成本抽象，不会带来运行时开销。

> **译注（学习提示）**：这里的“零成本”指包装本身不要求增加一次堆分配或动态分发，并不意味着任何自定义方法都没有成本，也不自动保证包装类型与内部类型具有相同的
> ABI。若接口需要依赖布局与 ABI 一致性，应了解并满足
> [`#[repr(transparent)]` 的条件](https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation)，不要仅凭“只有一个字段”推断这一保证。

隐私机制确保使用者无法访问被包装的类型，前提是字段为私有；字段默认就是私有的。

## 缺点

新类型的缺点，尤其是与类型别名相比，在于语言没有为它提供专门的支持。
因此，它可能需要*大量*样板代码：希望暴露底层类型的哪个方法，就要为它写一个转发方法；
希望包装类型实现哪个 trait，也要分别编写相应的实现。

> **译注（学习提示）**：转发哪些能力本身就是抽象设计的一部分。例如，用新类型维护“数值非负”这一不变量时，构造与修改接口都必须维护它；如果直接暴露内部值的
> `&mut` 引用，调用者就可能绕过检查。为省去转发而实现 `Deref`
> 之前，应先判断“像底层类型一样使用”是否符合这个新类型想表达的语义。

## 讨论

新类型在 Rust
代码中十分常见。建立抽象和表示计量单位是最常见的用途，但它还可以用于其他目的：

- 限制功能，例如减少暴露的函数或实现的 trait。
- 将具有复制语义的类型包装成具有移动语义的类型。
- 提供一个更具体的类型来建立抽象，从而隐藏内部类型。例如：

```rust,ignore
pub struct Foo(Bar<T1, T2>);
```

这里的 `Bar` 可能是某个公开的泛型类型，而 `T1` 和 `T2` 是内部类型。
模块的使用者不应该需要知道我们用 `Bar` 实现了 `Foo`； 不过，这里真正隐藏的是
`T1`、`T2` 这两个类型，以及它们与 `Bar` 的组合方式。

## 另请参阅

- [《Rust 程序设计语言》中的高级类型](https://doc.rust-lang.org/book/ch19-04-advanced-types.html?highlight=newtype#using-the-newtype-pattern-for-type-safety-and-abstraction)
- [Haskell 中的新类型](https://wiki.haskell.org/Newtype)
- [类型别名](https://doc.rust-lang.org/stable/book/ch19-04-advanced-types.html#creating-type-synonyms-with-type-aliases)
- [derive_more](https://crates.io/crates/derive_more)：为新类型派生多种内置
  trait 的 crate。
- [Rust 中的新类型模式](https://web.archive.org/web/20230519162111/https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html)
