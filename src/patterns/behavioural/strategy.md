# 策略（Strategy，也称 Policy）

## 描述

[策略设计模式](https://en.wikipedia.org/wiki/Strategy_pattern)是一种实现关注点分离的技术，
也可以通过[依赖倒置](https://en.wikipedia.org/wiki/Dependency_inversion_principle)来解耦软件模块。

策略模式的基本思想是：对于解决某个特定问题的算法，只在抽象层面定义算法的骨架，
并将算法的具体实现拆分为不同的部分。

这样，使用算法的调用方就可以选择某个具体实现，而算法的整体流程保持不变。
换句话说，类的抽象规范不依赖派生类的具体实现，而具体实现必须遵守抽象规范。
这就是“依赖倒置”这个名称的由来。

## 动机

假设我们正在开发一个每月生成报表的项目。报表需要采用不同的格式，也就是不同的策略，
例如 `JSON` 或
`Plain Text`（纯文本）。但需求会随时间变化，我们无法预知将来会出现什么要求。
例如，可能需要以一种全新的格式生成报表，也可能只需要修改某种现有格式。

## 示例

在这个例子中，不变量（或者说抽象）是 `Formatter` 和 `Report`， 而 `Text` 与
`Json` 是具体的策略结构体。这些策略都必须实现 `Formatter` trait。

> **译注（学习提示）**：这里的“不变量”指设计中相对稳定的抽象和流程。`Report::generate<T: Formatter>`
> 把策略类型作为泛型参数，编译器可以针对具体类型进行单态化，使用静态分发。如果希望在运行时从异构集合中选择策略，也可以围绕
> `&dyn Formatter` 等 trait
> 对象来设计接口。策略模式规定的是行为可替换，采用哪一种分发方式则是另一个设计选择。

```rust
use std::collections::HashMap;

type Data = HashMap<String, u32>;

trait Formatter {
    fn format(&self, data: &Data, buf: &mut String);
}

struct Report;

impl Report {
    // 本应使用 Write，但这里保留 String，以略去错误处理
    fn generate<T: Formatter>(g: T, s: &mut String) {
        // 后端操作……
        let mut data = HashMap::new();
        data.insert("one".to_string(), 1);
        data.insert("two".to_string(), 2);
        // 生成报表
        g.format(&data, s);
    }
}

struct Text;
impl Formatter for Text {
    fn format(&self, data: &Data, buf: &mut String) {
        for (k, v) in data {
            let entry = format!("{k} {v}\n");
            buf.push_str(&entry);
        }
    }
}

struct Json;
impl Formatter for Json {
    fn format(&self, data: &Data, buf: &mut String) {
        buf.push('[');
        for (k, v) in data.into_iter() {
            let entry = format!(r#"{{"{}":"{}"}}"#, k, v);
            buf.push_str(&entry);
            buf.push(',');
        }
        if !data.is_empty() {
            buf.pop(); // 移除末尾多余的逗号
        }
        buf.push(']');
    }
}

fn main() {
    let mut s = String::from("");
    Report::generate(Text, &mut s);
    assert!(s.contains("one 1"));
    assert!(s.contains("two 2"));

    s.clear(); // 复用同一个缓冲区
    Report::generate(Json, &mut s);
    assert!(s.contains(r#"{"one":"1"}"#));
    assert!(s.contains(r#"{"two":"2"}"#));
}
```

> **译注（原文疑点）**：这里的 `Json`
> 只是针对示例数据手工拼接字符串，并没有对键中的引号、反斜杠等字符进行 JSON
> 转义；它还把数值写成了 JSON 字符串。因此不能直接把它当作通用 JSON
> 序列化器。另一方面，`HashMap` 的遍历顺序没有固定保证，所以测试使用 `contains`
> 检查各项，而没有比较整个输出的顺序；这也是选择容器时需要考虑的输出契约。

## 优点

主要优点是关注点分离。比如，本例中的 `Report` 无须了解 `Json` 和 `Text`
的具体实现； 输出格式的实现也不用关心数据如何预处理、存储和获取。
它们只需要知道应当实现哪个 trait，以及由哪个方法定义处理结果的具体算法实现，
也就是 `Formatter` 和 `format(...)`。

## 缺点

每种策略都必须实现至少一个模块，因此模块的数量会随着策略数量的增加而增加。
如果可选策略很多，使用者还必须了解各个策略之间的区别。

> **译注（原文疑点）**：“每种策略都必须实现至少一个模块”不是 Rust
> 的语言要求，也不是这个模式的必要条件；上面的例子就把两种策略写在同一个文件中。这里可以理解为：每增加一种策略，通常就要增加一份实现及其维护负担。是否拆成独立
> `mod` 或 crate，应由接口边界和复用需求决定。

## 讨论

上面的例子将所有策略都放在一个文件中。提供不同策略的方式包括：

- 全部写在同一个文件中，如本例所示，组织方式类似于分成模块。
- 拆分为不同模块，例如 `formatter::json` 模块和 `formatter::text` 模块。
- 使用编译特性开关，例如 `json` feature 和 `text` feature。
- 拆分为不同 crate，例如 `json` crate 和 `text` crate。

Serde crate 是应用策略模式的一个很好的例子。通过为自己的类型手动实现 `Serialize`
和 `Deserialize` trait， Serde
允许我们[完全定制](https://serde.rs/custom-serialization.html)序列化行为。
例如，由于 `serde_json` 与 `serde_cbor`
暴露了相似的方法，我们可以方便地将前者换成后者。 有了这一点，辅助 crate
`serde_transcode` 也变得更加实用和易用。

不过，在 Rust 中设计这个模式并不一定需要使用 trait。

下面这个小例子使用 Rust 的闭包来展示策略模式的思想：

```rust
struct Adder;
impl Adder {
    pub fn add<F>(x: u8, y: u8, f: F) -> u8
    where
        F: Fn(u8, u8) -> u8,
    {
        f(x, y)
    }
}

fn main() {
    let arith_adder = |x, y| x + y;
    let bool_adder = |x, y| {
        if x == 1 || y == 1 {
            1
        } else {
            0
        }
    };
    let custom_adder = |x, y| 2 * x + y;

    assert_eq!(9, Adder::add(4, 5, arith_adder));
    assert_eq!(0, Adder::add(0, 0, bool_adder));
    assert_eq!(5, Adder::add(1, 3, custom_adder));
}
```

实际上，Rust 已经在 `Options` 的 `map` 方法中使用了这个思想：

> **译注（原文疑点）**：这里的 `Options` 是原文中的名称，按示例应为
> `Option`。上文“不需要使用 trait”也应理解为“不需要自行定义一个策略
> trait”；闭包示例仍然通过 `F: Fn(u8, u8) -> u8` 这个 trait
> 约束来表达可调用行为。闭包让策略可以就地定义，并捕获所需环境，省去了为每个简单策略单独命名结构体的步骤。

```rust
fn main() {
    let val = Some("Rust");

    let len_strategy = |s: &str| s.len();
    assert_eq!(4, val.map(len_strategy).unwrap());

    let first_byte_strategy = |s: &str| s.bytes().next().unwrap();
    assert_eq!(82, val.map(first_byte_strategy).unwrap());
}
```

## 另请参阅

- [策略模式](https://en.wikipedia.org/wiki/Strategy_pattern)
- [依赖注入](https://en.wikipedia.org/wiki/Dependency_injection)
- [基于策略的设计](https://en.wikipedia.org/wiki/Modern_C++_Design#Policy-based_design)
- [使用策略模式，以 Rust 为航天应用实现 TCP 服务器](https://web.archive.org/web/20231003171500/https://robamu.github.io/posts/rust-strategy-pattern/)
