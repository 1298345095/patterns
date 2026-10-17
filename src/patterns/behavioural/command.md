# 命令

## 描述

命令（Command）模式的基本思想，是把操作封装为独立的对象，再将这些对象作为参数传递。

## 动机

假设我们把一系列操作或事务封装成对象，并希望稍后在其他时刻，按照某种顺序执行或调用它们。
这些操作或命令也可能由某个事件触发，例如用户按下按钮，或者一个数据包到达。
此外，这些命令还可能支持撤销，这对编辑器一类的操作很有帮助。
我们也可能希望记录已执行命令的日志，以便在系统崩溃后重新应用这些更改。

## 示例

定义两个数据库操作：`create table`（创建表）和 `add field`（添加字段）。
每个操作都是一条命令，并且知道如何撤销自身，例如通过 `drop table`（删除表）和
`remove field`（移除字段）。
当用户发起数据库迁移时，按既定顺序执行每条命令；当用户发起回滚时，则按相反顺序调用整组命令。

## 实现方式：使用 trait 对象

我们定义一个公共 trait，通过 `execute` 与 `rollback` 两个操作封装命令。
所有表示命令的结构体都必须实现这个 trait。

```rust
pub trait Migration {
    fn execute(&self) -> &str;
    fn rollback(&self) -> &str;
}

pub struct CreateTable;
impl Migration for CreateTable {
    fn execute(&self) -> &str {
        "create table"
    }
    fn rollback(&self) -> &str {
        "drop table"
    }
}

pub struct AddField;
impl Migration for AddField {
    fn execute(&self) -> &str {
        "add field"
    }
    fn rollback(&self) -> &str {
        "remove field"
    }
}

struct Schema {
    commands: Vec<Box<dyn Migration>>,
}

impl Schema {
    fn new() -> Self {
        Self { commands: vec![] }
    }

    fn add_migration(&mut self, cmd: Box<dyn Migration>) {
        self.commands.push(cmd);
    }

    fn execute(&self) -> Vec<&str> {
        self.commands.iter().map(|cmd| cmd.execute()).collect()
    }
    fn rollback(&self) -> Vec<&str> {
        self.commands
            .iter()
            .rev() // 反转迭代器的方向
            .map(|cmd| cmd.rollback())
            .collect()
    }
}

fn main() {
    let mut schema = Schema::new();

    let cmd = Box::new(CreateTable);
    schema.add_migration(cmd);
    let cmd = Box::new(AddField);
    schema.add_migration(cmd);

    assert_eq!(vec!["create table", "add field"], schema.execute());
    assert_eq!(vec!["remove field", "drop table"], schema.rollback());
}
```

## 实现方式：使用函数指针

还可以采用另一种方式：将每条命令写成一个单独的函数，保存函数指针，等到之后再调用这些函数。
由于函数指针实现了 `Fn`、`FnMut` 和 `FnOnce` 这三个
trait，我们也可以传递并保存闭包来代替函数指针。

