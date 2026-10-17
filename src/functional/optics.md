# 函数式语言中的光学抽象

光学抽象（optics）是函数式语言中常见的一类 API 设计。它是一种纯函数式概念，在
Rust 中并不常用。

不过，探索这个概念有助于理解 Rust API
中的其他模式，例如[访问者](../patterns/behavioural/visitor.md)。Optics
本身也有一些小众的应用场景。

这是一个相当庞大的主题；要全面了解它的能力，需要借助专门讨论语言设计的书籍。不过，它在
Rust 中的适用范围要简单得多。

为了说明这个概念中与我们相关的部分，本章将使用 `Serde` API
作为例子，因为许多人很难仅凭 API 文档理解它。

在此过程中，我们会介绍几种称为 optics 的具体模式：*同构（Iso）*、*多态同构（Poly
Iso）*和*棱镜（Prism）*。

## 一个 API 示例：Serde

仅靠阅读 API 来理解 *Serde* 的工作方式颇有挑战，第一次接触时尤其如此。考虑下面的
`Deserializer` trait；任何解析新数据格式的库都会实现它：

```rust,ignore
pub trait Deserializer<'de>: Sized {
    type Error: Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    // 省略其余部分
}
```

下面是通过泛型参数传入的 `Visitor` trait 的定义：

```rust,ignore
pub trait Visitor<'de>: Sized {
    type Value;

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error;

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error;

    // 省略其余部分
}
```

这里发生了大量类型擦除，多层关联类型在接口之间来回传递。

> **译注（原文疑点）**：这里的“类型擦除”容易引起误解。上面的泛型参数与关联类型是在抽象具体实现，通常仍由编译器通过单态化进行静态分发；它们没有因为出现在
> trait 签名里就变成
> `dyn Trait`，也不必在运行时抹去具体类型。阅读本章后续同类表述时，应把“隐藏实现细节”与通常所说的运行时类型擦除区分开。参见
> [Rust 对单态化的说明](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics)。

但从整体上看，这些接口究竟在做什么？为什么不直接提供一个流式 API，让 `Visitor`
返回调用者需要的各部分数据，就此完成任务？为什么还需要这么多额外组件？

一种理解方式，是考察函数式语言中称为 *optics* 的概念。

它是一种组合行为与性质的方式，旨在方便地表达 Rust
中常见的模式，例如失败和类型转换等。[^1]

Rust
对这些概念的直接支持并不算好。不过，它们也体现在语言本身的设计中，理解这些概念有助于理解某些
Rust API。因此，本章尝试用 Rust 的实现方式来解释它们。

这或许能帮助我们看清，这些 API 所追求的是哪些具体的可组合性质。

## 基本光学抽象

### 同构（Iso）

Iso 是两个类型之间的值转换器。它非常简单，但在概念上是一个重要的基础构件。

例如，假设我们有一个自定义哈希表结构，用来保存文档的词语索引。[^2]
它以字符串作为键，表示单词；以一组索引作为值，例如文件中的偏移量。

其中一项关键能力，是将这种格式序列化到磁盘。一个“简单粗糙但能用”的方案，是实现它与
JSON 格式字符串之间的相互转换。这里暂时忽略错误，稍后再处理。

用函数式语言使用者熟悉的形式表示，就是：

```text
case class ConcordanceSerDe {
  serialize: Concordance -> String
  deserialize: String -> Concordance
}
```

因此，Iso 是一对在不同类型的值之间进行转换的函数：`serialize` 和 `deserialize`。

一个直观的实现如下：

```rust
use std::collections::HashMap;

struct Concordance {
    keys: HashMap<String, usize>,
    value_table: Vec<(usize, usize)>,
}

struct ConcordanceSerde {}

impl ConcordanceSerde {
    fn serialize(value: Concordance) -> String {
        todo!()
    }
    // 无效的词语索引按空索引处理
    fn deserialize(value: String) -> Concordance {
        todo!()
    }
}
```

