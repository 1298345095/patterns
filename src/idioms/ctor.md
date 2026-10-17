# 构造函数

## 描述

Rust 没有将构造函数作为一种语言结构提供。通常的约定是通过名为 `new`
的[关联函数][associated function]创建对象：

````rust
/// 以秒为单位的时间。
///
/// # 示例
///
/// ```
/// let s = Second::new(42);
/// assert_eq!(42, s.value());
/// ```
pub struct Second {
    value: u64,
}

impl Second {
    // 构造一个新的 [`Second`] 实例。
    // 注意，这是一个关联函数，没有 self 参数。
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    /// 返回以秒为单位的值。
    pub fn value(&self) -> u64 {
        self.value
    }
}
````

> **译注（学习提示）**：`new` 的价值不仅在于缩短初始化语法。例子将 `value`
> 设为私有字段，把外部代码构造 `Second`
> 的入口集中到关联函数中；若以后需要检查范围，就能在这个入口维护类型的不变量。`new`
> 本身没有特殊的语言保证，能否绕过它取决于字段可见性以及其他构造入口的设计。参见
> [Rust 的可见性规则](https://doc.rust-lang.org/reference/visibility-and-privacy.html)。

## 默认构造函数

Rust 通过 [`Default`][std-default] trait 支持默认构造函数：

````rust
/// 以秒为单位的时间。
///
/// # 示例
///
/// ```
/// let s = Second::default();
/// assert_eq!(0, s.value());
/// ```
pub struct Second {
    value: u64,
}

impl Second {
    /// 返回以秒为单位的值。
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Default for Second {
    fn default() -> Self {
        Self { value: 0 }
    }
}
````

如果所有字段的类型都实现了 `Default`，也可以直接派生 `Default`，就像 `Second`
这样：

````rust
/// 以秒为单位的时间。
///
/// # 示例
///
/// ```
/// let s = Second::default();
/// assert_eq!(0, s.value());
/// ```
#[derive(Default)]
pub struct Second {
    value: u64,
}

impl Second {
    /// 返回以秒为单位的值。
    pub fn value(&self) -> u64 {
        self.value
    }
}
````

**注意：** 同时为类型实现 `Default` 和无参的 `new`
构造函数，是一种常见且符合使用者预期的做法。`new` 是 Rust
中约定俗成的构造函数名称，使用者会期待它的存在。因此，如果基础构造函数不接收参数是合理的，就应当提供这样的构造函数，即使它在功能上与
`default` 完全一致。

**提示：** 实现或派生 `Default` 后，你的类型便可以用于要求 `Default`
实现的场景，其中最典型的就是[标准库中的各种 `*or_default` 函数][std-or-default]。

> **译注（学习提示）**：只有在确实存在合理的无参默认值时，才适合提供这种双入口。若同时提供
> `new()` 和
> `Default::default()`，让一个调用另一个可以避免两套默认值逐渐偏离；若构造必须依赖调用者提供的信息，就不必为了形式统一硬造一个默认状态。`Default`
> 的契约是提供有用的默认值，并不等同于把内存清零。参见
> [`Default` 文档][std-default]。

## 另请参阅

- [默认值惯用法](default.md)更深入地介绍了 `Default` trait。

- [构建器模式](../patterns/creational/builder.md)适用于构造具有多种配置的对象。

- [API 指南中的 C-COMMON-TRAITS 条目][API Guidelines/C-COMMON-TRAITS]讨论了同时实现
  `Default` 和 `new` 的做法。

[associated function]: https://doc.rust-lang.org/stable/book/ch05-03-method-syntax.html#associated-functions
[std-default]: https://doc.rust-lang.org/stable/std/default/trait.Default.html
[std-or-default]: https://doc.rust-lang.org/stable/std/?search=or_default
[API Guidelines/C-COMMON-TRAITS]: https://rust-lang.github.io/api-guidelines/interoperability.html#types-eagerly-implement-common-traits-c-common-traits