> **译注（原文疑点）**：最后一句需要加上限制：能强制转换为本例 `fn() -> String`
> 的，是签名匹配、且不捕获环境的普通非异步闭包。捕获了变量的闭包即使实现
> `Fn`，也不能因此装进这个函数指针类型；这类命令可以使用下一节的 trait
> 对象，或者保留具体闭包类型的泛型设计。参见
> [Rust Reference 中的闭包强制转换规则](https://doc.rust-lang.org/reference/types/closure.html)。

```rust
type FnPtr = fn() -> String;
struct Command {
    execute: FnPtr,
    rollback: FnPtr,
}

struct Schema {
    commands: Vec<Command>,
}

impl Schema {
    fn new() -> Self {
        Self { commands: vec![] }
    }
    fn add_migration(&mut self, execute: FnPtr, rollback: FnPtr) {
        self.commands.push(Command { execute, rollback });
    }
    fn execute(&self) -> Vec<String> {
        self.commands.iter().map(|cmd| (cmd.execute)()).collect()
    }
    fn rollback(&self) -> Vec<String> {
        self.commands
            .iter()
            .rev()
            .map(|cmd| (cmd.rollback)())
            .collect()
    }
}

fn add_field() -> String {
    "add field".to_string()
}

fn remove_field() -> String {
    "remove field".to_string()
}

fn main() {
    let mut schema = Schema::new();
    schema.add_migration(|| "create table".to_string(), || "drop table".to_string());
    schema.add_migration(add_field, remove_field);
    assert_eq!(vec!["create table", "add field"], schema.execute());
    assert_eq!(vec!["remove field", "drop table"], schema.rollback());
}
```

## 实现方式：使用 `Fn` trait 对象

最后，我们也可以不定义公共的命令 trait，而是将每个实现了 `Fn` trait
的命令分别存入向量。

```rust
type Migration<'a> = Box<dyn Fn() -> &'a str>;

struct Schema<'a> {
    executes: Vec<Migration<'a>>,
    rollbacks: Vec<Migration<'a>>,
}

impl<'a> Schema<'a> {
    fn new() -> Self {
        Self {
            executes: vec![],
            rollbacks: vec![],
        }
    }
    fn add_migration<E, R>(&mut self, execute: E, rollback: R)
    where
        E: Fn() -> &'a str + 'static,
        R: Fn() -> &'a str + 'static,
    {
        self.executes.push(Box::new(execute));
        self.rollbacks.push(Box::new(rollback));
    }
    fn execute(&self) -> Vec<&str> {
        self.executes.iter().map(|cmd| cmd()).collect()
    }
    fn rollback(&self) -> Vec<&str> {
        self.rollbacks.iter().rev().map(|cmd| cmd()).collect()
    }
}

fn add_field() -> &'static str {
    "add field"
}

fn remove_field() -> &'static str {
    "remove field"
}

fn main() {
    let mut schema = Schema::new();
    schema.add_migration(|| "create table", || "drop table");
    schema.add_migration(add_field, remove_field);
    assert_eq!(vec!["create table", "add field"], schema.execute());
    assert_eq!(vec!["remove field", "drop table"], schema.rollback());
}
```

> **译注（学习提示）**：这里 `E: 'static`、`R: 'static`
> 约束的是闭包自身所持有的借用，不意味着返回的 `&'a str` 自动变成
> `&'static str`，也不要求闭包对象必须一直活到进程结束。另一个值得留意的不变量是：两个向量中的命令必须按索引一一配对；将执行与回滚操作放在同一个结构体中，通常更容易维护这种对应关系。真实数据库迁移还需记录哪些命令已成功执行，并处理执行与回滚本身可能失败的情况，本例返回字符串只是在演示调度顺序。

## 讨论

如果命令很小，可以定义成函数或通过闭包传入，那么使用函数指针可能更合适，因为它不使用动态分发。
如果命令是一个完整的结构体，包含许多函数和变量，并被组织为独立模块，那么 trait
对象就可能更适合。 [`actix`](https://actix.rs/)
中可以找到一个应用实例：它在为路由注册处理函数时使用了 trait 对象。 使用 `Fn`
trait 对象时，我们可以像使用函数指针那样创建和使用命令。

从性能角度看，性能与代码的简洁性、组织方式之间总是存在取舍。
静态分发能提供更快的性能，而动态分发则让应用结构更灵活。

> **译注（原文疑点）**：这里“函数指针不使用动态分发”容易造成误解：函数指针虽然不经由
> trait 对象的虚表，却仍可以在运行时指向不同函数，调用通常属于间接调用。trait
> 对象则通过虚表取出函数指针再调用，参见
> [trait 对象的分发机制](https://doc.rust-lang.org/reference/types/trait-object.html)。泛型具体类型常能帮助编译器单态化和内联，但“静态分发一定更快”不是普遍保证；实际还取决于优化、代码体积和调用所做的工作。上文
> `actix` 的说法保留为原文举例，不应据此推断具体版本的内部实现。

## 另请参阅

- [命令模式](https://en.wikipedia.org/wiki/Command_pattern)
- [命令模式的另一个示例](https://web.archive.org/web/20210223131236/https://chercher.tech/rust/command-design-pattern-rust)
