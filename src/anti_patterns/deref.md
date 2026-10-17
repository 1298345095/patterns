# `Deref` 多态

## 描述

滥用 `Deref` trait，在结构体之间模拟继承，从而复用方法。

## 示例

有时，我们想模拟 Java 等面向对象语言中的常见写法：

```java
class Foo {
    void m() { ... }
}

class Bar extends Foo {}

public static void main(String[] args) {
    Bar b = new Bar();
    b.m();
}
```

可以用 `Deref` 多态这个反模式来实现：

```rust
use std::ops::Deref;

struct Foo {}

impl Foo {
    fn m(&self) {
        //..
    }
}

struct Bar {
    f: Foo,
}

impl Deref for Bar {
    type Target = Foo;
    fn deref(&self) -> &Foo {
        &self.f
    }
}

fn main() {
    let b = Bar { f: Foo {} };
    b.m();
}
```

Rust 没有结构体继承。我们改用组合，在 `Bar` 内包含一个 `Foo` 实例。
由于这个字段是一个值，它会内联存储；因此，如果这些结构体包含字段，
它们的内存布局就会与 Java 版本相同——大概如此，如果想确定这一点，应使用
`#[repr(C)]`。

> **译注（原文疑点）**：上段的内存布局类比不成立。字段按值内嵌这一点是正确的，
> 但默认的 Rust 表示并不保证字段按声明顺序排列，`#[repr(C)]` 规定的也是 面向 C
> 互操作的数据布局，并不保证与 Java 对象布局相同。不要根据这段类比推导
> 跨语言内存兼容性。参见
> [Rust Reference 的类型布局说明](https://doc.rust-lang.org/reference/type-layout.html)。

为了让方法调用成立，我们为 `Bar` 实现 `Deref`，并将 `Foo` 设为目标类型， 在
`deref` 中返回对内嵌 `Foo` 字段的引用。这意味着对 `Bar` 解引用时， 例如使用
`*`，就会得到一个 `Foo`。这很奇怪：解引用通常是从对 `T` 的引用得到 `T`，
而这里却涉及两个没有这种指针关系的类型。不过，点运算符会执行隐式解引用，
因此方法查找除了会检查 `Bar`，也会检查 `Foo`。

## 优点

可以少写一点样板代码，例如：

```rust,ignore
impl Bar {
    fn m(&self) {
        self.f.m()
    }
}
```

## 缺点

最重要的是，这种写法令人意外：后续阅读代码的程序员不会预期它有这样的行为。
这是因为我们没有按照 `Deref` 的设计意图和文档约定来使用它，而是在滥用它；
同时，这套机制又是完全隐式的。

这个模式不会像 Java 或 C++ 的继承那样，在 `Foo` 与 `Bar` 之间建立子类型关系。
而且，`Foo` 实现的 trait 不会自动由 `Bar` 实现，因此这种模式在类型约束检查，
进而在泛型编程中，都会带来问题。

> **译注（学习提示）**：要区分“方法调用能找到 `Foo::m`”与“`Bar` 满足 `Foo`
> 所满足的 trait 约束”。前者可以依靠方法查找中的自动解引用完成；后者要求 `Bar`
> 自己实现相应 trait。因而某段具体类型代码中可用的 `b.m()`， 不代表把 `b`
> 传给一个有相应泛型约束的函数也会成立。 `Deref` 的隐式行为会成为公开 API
> 的一部分，参见
> [标准库关于何时实现 `Deref` 的建议](https://doc.rust-lang.org/std/ops/trait.Deref.html#when-to-implement-deref-or-derefmut)。

这种模式对 `self` 的处理，与多数面向对象语言也有细微的语义差别。
在那些语言中，`self` 通常仍然是对子类的引用；而在这种模式下，
它指向的是定义该方法的那个“类”。

最后，这种模式只支持单继承，也没有接口、基于类的私有访问控制，
以及其他与继承有关的特性。因此，习惯 Java 等语言继承机制的程序员，
使用它时会遇到一些不易察觉却令人意外的差别。

## 讨论

不存在一种适用于所有情况的理想替代方案。根据具体场景， 可能更适合用 trait
重新实现，也可能更适合手工编写门面方法，将调用转发给 `Foo`。 我们确实打算为 Rust
加入一种与此类似的继承机制，但它进入稳定版 Rust 很可能还需要
相当长的时间。详情可阅读这两篇[博客](http://aturon.github.io/blog/2015/09/18/reuse/)
[文章](http://smallcultfollowing.com/babysteps/blog/2015/10/08/virtual-structs-part-4-extended-enums-and-thin-traits/)，
以及这个 [RFC issue](https://github.com/rust-lang/rfcs/issues/349)。

> **译注（版本说明）**：关于“打算加入继承机制”的这句话来自原文， 所引博客发表于
> 2015 年，应作为当时的探索背景阅读，而不是当前语言路线图的承诺。 Rust
> 对代码复用和多态提供的机制，可参见
> [《Rust 程序设计语言》的相关章节](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html)。

`Deref` trait 是为实现自定义指针类型而设计的。它的意图是从“指向 `T` 的指针” 得到
`T`，而不是在不同类型之间做转换。遗憾的是，这种意图没有被 trait 定义强制保证，
而且很可能也无法强制保证。

Rust 试图在显式机制与隐式机制之间取得谨慎的平衡，倾向于让类型转换显式发生。
点运算符的自动解引用是一个例外：这里的使用便利性强烈支持采用隐式机制。
但其设计意图是让隐式处理限于指针的间接层级，而不是任意类型之间的转换。

### Rust 1.86+ 的 trait object 向上转换不是结构体继承

> **译注（版本说明）**：原文关于将来可能加入继承机制的展望
> 应放回 2015 年历史背景。Rust 1.86 稳定了 **trait object upcasting**：
> 已知 `Sub: Super` 时，`&dyn Sub` 可以强制转换为 `&dyn Super`，
> `Arc` 等智能指针上的类似转换也受支持。
> 这解决的是 trait 对象在父 trait 接口上的动态分发适配，
> **不提供**“结构体继承字段／实现”或经 `Deref` 模拟子类型的能力。

```rust
trait Base { fn id(&self) -> u32; }
trait Extended: Base {}
struct Record;
impl Base for Record { fn id(&self) -> u32 { 42 } }
impl Extended for Record {}

fn upcast(value: &dyn Extended) -> &dyn Base { value }

fn main() {
    let record = Record;
    assert_eq!(upcast(&record).id(), 42);
}
```

> **译注（学习提示）**：由于两种 `dyn Trait` 对象的 vtable
> 可能不同，上转型不应理解为“只是复制一个数据指针”。
> trait 约束、对象安全性（dyn compatibility）、字段复用和
> 继承语义仍属于不同的设计问题。参见
> [Rust 1.86 发布说明](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/#trait-upcasting)。

## 另请参阅

- [集合也是智能指针](../idioms/deref.md)。
- 可以减少样板代码的委托 crate，例如
  [delegate](https://crates.io/crates/delegate) 和
  [ambassador](https://crates.io/crates/ambassador)。
- [`Deref` trait 文档](https://doc.rust-lang.org/std/ops/trait.Deref.html)。
