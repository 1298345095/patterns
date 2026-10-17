# 简化文档示例中的初始化

## 描述

编写文档时，如果初始化某个结构体需要花费很多工夫，可以用一个接收该结构体参数的辅助函数包住示例，这样通常更省事。

## 动机

有些结构体的初始化参数很多或很复杂，同时又有多个方法，而这些方法都应该配有示例。

例如：

````rust,ignore
struct Connection {
    name: String,
    stream: TcpStream,
}

impl Connection {
    /// 通过连接发送请求。
    ///
    /// # 示例
    /// ```no_run
    /// # // 为了让示例能够工作，需要这些样板代码。
    /// # let stream = TcpStream::connect("127.0.0.1:34254");
    /// # let connection = Connection { name: "foo".to_owned(), stream };
    /// # let request = Request::new("RequestId", RequestType::Get, "payload");
    /// let response = connection.send_request(request);
    /// assert!(response.is_ok());
    /// ```
    fn send_request(&self, request: Request) -> Result<Status, SendErr> {
        // ...
    }

    /// 糟糕，这里又得重复全部样板代码！
    fn check_status(&self) -> Status {
        // ...
    }
}
````

> **译注（原文疑点）**：这里是省略了上下文的示意代码，外层代码块也标注了
> `ignore`。其中 `TcpStream::connect` 实际返回
> `io::Result<TcpStream>`，不能直接作为 `TcpStream`
> 字段传入；把它用于真正的文档测试时，需要先处理连接错误。译文保留原始代码，避免把这种修正混入翻译。参见
> [`TcpStream::connect`](https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.connect)。

## 示例

与其写下全部样板代码来创建 `Connection` 和
`Request`，不如创建一个接收它们作为参数的辅助函数，把示例包在里面：

````rust,ignore
struct Connection {
    name: String,
    stream: TcpStream,
}

impl Connection {
    /// 通过连接发送请求。
    ///
    /// # 示例
    /// ```
    /// # fn call_send(connection: Connection, request: Request) {
    /// let response = connection.send_request(request);
    /// assert!(response.is_ok());
    /// # }
    /// ```
    fn send_request(&self, request: Request) -> Result<Status, SendErr> {
        // ...
    }
}
````

**注意：** 上例中的 `assert!(response.is_ok());`
在测试时并不会实际执行，因为它位于一个从未被调用的函数中。

## 优点

写法更加简洁，也避免了在各个示例中重复代码。

## 缺点

由于示例被放在函数中，代码不会被执行测试。不过，运行 `cargo test`
时仍会检查它能否编译。因此，这种模式最适合本来就需要 `no_run`
的场景；使用它以后，就不必再添加 `no_run`。

> **译注（学习提示）**：这里保留下来的是编译期验证，包括名称解析、类型、借用和
> trait 约束；没有验证的是请求是否成功、断言是否成立等运行期行为。以 `#` 开头的
> rustdoc 示例行只是展示时隐藏，仍会参与编译。还要区分本书外层的 `ignore`
> 示例与实际放入库中的文档测试：`ignore`
> 本身会跳过测试，不能据此宣称本书这些不完整片段已经过编译验证。参见
> [rustdoc 文档测试](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)。

## 讨论

如果不需要断言，这种模式就很适合。

如果需要断言，另一种方案是添加一个用于创建辅助实例的公开方法，并标注
`#[doc(hidden)]`，让使用者看不到它。这个方法是 crate 公开 API
的一部分，因此可以在 rustdoc 示例中调用。

> **译注（学习提示）**：`#[doc(hidden)]`
> 隐藏的是生成文档中的展示，不会把公开方法变成私有方法。如果为测试专门提供这样的入口，外部代码仍可能调用它；需要考虑它是否允许绕过正常构造过程中的不变量，以及以后是否愿意继续维护它。参见
> [`#[doc(hidden)]` 的行为](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html#hidden)。
