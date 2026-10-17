# 构建器

## 描述

通过调用辅助的构建器（builder）来构造对象。

## 示例

```rust
#[derive(Debug, PartialEq)]
pub struct Foo {
    // 许多复杂的字段。
    bar: String,
}

impl Foo {
    // 这个方法让使用者更容易发现构建器
    pub fn builder() -> FooBuilder {
        FooBuilder::default()
    }
}

#[derive(Default)]
pub struct FooBuilder {
    // 可能有许多可选字段。
    bar: String,
}

impl FooBuilder {
    pub fn new(/* ... */) -> FooBuilder {
        // 设置 Foo 最少需要的字段。
        FooBuilder {
            bar: String::from("X"),
        }
    }

    pub fn name(mut self, bar: String) -> FooBuilder {
        // 在构建器自身上设置名称，再按值返回构建器。
        self.bar = bar;
        self
    }

    // 如果这里能够不消费构建器，就会带来一个好处：
    // 我们可以把 FooBuilder 作为模板，
    // 用来构造多个 Foo。
    pub fn build(self) -> Foo {
        // 从 FooBuilder 创建 Foo，将 FooBuilder 中的所有设置
        // 应用到 Foo 上。
        Foo { bar: self.bar }
    }
}

#[test]
fn builder_test() {
    let foo = Foo {
        bar: String::from("Y"),
    };
    let foo_from_builder: Foo = FooBuilder::new().name(String::from("Y")).build();
    assert_eq!(foo, foo_from_builder);
}
```

> **译注（原文疑点）**：示例中的两个入口并不等价：`Foo::builder()` 调用派生的
> `Default`，使 `bar` 初始为空字符串；`FooBuilder::new()` 则把它设为
> `"X"`。此外，`build` 上方的注释说的是“不消费构建器时”的好处，但这里实际采用
> `build(self)`，会消费构建器。原代码保留；在自己的 API
> 中，应明确各入口的默认值，以及构建器是否支持重复使用。

## 动机

当其他做法需要很多构造函数，或者构造过程带有副作用时，这个模式很有用。

## 优点

将用于构建对象的方法与其他方法分开。

避免构造函数数量不断膨胀。

既适合用一行代码完成初始化，也适合更复杂的构造过程。

为目标结构体添加新字段时，可以通过更新构建器，使调用方代码保持向后兼容。

> **译注（学习提示）**：构建器还可以充当不变量的入口：先允许配置处于未完成状态，再在
> `build` 时统一检查字段之间的约束，并在必要时返回
> `Result<T, E>`。如果某些字段必须在编译期保证已设置，可以进一步用类型状态区分构建阶段；代价是泛型和类型数量增加。新增字段要保持兼容，也需要为已有调用方式提供合理的默认行为。

## 缺点

比直接创建结构体值或使用简单的构造函数更复杂。

## 讨论

相较于许多其他语言，Rust 更常使用这个模式，甚至会把它用于更简单的对象， 因为
Rust 不支持函数重载，也不支持为函数参数指定默认值。
同一个类型上，一个名称只能对应一个方法，因此在 Rust 中提供多个构造函数， 不像在
C++、Java 等语言中那么方便。

当构建器对象本身也有独立用途，而不只是临时的构建辅助对象时，这个模式也很常见。
例如，[`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html)
就是
[`Child`](https://doc.rust-lang.org/std/process/struct.Child.html)（子进程）的构建器。
在这些情况下，通常不会采用 `T` 与 `TBuilder` 这一对命名。

上面的示例按值接收并返回构建器。通过可变引用接收和返回构建器，往往更易用，也更高效。
借用检查器让这种写法能够自然地工作。它的一个优点是，既可以写出这样的代码：

```rust,ignore
let mut fb = FooBuilder::new();
fb.a();
fb.b();
let f = fb.build();
```

也可以使用 `FooBuilder::new().a().b().build()` 这样的风格。

> **译注（学习提示）**：设置方法的接收者与 `build`
> 的接收者要一起设计。若设置方法返回 `&mut Self`，而 `build(self)`
> 需要取得所有权，链式调用末尾就不能直接从这个可变引用中移走构建器。`build(&self)`
> 通常需要复制或共享内部配置；`build(&mut self)`
> 可以取走部分字段，但要定义构建之后的状态；本例的 `build(self)` 则能直接移动
> `String`，避免为构建而克隆它。哪一种更高效，取决于配置是否需要复用以及字段的所有权结构。

## 另请参阅

- [风格指南中的说明](https://web.archive.org/web/20210104103100/https://doc.rust-lang.org/1.12.0/style/ownership/builders.html)
- [derive_builder](https://crates.io/crates/derive_builder)：自动实现这个模式、避免样板代码的
  crate。
- 构造过程更简单时，可使用[构造函数模式](../../idioms/ctor.md)。
- [构建器模式（维基百科）](https://en.wikipedia.org/wiki/Builder_pattern)
- [复杂值的构造](https://web.archive.org/web/20210104103000/https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder)
