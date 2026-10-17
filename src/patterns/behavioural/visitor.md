# 访问者

## 描述

访问者（visitor）将作用于异构对象集合的算法封装起来。
它允许我们针对同一份数据编写多种不同的算法，而不必修改数据本身或其主要行为。

此外，访问者模式还能将对象集合的遍历过程，与针对各个对象执行的操作分离。

## 示例

```rust,ignore
// 将要访问的数据
mod ast {
    pub enum Stmt {
        Expr(Expr),
        Let(Name, Expr),
    }

    pub struct Name {
        value: String,
    }

    pub enum Expr {
        IntLit(i64),
        Add(Box<Expr>, Box<Expr>),
        Sub(Box<Expr>, Box<Expr>),
    }
}

// 抽象访问者
mod visit {
    use crate::ast::*;

    pub trait Visitor<T> {
        fn visit_name(&mut self, n: &Name) -> T;
        fn visit_stmt(&mut self, s: &Stmt) -> T;
        fn visit_expr(&mut self, e: &Expr) -> T;
    }
}

use ast::*;
use visit::*;

// 一个具体实现示例：遍历 AST，并将其作为代码解释执行。
struct Interpreter;
impl Visitor<i64> for Interpreter {
    fn visit_name(&mut self, _n: &Name) -> i64 {
        panic!()
    }
    fn visit_stmt(&mut self, s: &Stmt) -> i64 {
        match *s {
            Stmt::Expr(ref e) => self.visit_expr(e),
            Stmt::Let(..) => unimplemented!(),
        }
    }

    fn visit_expr(&mut self, e: &Expr) -> i64 {
        match *e {
            Expr::IntLit(n) => n,
            Expr::Add(ref lhs, ref rhs) => self.visit_expr(lhs) + self.visit_expr(rhs),
            Expr::Sub(ref lhs, ref rhs) => self.visit_expr(lhs) - self.visit_expr(rhs),
        }
    }
}
```

我们还可以实现其他访问者，例如类型检查器，而不必修改 AST 的数据结构。

> **译注（学习提示）**：这里的扩展方向是“为既有节点类型增加新操作”：新增解释器、检查器或统计器时，可以各自实现一个访问者。反过来，如果
> AST 新增节点种类，`Visitor`
> 的接口和各个实现就可能都需要调整。决定是否采用这个模式时，应先判断项目中更常变化的是数据种类，还是作用于这些数据的操作。

## 动机

只要想把算法应用于异构数据，访问者模式就可能派上用场。
如果数据是同构的，可以使用类似迭代器的模式。
与函数式写法相比，使用访问者对象可以让访问者持有状态，从而在不同节点之间传递信息。

> **译注（学习提示）**：`&mut self`
> 让访问者能够积累上下文，例如作用域栈、诊断列表或符号表，因此访问顺序可能成为算法语义的一部分。遍历函数可以负责递归，具体访问者负责进入、离开节点时的状态变化；重写某个访问方法时，要明确是否仍要遍历子节点。闭包也可以捕获状态，所以“能够保存状态”并非访问者对象独有的能力。

## 讨论

`visit_*` 方法经常不返回值，这与上面的示例不同。
在这种情况下，可以把遍历代码提取出来供多个算法共享，也可以提供默认不执行任何操作的方法。
在 Rust 中，常见的做法是为每一种数据提供相应的 `walk_*` 函数。例如：

```rust,ignore
pub fn walk_expr(visitor: &mut Visitor, e: &Expr) {
    match *e {
        Expr::IntLit(_) => {}
        Expr::Add(ref lhs, ref rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        Expr::Sub(ref lhs, ref rhs) => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
    }
}
```

> **译注（原文疑点）**：这里沿用了原文的示意签名 `&mut Visitor`，但前面声明的
> trait 是带类型参数的 `Visitor<T>`，而现代 Rust 的 trait 对象写法也需要
> `dyn`。若这里的访问方法返回 `()`，动态分发版本的参数可写为
> `&mut dyn Visitor<()>`；若要同时支持静态分发与 trait 对象，可以考虑
> `V: Visitor<()> + ?Sized` 并接收 `&mut V`。原代码保留，用于呈现遍历结构。

在其他语言中，例如 Java，通常会在数据类型上定义 `accept`
方法，由它承担同样的职责。

## 另请参阅

访问者模式在大多数面向对象语言中都很常见。

[维基百科条目](https://en.wikipedia.org/wiki/Visitor_pattern)

[折叠（fold）](../creational/fold.md)模式与访问者类似，但它会生成所访问数据结构的一个新版本。
