# 集合也是智能指针

## 描述

使用 [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html)
trait，将集合视为智能指针，为数据提供拥有型视图与借用视图。

## 示例

```rust,ignore
use std::ops::Deref;

struct Vec<T> {
    data: RawVec<T>,
    //..
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        //..
    }
}
```

`Vec<T>` 是拥有其中各个 `T` 值的集合，而切片（`&[T]`）则是对这些 `T`
值的借用集合。为 `Vec` 实现 `Deref` 后，就可以隐式地将 `&Vec<T>` 解引用为
`&[T]`，而且自动解引用查找也会纳入这一关系。许多你可能以为由 `Vec`
实现的方法，其实是为切片实现的。

`String` 和 `&str` 之间也有类似的关系。

## 动机

所有权和借用是 Rust
语言的核心概念。数据结构必须妥善处理这些语义，才能带来良好的使用体验。实现拥有自身数据的数据结构时，为这些数据提供借用视图，可以使
API 更加灵活。

## 优点

大多数方法只需要为借用视图实现，拥有型视图就能隐式地使用它们。

调用者可以自行选择借用数据，还是取得数据的所有权。

> **译注（学习提示）**：这里“视图”的边界也是能力的边界。`&[T]`
> 暴露连续元素，却不暴露 `Vec<T>`
> 的容量管理；因此切片上的算法可以复用于向量、数组的一部分以及其他连续存储。为自定义类型实现
> `Deref` 也会把目标类型的方法纳入隐式查找，这是一项公开 API
> 承诺，需要考虑以后双方出现同名方法时的影响。参见
> [`Deref` 的实现建议](https://doc.rust-lang.org/std/ops/trait.Deref.html#when-to-implement-deref-or-derefmut)。

## 缺点

只有通过解引用才能使用的方法和
trait，不会在约束检查中被考虑。因此，对采用这种模式的数据结构进行泛型编程时，情况可能变得复杂，参见
`Borrow`、`AsRef` 等 trait。

> **译注（学习提示）**：原文的 “bounds checking” 在这里指泛型的 trait
> 约束检查，不是索引越界检查。`Wrapper: Deref<Target = T>` 并不意味着 `Wrapper`
> 自动实现 `T` 所实现的其他 trait；“能够通过方法调用语法访问某方法”与“满足
> `U: SomeTrait`”是两件事。泛型接口可以要求适当的 `AsRef`／`Borrow`
> 实现，或由调用者显式取得目标引用。参见
> [强制转换的发生位置](https://doc.rust-lang.org/reference/type-coercions.html#coercion-sites)。

## 讨论

智能指针和集合有相似之处：智能指针指向一个对象，集合则指向多个对象。从类型系统的角度看，两者没有太大区别。如果只能通过集合访问其中的各个数据项，并且集合负责销毁这些数据，那么这个集合就拥有这些数据。即使存在共享所有权，某种形式的借用视图仍可能是合适的。如果集合拥有自己的数据，为这些数据提供借用视图通常很有用，这样就可以多次引用它们。

大多数智能指针，例如 `Foo<T>`，都会实现
`Deref<Target=T>`。不过，集合通常会解引用为某种自定义类型。`[T]` 和 `str`
获得了一些语言层面的支持，但一般情况下并不需要这样的支持。`Foo<T>` 可以实现
`Deref<Target=Bar<T>>`，其中 `Bar` 是一个动态大小类型，而 `&Bar<T>` 是 `Foo<T>`
内部数据的借用视图。

通常，有序集合会以 `Range` 类型为索引实现
`Index`，以提供切片语法；其目标将是借用视图。

## 另请参阅

- [解引用多态反模式](../anti_patterns/deref.md)。
- [`Deref` trait 文档](https://doc.rust-lang.org/std/ops/trait.Deref.html)。
