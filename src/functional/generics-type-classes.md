# 将泛型用作类型类

## 描述

Rust 的类型系统在设计上更接近 Haskell 等函数式语言，而非 Java 和 C++
等命令式语言。因此，Rust
能把许多编程问题转化为“静态类型”问题。这是选择函数式语言的一项重要收益，也是
Rust 许多编译期保证的关键。

这种思路的一个核心部分，是泛型类型的工作方式。例如，在 C++ 和 Java
中，泛型类型是供编译器使用的元编程构造。C++ 中的 `vector<int>` 和
`vector<char>`，只是把两个不同类型填入 `vector` 类型的同一份样板代码（称为
`template`）后得到的两份副本。

在 Rust
中，泛型类型参数会建立函数式语言所谓的“类型类约束”；最终使用者填入不同参数时，*类型本身确实会随之改变*。换句话说，`Vec<isize>`
和 `Vec<char>` *是两个不同类型*，类型系统的各个部分都会将它们区别对待。

这称为**单态化（monomorphization）**，即从**多态**代码中生成不同类型。这种特殊行为要求
`impl`
块指明泛型参数。泛型类型参数取不同值，就会产生不同类型，而不同类型可以拥有不同的
`impl` 块。

> **译注（原文疑点）**：这里把几件事混在了一起。C++ 的 `vector<int>` 与
> `vector<char>` 同样是不同类型；Java
> 的泛型虽有运行时类型擦除，编译期仍会区分参数化类型。Rust
> 中真正类似类型类约束的是 `P: ProtoKind` 这样的 trait
> bound，并非“出现一个泛型参数”本身。单态化描述的是根据具体类型生成代码的策略，不是不同泛型实例具有不同类型身份的原因。参见
> [Rust 对单态化的说明](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics)、[C++ 模板类型等价规则](https://eel.is/c++draft/temp.type)和
> [Java 类型擦除说明](https://docs.oracle.com/javase/tutorial/java/generics/erasure.html)。

在面向对象语言中，类可以从父类继承行为。而这里的方式不仅允许为类型类中的特定成员附加行为，还允许为它们提供额外的行为。

最接近的对应物，是 JavaScript 和 Python
中的运行时多态：任意构造函数都可以随意为对象添加新成员。不过，与这些语言不同，Rust
的泛型在编译期就已经确定，因此所有额外方法在使用时都能接受类型检查。这既提高了可用性，也保持了安全性。

## 示例

假设你正在为实验室的一组机器设计存储服务器。由于相关软件的要求，需要支持两种不同协议：BOOTP（用于
PXE 网络启动）和 NFS（用于远程挂载存储）。

你的目标是编写一个 Rust
程序，同时处理这两种协议。它会包含协议处理器，监听这两类请求。应用程序的主要逻辑则允许实验室管理员配置存储，以及实际文件的安全控制。

> **译注（原文疑点）**：这个场景对协议作了简化。BOOTP
> 负责发现地址、服务器和启动文件名，启动文件的数据传输通常由后续的 TFTP
> 等协议完成；因此下面的统一“文件下载请求”应理解为应用层抽象，不能直接当成 BOOTP
> 的完整工作流程。参见
> [RFC 951 第 2 节](https://datatracker.ietf.org/doc/html/rfc951#section-2)。

实验室机器发出的文件请求，无论使用哪种协议，都包含相同的基本信息：一种身份验证方式，以及待获取文件的名称。直观的实现可能是这样：

```rust,ignore
enum AuthInfo {
    Nfs(crate::nfs::AuthInfo),
    Bootp(crate::bootp::AuthInfo),
}

struct FileDownloadRequest {
    file_name: PathBuf,
    authentication: AuthInfo,
}
```

这个设计或许已经够用。但现在假设你需要添加*某个协议特有的*元数据。例如，对于
NFS，你想知道它的挂载点，以便实施额外的安全规则。

目前的结构体设计把协议的判定留到了运行时。这意味着，任何只适用于某一种协议的方法，都要求程序员在运行时进行检查。

获取 NFS 挂载点的代码可能如下：

```rust,ignore
struct FileDownloadRequest {
    file_name: PathBuf,
    authentication: AuthInfo,
    mount_point: Option<PathBuf>,
}

impl FileDownloadRequest {
    // ...其他方法...

    /// 如果这是 NFS 请求，就获取其挂载点。
    /// 否则返回 None。
    pub fn mount_point(&self) -> Option<&Path> {
        self.mount_point.as_ref()
    }
}
```

> **译注（原文疑点）**：此例中的 `as_ref()` 返回
> `Option<&PathBuf>`，不会自动把容器内部的值转换为
> `Option<&Path>`。如果按所写返回类型实现，可考虑
> `as_deref()`；这里保留原始代码，以便区分示例本身的类型不匹配和下文要讨论的 API
> 设计问题。

每个 `mount_point()` 调用者都必须检查
`None`，并编写处理它的代码。即使他们知道某条代码路径上永远只会使用 NFS
请求，也不能省略这一步！

如果把不同请求类型弄混时能直接产生编译期错误，就会理想得多。毕竟，使用者代码的整条路径，包括调用库中哪些函数，都知道某个请求是
NFS 请求还是 BOOTP 请求。

在 Rust 中，这确实可以做到！解决办法是*加入泛型类型参数*，从而拆分 API。

具体如下：

```rust
use std::path::{Path, PathBuf};

mod nfs {
    #[derive(Clone)]
    pub(crate) struct AuthInfo(String); // 省略 NFS 会话管理
}

mod bootp {
    pub(crate) struct AuthInfo(); // bootp 中没有身份验证
}

// 保持模块私有，防止外部使用者自行定义协议。
mod proto_trait {
    use super::{bootp, nfs};
    use std::path::{Path, PathBuf};

    pub(crate) trait ProtoKind {
        type AuthInfo;
        fn auth_info(&self) -> Self::AuthInfo;
    }

    pub struct Nfs {
        auth: nfs::AuthInfo,
        mount_point: PathBuf,
    }

    impl Nfs {
        pub(crate) fn mount_point(&self) -> &Path {
            &self.mount_point
        }
    }

    impl ProtoKind for Nfs {
        type AuthInfo = nfs::AuthInfo;
        fn auth_info(&self) -> Self::AuthInfo {
            self.auth.clone()
        }
    }

    pub struct Bootp(); // 没有额外的元数据

    impl ProtoKind for Bootp {
        type AuthInfo = bootp::AuthInfo;
        fn auth_info(&self) -> Self::AuthInfo {
            bootp::AuthInfo()
        }
    }
}

use proto_trait::ProtoKind; // 保持内部可见，防止外部实现
pub use proto_trait::{Bootp, Nfs}; // 重新导出，供调用者使用

struct FileDownloadRequest<P: ProtoKind> {
    file_name: PathBuf,
    protocol: P,
}

// 所有共用的 API 部分都放进泛型 impl 块
impl<P: ProtoKind> FileDownloadRequest<P> {
    fn file_path(&self) -> &Path {
        &self.file_name
    }

    fn auth_info(&self) -> P::AuthInfo {
        self.protocol.auth_info()
    }
}

// 所有协议专属的实现都放进各自的 impl 块
impl FileDownloadRequest<Nfs> {
    fn mount_point(&self) -> &Path {
        self.protocol.mount_point()
    }
}

fn main() {
    // 在这里编写你的代码
}
```

> **译注（学习提示）**：`P`
> 同时决定存储布局、认证信息的关联类型以及可用方法，所以它携带的是编译期可检查的能力信息。隐藏
> `ProtoKind`
> 则限制了谁能扩展这组协议。如果协议只能在收到网络数据后才能确定，运行时判断依然存在，但可以集中在入口处，再把明确类型的请求交给对应处理函数。泛型的收益在于把“已经确定的事实”传递下去，避免后续每一层重复检查。

采用这种方式后，假如使用者犯错，用了错误的类型：

```rust,ignore
fn main() {
    let mut socket = crate::bootp::listen()?;
    while let Some(request) = socket.next_request()? {
        match request.mount_point().as_ref() {
            "/secure" => socket.send("Access denied"),
            _ => {} // 继续处理……
        }
        // 其余代码
    }
}
```

他们就会遇到语法错误。`FileDownloadRequest<Bootp>` 没有实现
`mount_point()`，只有 `FileDownloadRequest<Nfs>` 才有。而后者当然是由 NFS
模块创建的，不是由 BOOTP 模块创建的！

> **译注（原文疑点）**：这里原文称“语法错误”，更准确地说是编译期的方法查找或类型错误：代码的语法形式可以成立，但该类型没有对应的方法。

## 优点

首先，它可以消除多个状态之间公共字段的重复定义。把非共享字段放进泛型参数所表示的类型后，公共部分就只需要实现一次。

其次，它按状态划分 `impl`
块，让这些代码更容易阅读。所有状态共用的方法集中编写在一个块中，而仅属于某个状态的方法则放在单独的块里。

这两点都能减少代码行数，并改善代码组织。

## 缺点

由于编译器目前实现单态化的方式，这会增加二进制文件的大小。希望未来的实现能够有所改进。

> **译注（学习提示）**：单态化可能为不同具体类型生成多份机器码，但“二进制一定变大”不是无条件结论，实际结果还受实例数量、内联和链接优化影响。这里值得权衡的是：独立实例通常带来更好的静态检查和优化机会，也可能增加编译时间与代码体积。若大量实例都调用同一段体积较大的逻辑，可以把不依赖
> `P` 的部分提取到普通函数中，让类型约束留在边界上。

## 替代方案

- 如果某个类型因为构造过程或部分初始化而需要“拆分
  API”，可以考虑使用[构建器模式](../patterns/creational/builder.md)。

- 如果不同类型之间的 API
  没有变化，变化的仅仅是行为，则更适合使用[策略模式](../patterns/behavioural/strategy.md)。

### 当前稳定版的泛型表达能力

> **译注（版本说明）**：原书对单态化、泛型类型身份与“类型类”
> 的讨论仍可学习，但不能据此推断 Rust 缺乏带生命周期参数的
> 关联类型。**GAT（泛型关联类型）已在 Rust 1.65 稳定**，
> **trait 中的返回位置 `impl Trait` 已在 Rust 1.75 稳定**；
> Rust 2024 还调整了返回位置 `impl Trait` 对生命周期参数的
> 默认捕获规则。它们可以改善借用返回值和 trait API 的表达，
> 但并不等同于支持任意高阶类型构造器，也不能自动让所有
> trait 都能做成 `dyn Trait`。

例如 GAT 可把“返回的视图借用 `self`”编码进关联类型：

```rust
trait ViewSource {
    type View<'a> where Self: 'a;
    fn view(&self) -> Self::View<'_>;
}

struct Bytes(Vec<u8>);

impl ViewSource for Bytes {
    type View<'a> = &'a [u8] where Self: 'a;
    fn view(&self) -> Self::View<'_> { &self.0 }
}

fn main() {
    let source = Bytes(vec![1, 2, 3]);
    assert_eq!(source.view(), &[1, 2, 3]);
}
```

> **译注（学习提示）**：这仍然是借用约束，不是运行时动态类型：
> 所返回的切片不能比 `source` 活得更久。设计 API 时要区分
> “用 GAT 表达借用关系”“用 trait object 做运行时分发”和
> “用单态化生成具体类型实例”三个独立维度。
> 参见 [GAT 稳定说明](https://blog.rust-lang.org/2022/10/28/gats-stabilization/)、
> [RPIT in trait 说明](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/)、
> [2024 `impl Trait` 捕获规则](https://doc.rust-lang.org/reference/types/impl-trait.html#capturing)。

## 另请参阅

这个模式在标准库中随处可见：

- `Vec<u8>` 可以由 String 转换而来，其他 `Vec<T>` 类型则不行。[^1]
- 迭代器可以转换成二叉堆，但其元素类型必须实现 `Ord` trait。[^2]
- `to_string` 方法专门为类型参数是 `str` 的 `Cow` 做了特化。[^3]

> **译注（原文疑点）**：第一项正文写的是 `String`，脚注链接却指向
> `From<CString> for Vec<u8>`，两者并不一致。`String` 和 `CString`
> 都有相应的字节转换方式，但它们的编码与终止符契约不同，不应把这两个类型混为一谈；原有正文与链接均予保留。

一些流行的 crate 也使用这个模式来提供灵活的 API：

- 面向嵌入式设备的 `embedded-hal`
  生态大量使用这个模式。例如，它可以静态验证控制嵌入式引脚的设备寄存器配置。当引脚被设为某种模式时，会返回一个
  `Pin<MODE>`
  结构体；泛型参数决定了该模式下可用的函数，这些函数并不直接属于不带具体模式的
  `Pin`。[^4]

- `hyper` HTTP 客户端库用它为不同的可插拔请求暴露丰富的
  API。使用不同连接器的客户端具有不同的方法和 trait
  实现，同时也有一组核心方法适用于任何连接器。[^5]

- “类型状态”（type state）模式让对象根据内部状态或不变量获得或失去某些 API。Rust
  使用同样的基本概念、略有不同的技术来实现这个模式。[^6]

[^1]: 参见：
    [impl From\<CString\> for Vec\<u8\>](https://doc.rust-lang.org/1.59.0/src/std/ffi/c_str.rs.html#803-811)

[^2]: 参见：
    [impl\<T: Ord\> FromIterator\<T\> for BinaryHeap\<T\>](https://web.archive.org/web/20201030132806/https://doc.rust-lang.org/stable/src/alloc/collections/binary_heap.rs.html#1330-1335)

[^3]: 参见：
    [impl\<'\_\> ToString for Cow\<'\_, str>](https://doc.rust-lang.org/stable/src/alloc/string.rs.html#2235-2240)

[^4]: 示例：
    [https://docs.rs/stm32f30x-hal/0.1.0/stm32f30x_hal/gpio/gpioa/struct.PA0.html](https://docs.rs/stm32f30x-hal/0.1.0/stm32f30x_hal/gpio/gpioa/struct.PA0.html)

[^5]: 参见：
    [https://docs.rs/hyper/0.14.5/hyper/client/struct.Client.html](https://docs.rs/hyper/0.14.5/hyper/client/struct.Client.html)

[^6]: 参见：
    [《类型状态模式的应用理由》](https://web.archive.org/web/20210325065112/https://www.novatec-gmbh.de/en/blog/the-case-for-the-typestate-pattern-the-typestate-pattern-itself/)
    和
    [Rust 类型状态系列（详尽的论述）](https://web.archive.org/web/20210328164854/https://rustype.github.io/notes/notes/rust-typestate-series/rust-typestate-index)
