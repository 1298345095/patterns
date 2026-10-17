# 使用 `#[non_exhaustive]` 和私有字段保留扩展空间

## 描述

在少数场景中，库作者可能希望为公开结构体添加公开字段，或者为枚举添加新变体，同时不破坏向后兼容性。

Rust 为这个问题提供了两种解决方案：

- 在结构体、枚举或枚举变体上使用
  `#[non_exhaustive]`。关于这个属性可以用于哪些位置，完整说明见[文档](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)。

- 为结构体添加一个私有字段，阻止使用者直接构造它或对它进行匹配，详见“替代方案”。

## 示例

```rust
mod a {
    // 公开结构体。
    #[non_exhaustive]
    pub struct S {
        pub foo: i32,
    }

    #[non_exhaustive]
    pub enum AdmitMoreVariants {
        VariantA,
        VariantB,
        #[non_exhaustive]
        VariantC {
            a: String,
        },
    }
}

fn print_matched_variants(s: a::S) {
    // 因为 S 标注了 `#[non_exhaustive]`，这里不能直接写出它的名称，
    // 而必须在模式中使用 `..`。
    let a::S { foo: _, .. } = s;

    let some_enum = a::AdmitMoreVariants::VariantA;
    match some_enum {
        a::AdmitMoreVariants::VariantA => println!("it's an A"),
        a::AdmitMoreVariants::VariantB => println!("it's a b"),

        // 这个变体也标注了 non_exhaustive，因此需要 ..
        a::AdmitMoreVariants::VariantC { a, .. } => println!("it's a c"),

        // 必须使用通配匹配，因为将来可能
        // 添加更多变体
        _ => println!("it's a new variant"),
    }
}
```

> **译注（原文疑点）**：这个示例把定义和使用放在同一个 crate
> 中，只有模块不同；`#[non_exhaustive]` 在定义它的 crate
> 内不施加上述限制，所以注释中“必须”添加 `..` 和 `_`
> 的说法在此示例中并不成立。把定义与调用方放到不同 crate
> 后，才能体现这些约束。此外，属性不禁止写出类型名称，限制的是外部代码的构造与穷尽匹配方式。参见
> [同 crate 与跨 crate 的区别](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)。

## 替代方案：为结构体添加私有字段

`#[non_exhaustive]` 只在跨 crate 的边界上生效。在同一个 crate
内，可以使用私有字段的方法。

为结构体添加字段，通常是向后兼容的修改。不过，如果调用者用模式解构结构体实例时写出了它的全部字段，添加新字段就会破坏这个模式。调用者也可以只写出部分字段，再在模式中使用
`..`；此时添加其他字段就是向后兼容的。让结构体至少有一个私有字段，就会迫使调用者采用后一种写法，从而为结构体将来的扩展留出空间。

这种做法的缺点是，你可能不得不添加一个本来不需要的字段。可以把字段类型设为
`()`，这样就不会带来运行时开销；再在字段名之前加上 `_`，避免未使用字段的警告。

```rust
pub struct S {
    pub a: i32,
    // 因为 `b` 是私有字段，匹配 `S` 时必须使用 `..`，并且 `S`
    // 不能被直接构造或直接匹配
    _b: (),
}
```

> **译注（原文疑点）**：这里私有字段实际名为 `_b`，注释中的 `b`
> 是原文笔误。“不能直接匹配”也过于宽泛：在无权访问私有字段的位置，仍可用
> `S { a, .. }` 匹配公开字段；不能做的是写出私有字段或省略必要的
> `..`。定义所在模块及其后代模块仍能访问私有字段，因此该方案的边界是模块可见性，并非整个
> crate 一律受限。参见
> [Rust 的私有项访问规则](https://doc.rust-lang.org/reference/visibility-and-privacy.html)。

## 讨论

对结构体使用
`#[non_exhaustive]`，可以在保持向后兼容的同时添加字段。即使所有字段都是公开的，它也会阻止调用者使用结构体构造语法。这可能很有帮助，但也应考虑：你是否*希望*通过编译错误，让调用者注意到新字段的出现，而不是让新增字段悄无声息地被忽略。

`#[non_exhaustive]`
也可以用于枚举变体。标注了这个属性的变体，与标注了同一属性的结构体具有相同的行为。

使用时应有明确目的，并保持谨慎：添加字段或变体时提升主版本号，往往是更好的选择。如果你建模的外部资源可能独立于库发生变化，`#[non_exhaustive]`
或许适用，但它并不是一个通用工具。

### 缺点

`#[non_exhaustive]`
可能明显降低代码的易用性，尤其是在使用者被迫处理未知枚举变体时。只有确实需要在**不提升**主版本号的情况下进行这类演进时，才应使用它。

把 `#[non_exhaustive]`
用于枚举，会迫使调用者处理通配分支。如果对这个分支没有合理的处理方式，就可能产生别扭的代码，以及极少执行到的路径。如果调用者选择在这里
`panic!()`，那么一开始就通过编译错误暴露问题，或许反而更好。实际上，`#[non_exhaustive]`
迫使调用者处理“其他情况”，而这种情况往往没有合理的处理方式。

> **译注（学习提示）**：这里需要权衡的是新增情况由谁承担处理责任。穷尽匹配把“重新审视所有分支”的责任放在升级时，由编译器提醒调用者；通配分支则要求调用者预先设计可持续的降级行为。若未知值可以被保存、记录、忽略或作为可恢复错误返回，预留分支就可能有意义；若每种新变体都必须改变业务决策，编译期的穷尽性检查往往更有价值。这是对上文
> API 演进取舍的进一步分析。

## 另请参阅

- [为枚举和结构体引入 #[non_exhaustive] 属性的 RFC](https://github.com/rust-lang/rfcs/blob/master/text/2008-non-exhaustive.md)
