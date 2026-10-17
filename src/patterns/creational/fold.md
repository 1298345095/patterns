# 折叠（fold）

## 描述

对数据集合中的每个元素运行一个算法，生成一个新元素，从而构成一个全新的集合。

我不太清楚这里名称的来历。Rust 编译器使用了“fold”和“folder”这两个术语，
不过在我看来，它似乎更像通常意义上的映射（map），而不是折叠（fold）。
更多细节请参阅下面的讨论。

## 示例

```rust,ignore
// 将要折叠的数据：一棵简单的 AST。
mod ast {
    pub enum Stmt {
        Expr(Box<Expr>),
        Let(Box<Name>, Box<Expr>),
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

// 抽象的 folder
mod fold {
    use ast::*;

    pub trait Folder {
        // 对叶节点，直接返回节点本身。有些情况下，
        // 对内部节点也可以这样做。
        fn fold_name(&mut self, n: Box<Name>) -> Box<Name> { n }
        // 折叠子节点，创建新的内部节点。
        fn fold_stmt(&mut self, s: Box<Stmt>) -> Box<Stmt> {
            match *s {
                Stmt::Expr(e) => Box::new(Stmt::Expr(self.fold_expr(e))),
                Stmt::Let(n, e) => Box::new(Stmt::Let(self.fold_name(n), self.fold_expr(e))),
            }
        }
        fn fold_expr(&mut self, e: Box<Expr>) -> Box<Expr> { ... }
    }
}

use fold::*;
use ast::*;

// 一个具体实现示例：将每个名称都重命名为 'foo'。
struct Renamer;
impl Folder for Renamer {
    fn fold_name(&mut self, n: Box<Name>) -> Box<Name> {
        Box::new(Name { value: "foo".to_owned() })
    }
    // 其他节点使用默认方法。
}
```

> **译注（原文疑点）**：这段代码在原文中标为
> `ignore`，属于未补全的示意：`fold_expr` 的函数体仍是 `...`，而且 `Name::value`
> 是 `ast` 模块的私有字段，模块外的 `Renamer` 按当前写法不能直接构造它。在现代
> edition 中还需检查 `use ast::*`
> 等模块路径。这里保留代码原貌，重点展示“取得节点所有权、递归转换子节点、返回替代节点”的接口形状。

在一棵 AST 上运行 `Renamer`，结果会是一棵新的 AST：除了每个名称都改成了 `foo`
之外， 它与原来的 AST 完全相同。在实际使用中，folder
还可能在自身结构体中保存跨节点的状态。

也可以把 folder 定义成将一种数据结构映射为另一种不同、但通常相近的数据结构。
例如，我们可以把 AST 折叠成一棵 HIR 树；HIR 是高层中间表示（high-level
intermediate representation）的缩写。

## 动机

我们经常希望对数据结构中的每个节点执行某种操作，从而得到映射后的数据结构。
对于简单数据结构上的简单操作，可以使用 `Iterator::map`。
如果操作更加复杂，例如前面节点的处理可能影响后面节点，或者数据结构本身不容易直接迭代，
使用折叠模式就更加合适。

与访问者模式一样，折叠模式让我们可以将数据结构的遍历过程，与作用于每个节点的操作分离。

## 讨论

以这种方式映射数据结构，在函数式语言中很常见。在面向对象语言中，通常更倾向于原地修改数据结构。
这种“函数式”做法在 Rust 中也很常见，主要是因为 Rust 倾向于使用不可变数据。
在多数情况下，使用新的数据结构而不是修改旧的数据结构，会让代码更容易推理。

通过调整 `fold_*` 方法接收节点的方式，可以改变效率与可复用性之间的取舍。

在上面的示例中，我们操作的是 `Box`
指针。由于这些指针独占其数据的所有权，原来的数据结构就不能再次使用。
另一方面，如果一个节点没有变化，直接复用它的效率很高。

> **译注（学习提示）**：“返回新的结构”不一定意味着每个节点都重新分配。`fold_name`
> 的默认实现直接返回传入的
> `Box`，转移的是所有权，底层分配可以原样保留；真正被替换的节点才需要另外构造。这个接口用“调用者放弃旧树的所有权”换取重用旧节点存储的机会，因此特别适合编译器中逐阶段消费中间表示的转换。

如果改为操作借用的引用，原来的数据结构就可以继续使用；不过，即使一个节点没有变化，也必须克隆它，
这可能很昂贵。

使用引用计数指针则可以兼顾两者：既能复用原来的数据结构，也不必克隆未发生变化的节点。
不过，这类指针使用起来不那么方便，而且意味着数据结构不能是可变的。

> **译注（原文疑点）**：最后一句过于绝对。共享所有权确实限制了直接取得可变引用，但并不禁止可变性：例如
> `Rc<RefCell<T>>` 提供内部可变性，`Rc::get_mut`
> 在满足唯一性条件时允许修改，`Rc::make_mut`
> 则支持写时克隆。对本模式而言，保持共享节点不可变的价值是让旧版本继续表示旧状态；若直接修改共享节点，旧树和新树都可能观察到变化，便失去了这种版本隔离。参见
> [`Rc` 的可变性说明](https://doc.rust-lang.org/std/rc/index.html)与
> [`Rc::make_mut`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut)。

## 另请参阅

迭代器也有一个 `fold` 方法，但它是把数据结构折叠为一个值，而不是新的数据结构。
迭代器的 `map` 更接近本章的折叠模式。

> **译注（原文疑点）**：“一个值”也可以是集合或树，`Iterator::fold`
> 并不限制结果的数据结构。这里主要应区分两种组织方式：迭代器的 `fold`
> 用每个元素更新一个累积值，而本章的模式按节点递归转换结构。两者的表达能力存在重叠，不能仅凭结果是否为数据结构来区分。参见
> [`Iterator::fold`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold)。

在其他语言中，fold 通常采用 Rust 迭代器中的含义，而不是本章这个模式的含义。
一些函数式语言提供了强大的语言结构，能够灵活地映射数据结构。

[访问者](../behavioural/visitor.md)模式与折叠模式密切相关：两者都会遍历数据结构，并对每个节点执行某种操作。
但访问者既不会创建新的数据结构，也不会消费原有的数据结构。