> **译注（原文疑点）**：标准 optics 语境中的 Iso
> 不只是“两个方向都有函数”，还要求它们满足互逆规律：往返转换能恢复原值。任意
> JSON 字符串与 `Concordance`
> 通常不满足这一点，因为无效输入、空白格式和对象键顺序等信息可能丢失；代码注释中“无效输入得到空索引”的约定更会破坏可逆性。因此，这里应视为帮助理解
> API 分层的类比，而不是严格的 Iso 定义。参见
> [`lens` 的 Iso 定义与规律](https://hackage-content.haskell.org/package/lens-5.3.4/docs/Control-Lens-Iso.html)。

这看起来也许有些多此一举。在 Rust 中，这类行为通常用 trait
来实现；毕竟，标准库里已经有 `FromStr` 和 `ToString`。

这就引出了下一个主题：Poly Iso。

### 多态同构（Poly Iso）

前一个示例只是在两个固定类型的值之间进行转换。接下来的构件在此基础上引入了泛型，也更有意思。

Poly Iso 允许某项操作对任意类型保持泛型，同时返回一个固定类型。

这样就更接近解析过程了。先不考虑出错的情况，想一想一个基本解析器会做什么。同样，用通常的表示形式写出来：

```text
case class Serde[T] {
    deserialize(String) -> T
    serialize(T) -> String
}
```

这里出现了第一个泛型参数，也就是参与转换的类型 `T`。

在 Rust 中，这可以通过标准库中的一对 trait 来实现：`FromStr` 和 `ToString`。Rust
的版本甚至还会处理错误：

```rust,ignore
pub trait FromStr: Sized {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

pub trait ToString {
    fn to_string(&self) -> String;
}
```

与 Iso 不同，Poly Iso
可以应用于多个类型，并以泛型方式返回它们。这正是一个基本字符串解析器所需要的能力。

> **译注（原文疑点）**：本章用“Poly
> Iso”描述一组按类型参数化的双向转换，是简化的教学叫法。标准的多态 Iso 往往用
> `Iso s t a b`
> 表示，区分整体和焦点在转换前后的类型，并受相应规律约束；仅给一对转换函数加上泛型，并不足以证明它是合法的
> optics。`FromStr` 和 `ToString` 本身也不保证互逆。参见
> [`lens` 的 Iso 文档](https://hackage-content.haskell.org/package/lens-5.3.4/docs/Control-Lens-Iso.html)。

乍看之下，这似乎是编写解析器的好办法。我们来看一个使用示例：

```rust,ignore
use anyhow;

use std::str::FromStr;

struct TestStruct {
    a: usize,
    b: String,
}

impl FromStr for TestStruct {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<TestStruct, Self::Err> {
        todo!()
    }
}

impl ToString for TestStruct {
    fn to_string(&self) -> String {
        todo!()
    }
}

fn main() {
    let a = TestStruct {
        a: 5,
        b: "hello".to_string(),
    };
    println!("Our Test Struct as JSON: {}", a.to_string());
}
```

这看起来很合理。不过，它有两个问题。

首先，`to_string` 没有向 API 使用者表明“这是 JSON”。所有类型都得约定使用 JSON
表示，而 Rust
标准库中的许多类型早就不符合这个约定了。因此，使用它并不合适。定义我们自己的
trait，就能轻松解决这个问题。

但还有第二个更隐蔽的问题：规模扩大后怎么办？

只要每个类型都手动编写
`to_string`，这个方案就能运行。但如果每个想让自己的类型支持序列化的人，都得亲自编写大量代码，甚至使用不同的
JSON 库，事情很快就会变得混乱！

答案是 Serde
的两项关键创新之一：独立的数据模型。它用数据序列化语言中常见的结构来表示 Rust
数据。这样就能利用 Rust 的代码生成能力，创建一个称为 `Visitor` 的中间转换类型。

用通常的表示形式写出来就是下面这样；为简化说明，再次略去错误处理：

```text
case class Serde[T] {
    deserialize: Visitor[T] -> T
    serialize: T -> Visitor[T]
}

case class Visitor[T] {
    toJson: Visitor[T] -> String
    fromJson: String -> Visitor[T]
}
```

由此分别得到一个 Poly Iso 和一个 Iso。两者都可以用 trait 实现：

```rust
trait Serde {
    type V;
    fn deserialize(visitor: Self::V) -> Self;
    fn serialize(self) -> Self::V;
}

trait Visitor {
    fn to_json(self) -> String;
    fn from_json(json: String) -> Self;
}
```

> **译注（学习提示）**：这里的 `Serde`、`Visitor`
> 和稍后的宏是作者为了推导设计而构造的简化接口，不能直接当成真实 Serde
> API。真正的 Serde
> 数据模型主要是一套交互约定，不要求先构造保存整份数据的中间对象；`Visitor`
> 是参与构建目标值的访问者。统一模型让数据类型与格式分别实现自己的接口，把“每个类型都要适配每种格式”的组合问题拆开。参见
> [Serde 数据模型说明](https://serde.rs/data-model.html)。

由于 Rust
结构体到独立表示之间的转换遵循统一规则，甚至可以通过代码生成，创建与类型 `T`
关联的 `Visitor`：

```rust,ignore
#[derive(Default, Serde)] // “Serde” 派生宏创建 trait 的 impl 块
struct TestStruct {
    a: usize,
    b: String,
}

// 使用者编写这个宏，生成一个关联的访问者类型
generate_visitor!(TestStruct);
```

不过，我们还是实际试试这个方案。

```rust,ignore
fn main() {
    let a = TestStruct { a: 5, b: "hello".to_string() };
    let a_data = a.serialize().to_json();
    println!("Our Test Struct as JSON: {a_data}");
    let b = TestStruct::deserialize(
        generated_visitor_for!(TestStruct)::from_json(a_data));
}
```

结果发现，转换终究不是对称的！纸面上是对称的，但使用自动生成的代码后，从
`String` 一路转换回来所必需的具体类型，其名称被隐藏了。我们需要某种
`generated_visitor_for!` 宏来取得类型名。

> **译注（原文疑点）**：按照前面已经声明的关联类型，通常可以通过
> `<TestStruct as Serde>::V` 指定该类型，再用完全限定语法调用它的 `Visitor`
> 方法；并不一定需要额外宏来取得类型名。原文想说明的是生成代码的接口设计可能不方便，但“类型名隐藏”本身不足以证明必须增加宏。

这个方案有点别扭，但还能用……直到我们面对那个一直无法忽视的大问题。

当前只支持 JSON 一种格式。怎样才能支持更多格式？

按照目前的设计，需要完全重写代码生成部分，并创建一个新的 Serde
trait。这实在很糟糕，也毫无可扩展性！

要解决这个问题，需要更强大的工具。

## 棱镜（Prism）

要把格式因素也纳入考虑，我们需要类似下面这样的表示：

```text
case class Serde[T, F] {
    serialize: T, F -> String
    deserialize: String, F -> Result[T, Error]
}
```

这种构造称为 Prism。它在泛型层次上比 Poly
Iso“高出一级”；在这里，起关键作用的是那个“交叉”的类型 `F`。

遗憾的是，`Visitor` 是一个
trait，因为它的每一种具体实现都需要自己的定制代码，所以这会需要某种 Rust
不支持的泛型类型边界。

> **译注（原文疑点）**：标准 optics
> 中，棱镜的关键不是“多加一层泛型”，而是一个方向可以构造、另一个方向可能匹配失败，并且满足往返规律。例如，从
> `T` 构造 `Some(T)` 总能成功，从 `Option<T>` 提取 `T`
> 则可能失败；与聚焦结构体字段的透镜相比，它更接近聚焦枚举变体。额外的格式参数
> `F` 本身并不能证明接口是棱镜，也不能证明 Rust
> 无法表达它。原文没有具体说明所缺失的泛型边界，下面宜继续作为 API
> 分层的类比来读。参见
> [`lens` 的 Prism 定义与规律](https://hackage-content.haskell.org/package/lens-5.3.4/docs/Control-Lens-Prism.html)。

> **译注（Rust 1.99.1 版本补充）**：原文把这种设计困难概括为 Rust
> “不支持某种泛型类型边界”，这一限制没有在示例中形式化，
> 不宜理解成当前 stable 无法给格式参数建模。
> GAT（Rust 1.65）、trait 方法返回位置的 `impl Trait`（Rust 1.75）
> 和 2024 Edition 的精确捕获语法，已经使不少过去难以表达的
> 借用视图和泛型接口成为可能；不过它们也不自动提供高阶类型构造器、
> 类型反射或用于任意格式的正确 Serde 协议。
> 是否能把多种序列化格式统一到一个 API，仍取决于其访问者协议、
> 错误类型、借用边界与对象安全性等具体约束。
> 本章更适合作为“尝试设计 API，再检查抽象定律与实现约束”
> 的案例，而非当前 Rust 不支持 Prism 的证据。
> 参见 [关联类型与 GAT](https://doc.rust-lang.org/reference/items/associated-items.html#associated-types) 和
> [`impl Trait` 捕获规则](https://doc.rust-lang.org/reference/types/impl-trait.html#capturing)。

好在前面还有那个 `Visitor` 类型。`Visitor`
究竟在做什么？它试图让每种数据结构自行定义该如何解析自己。

那么，如果再为泛型格式添加一个接口，会怎么样？这样，`Visitor`
就只是一个实现细节，负责在两个 API 之间“搭桥”。

用通常的表示形式写出来：

```text
case class Serde[T] {
    serialize: F -> String
    deserialize F, String -> Result[T, Error]
}

case class VisitorForT {
    build: F, String -> Result[T, Error]
    decompose: F, T -> String
}

case class SerdeFormat[T, V] {
    toString: T, V -> String
    fromString: V, String -> Result[T, Error]
}
```

看，底层又出现了一对可以用 trait 实现的 Poly Iso！

于是就得到了 Serde API：

1. 每个待序列化的类型都实现 `Deserialize` 或 `Serialize`，对应于 `Serde` 类。
1. 它们会获得一个实现 `Visitor` trait
   的类型；准确地说是两个，每个方向一个。通常由派生宏生成这些代码，但也并非总是如此。它们包含在具体数据类型与
   Serde 数据模型之间进行构建或拆解的逻辑。
1. 实现 `Deserializer` trait 的类型处理所有与格式有关的细节，并由 `Visitor`
   来“驱动”。

> **译注（原文疑点）**：真实 Serde 并非在两个方向都使用
> `Visitor`。序列化时，`Serialize` 实现调用 `Serializer`
> 的方法；反序列化时，`Deserialize` 向 `Deserializer` 提供 `Visitor`，随后由
> `Deserializer`
> 驱动访问者回调来构造目标值，原文第三项的驱动方向写反了。对于映射、序列等复合数据，访问者还会通过相应访问接口请求后续内容，因此整体是协作过程。参见
> [Serde 数据模型](https://serde.rs/data-model.html)和[如何实现 `Deserialize`](https://serde.rs/impl-deserialize.html)。

这种拆分与 Rust 的类型擦除，实际上是在通过间接方式实现一个 Prism。

从 `Deserializer` trait 中可以看出来：

```rust,ignore
pub trait Deserializer<'de>: Sized {
    type Error: Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>;

    // 省略其余部分
}
```

还有访问者：

```rust,ignore
pub trait Visitor<'de>: Sized {
    type Value;

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error;

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error;

    // 省略其余部分
}
```

以及由宏实现的 `Deserialize` trait：

```rust,ignore
pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}
```

上面的讨论比较抽象，我们来看一个具体例子。

真实的 Serde 会怎样把一段 JSON 反序列化为前面定义的 `struct Concordance`？

1. 使用者调用库函数来反序列化数据，这会创建一个处理 JSON 格式的 `Deserializer`。
1. 根据结构体的字段创建一个
   `Visitor`，稍后会进一步说明。它知道如何创建通用数据模型中表示该结构所需的各种类型：`Vec`（列表）、`u64`
   和 `String`。
1. 反序列化器解析各个条目时，会调用 `Visitor`。
1. `Visitor`
   判断遇到的条目是否符合预期；如果不符合，就报告错误，表明反序列化失败。

对于上面那个很简单的结构，预期的访问模式如下：

1. 开始访问一个映射（*Serde* 数据模型中与 Rust 的 `HashMap` 或 JSON
   字典对应的结构）。
1. 访问一个名为 "keys" 的字符串键。
1. 开始访问一个映射值。
1. 对每个条目，先访问字符串键，再访问整数值。
1. 访问映射的末尾。
1. 将映射存入数据结构的 `keys` 字段。
1. 访问一个名为 "value_table" 的字符串键。
1. 开始访问一个列表值。
1. 对每个条目，访问一个整数。
1. 访问列表的末尾。
1. 将列表存入 `value_table` 字段。
1. 访问映射的末尾。

> **译注（原文疑点）**：前面声明的 `value_table` 实际是
> `Vec<(usize, usize)>`，所以列表中的每个条目是包含两个整数的元组，而不是单个整数；上述访问步骤遗漏了这一层。另外，常规派生出的结构体反序列化实现会依据字段名匹配，并不要求
> JSON
> 对象严格按这里列出的字段顺序出现。这是一种概念上的访问过程，不是必须逐条照做的调用脚本。参见
> [Serde 手动反序列化结构体的完整示例](https://serde.rs/deserialize-struct.html)。

但是，究竟是什么决定了预期的“观察”模式？

函数式编程语言可以利用柯里化，根据类型本身为每种类型创建反射。Rust
不支持这一点，因此每种类型都需要根据自己的字段及其性质编写专属代码。

> **译注（原文疑点）**：柯里化是把多参数函数改写成逐个接受参数的函数链，它本身不会产生类型反射。Rust
> 也可以用返回闭包的方式表达柯里化，例如
> `|x| move |y| x + y`；这里真正需要的是获取类型结构，并生成与字段对应的实现。Serde
> 的派生宏在编译期读取类型声明来完成这件事，不能把这种能力与柯里化混为一谈。参见
> [Rust Reference 对派生宏输入与输出的说明](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)。

*Serde* 使用派生宏解决这个易用性问题：

```rust,ignore
use serde::Deserialize;

#[derive(Deserialize)]
struct IdRecord {
    name: String,
    customer_id: String,
}
```

这个宏只是生成一个 impl 块，让结构体实现名为 `Deserialize` 的 trait。

其中的函数决定了如何构造结构体本身，代码根据结构体的字段生成。调用解析库时，在我们的例子中就是
JSON 解析库，它会创建一个 `Deserializer`，并把它作为参数传给
`Type::deserialize`。

`deserialize` 代码接着创建一个 `Visitor`，其调用会经过 `Deserializer`
的“折射”。如果一切顺利，`Visitor`
最终就会构造出一个与待解析类型对应的值，并将其返回。

完整示例请参阅 [*Serde* 文档](https://serde.rs/deserialize-struct.html)。

这样，待反序列化的类型只需要实现 API
的“上层”，文件格式只需要实现“下层”。泛型类型会在两者之间搭桥，所以每一部分都能与生态中的其他部分自然配合。

> **译注（学习提示）**：理解这套接口时，还值得单独追踪
> `'de`。它描述输出能够从输入借用数据的时间范围，而不是“反序列化函数运行多久”。`visit_str`
> 接收到的字符串可能只是临时缓冲区，不能据此保存为
> `&'de str`；`visit_borrowed_str` 才提供带 `'de` 的借用，`visit_string`
> 则交出拥有所有权的字符串。能否零拷贝，还取决于输入存储是否足够长寿，以及格式解析是否需要转义还原等处理。参见
> [Serde 反序列化器的生命周期](https://serde.rs/lifetimes.html)。

因此，如这个 API 设计所示，Rust
以泛型为基础的类型系统可以接近这些概念，并发挥它们的能力。不过，也可能需要过程宏来为泛型之间搭桥。

如果你有兴趣进一步了解这个主题，请继续阅读下面的资料。

## 另请参阅

- [lens-rs crate](https://crates.io/crates/lens-rs)：提供现成的透镜实现，接口比本章示例更简洁。
- [Serde](https://serde.rs)
  本身：它让最终使用者，也就是定义结构体的人，无需理解内部细节，就能直观地使用这些概念。
- [luminance](https://crates.io/crates/luminance)：一个用于绘制计算机图形的
  crate，采用类似的 API
  设计，包括通过过程宏，为不同像素类型的缓冲区创建完整且仍保持泛型的棱镜。
- [一篇介绍 Scala 透镜的文章](https://web.archive.org/web/20221128185849/https://medium.com/zyseme-technology/functional-references-lens-and-other-optics-in-scala-e5f7e2fdafe)：即使不熟悉
  Scala，也很容易阅读。
- [论文：Profunctor Optics——模块化数据访问器](https://web.archive.org/web/20220701102832/https://arxiv.org/ftp/arxiv/papers/1703/1703.10857.pdf)
- [Musli](https://github.com/udoprog/musli)：一个尝试采用类似结构、但选择不同实现方式的库，例如不再使用访问者。

[^1]: [School of Haskell：透镜入门小教程](https://web.archive.org/web/20221128190041/https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/a-little-lens-starter-tutorial)

[^2]: [维基百科：词语索引](https://en.wikipedia.org/wiki/Concordance_(publishing))
