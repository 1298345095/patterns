# 拆分结构体以分别借用

## 描述

大型结构体有时会给借用检查带来麻烦：虽然各个字段可以分别借用，但有些场景最终还是会借用整个结构体，
从而妨碍其他用途。一种解决办法，是将结构体拆分成若干较小的结构体，再把它们组合成原来的结构体。
这样，每个结构体都可以分别借用，使用方式也就更加灵活。

这种做法往往还能从其他方面改善设计：采用这个模式时，经常会发现一些更小、职责更明确的功能单元。

## 示例

下面是一个人为构造的例子，展示借用检查器如何阻止我们按原计划使用结构体：

```rust,ignore
struct Database {
    connection_string: String,
    timeout: u32,
    pool_size: u32,
}

fn print_database(database: &Database) {
    println!("Connection string: {}", database.connection_string);
    println!("Timeout: {}", database.timeout);
    println!("Pool size: {}", database.pool_size);
}

fn main() {
    let mut db = Database {
        connection_string: "initial string".to_string(),
        timeout: 30,
        pool_size: 100,
    };

    let connection_string = &mut db.connection_string;
    print_database(&db);
    *connection_string = "new string".to_string();
}
```

编译器会报告以下错误：

```ignore
let connection_string = &mut db.connection_string;
                        ------------------------- mutable borrow occurs here
print_database(&db);
               ^^^ immutable borrow occurs here
*connection_string = "new string".to_string();
------------------ mutable borrow later used here
```

我们可以应用这个设计模式，把 `Database`
重构成三个更小的结构体，从而解决借用检查问题：

```rust
// Database 现在由三个结构体组成：ConnectionString、Timeout 和 PoolSize。
// 先将它拆成较小的结构体
#[derive(Debug, Clone)]
struct ConnectionString(String);

#[derive(Debug, Clone, Copy)]
struct Timeout(u32);

#[derive(Debug, Clone, Copy)]
struct PoolSize(u32);

// 再把这些较小的结构体组合成 `Database`
struct Database {
    connection_string: ConnectionString,
    timeout: Timeout,
    pool_size: PoolSize,
}

// print_database 随后改为接收 ConnectionString、Timeout 和 Poolsize 结构体
fn print_database(connection_str: ConnectionString, timeout: Timeout, pool_size: PoolSize) {
    println!("Connection string: {connection_str:?}");
    println!("Timeout: {timeout:?}");
    println!("Pool size: {pool_size:?}");
}

fn main() {
    // 用这三个结构体初始化 Database
    let mut db = Database {
        connection_string: ConnectionString("localhost".to_string()),
        timeout: Timeout(30),
        pool_size: PoolSize(100),
    };

    let connection_string = &mut db.connection_string;
    print_database(connection_string.clone(), db.timeout, db.pool_size);
    *connection_string = ConnectionString("new string".to_string());
}
```

> **译注（原文疑点）**：本例能通过借用检查，不只是因为给字段加了包装结构体。更关键的变化是：`print_database`
> 不再接收 `&Database`，而是分别接收字段值；其中连接字符串通过 `clone()`
> 复制，另外两个字段通过 `Copy` 复制。原来的错误来自借用整个 `db`
> 与尚未结束的字段可变借用发生重叠。如果仍传入
> `&db`，仅仅拆分或包装字段并不能消除这个冲突；复制字符串也可能有额外成本。

## 动机

当一个结构体积累了很多字段，而我们希望分别借用这些字段时，这个模式特别有用。
它最终能让各部分的使用更加灵活。

## 优点

拆分结构体能够绕开借用检查器的某些限制，而且往往还能带来更好的设计。

## 缺点

它可能让代码更加冗长。有时，拆出来的小结构体并不是合适的抽象，反而会让设计变差。
这可能是一种“代码异味”，说明程序需要从其他方面进行重构。

## 讨论

没有借用检查器的语言不需要为了这个原因采用本模式，因此从这个意义上说，它是 Rust
特有的。
不过，把功能拆成更小的单元往往会让代码更清晰；这是软件工程中广泛认可的原则，与具体语言无关。

这个模式依赖 Rust 借用检查器识别并分别借用不同字段的能力。
在示例中，借用检查器知道 `a.b` 与 `a.c` 是不同部分，可以分别借用，
而不会尝试借用整个 `a`；否则这个模式就没有意义了。

> **译注（学习提示）**：可以把函数或方法的参数看成它“声明需要访问的数据范围”。`&mut self`
> 声明独占借用整个接收者，即使实现暂时只读取其中一两个字段，调用方也不能把这一内部细节当成接口保证。将彼此独立的职责放进不同子结构体，并让方法接收相应子结构体的引用，才能把独立性表达到类型接口中。拆分时应按职责划分，而不是机械地把每个字段都变成一个类型。
