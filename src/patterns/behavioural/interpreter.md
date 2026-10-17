# 解释器

## 描述

如果某类问题经常出现，而且解决它需要漫长、重复的步骤，那么可以用一种简单的语言描述具体问题，
再由一个解释器对象解释用这种语言写出的语句，从而求解这些问题。

基本上，对于一类问题，我们会定义：

- 一种[领域专用语言](https://en.wikipedia.org/wiki/Domain-specific_language)。
- 这门语言的文法。
- 一个求解具体问题的解释器。

## 动机

我们的目标是将简单的数学表达式转换成后缀表达式，
也就是[逆波兰表示法](https://en.wikipedia.org/wiki/Reverse_Polish_notation)。
为简单起见，表达式只包含十个数字 `0`、……、`9`，以及 `+`、`-` 这两种运算。
例如，表达式 `2 + 4` 会被转换成 `2 4 +`。

## 本问题的上下文无关文法

我们的任务是把中缀表达式转换成后缀表达式。 下面为由 `0`、……、`9`、`+` 和 `-`
组成的一组中缀表达式定义一个上下文无关文法：

- 终结符：`0`、`...`、`9`、`+`、`-`。
- 非终结符：`exp`、`term`。
- 开始符号为 `exp`。
- 产生式规则如下。

```ignore
exp -> exp + term
exp -> exp - term
exp -> term
term -> 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
```

**注意：** 根据接下来的具体用途，这个文法可能还需要进一步转换，例如消除左递归。
更多细节请参阅[《编译原理》](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)
（Compilers: Principles, Techniques, and Tools，通常称为“龙书”）。

## 解决方案

我们直接实现一个递归下降解析器。为简单起见，表达式语法错误时，代码会触发 panic；
例如，按照文法定义，`2-34` 和 `2+5-` 都是错误的表达式。

```rust
pub struct Interpreter<'a> {
    it: std::str::Chars<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new(infix: &'a str) -> Self {
        Self { it: infix.chars() }
    }

    fn next_char(&mut self) -> Option<char> {
        self.it.next()
    }

    pub fn interpret(&mut self, out: &mut String) {
        self.term(out);

        while let Some(op) = self.next_char() {
            if op == '+' || op == '-' {
                self.term(out);
                out.push(op);
            } else {
                panic!("Unexpected symbol '{op}'");
            }
        }
    }

    fn term(&mut self, out: &mut String) {
        match self.next_char() {
            Some(ch) if ch.is_ascii_digit() => out.push(ch),
            Some(ch) => panic!("Unexpected symbol '{ch}'"),
            None => panic!("Unexpected end of string"),
        }
    }
}

pub fn main() {
    let mut intr = Interpreter::new("2+3");
    let mut postfix = String::new();
    intr.interpret(&mut postfix);
    assert_eq!(postfix, "23+");

    intr = Interpreter::new("1-2+3-4");
    postfix.clear();
    intr.interpret(&mut postfix);
    assert_eq!(postfix, "12-3+4-");
}
```

> **译注（原文疑点）**：前面为了便于阅读写成了 `2 + 4` 和
> `2 4 +`，但这段实现并不跳过空白，实际应传入 `"2+4"`，输出则是
> `"24+"`。它只处理单个 ASCII 数字，也不支持括号。`interpret`
> 用循环处理后续运算，相当于避开了文法中的直接左递归，并在每读完右操作数后输出运算符，从而保持加减运算的左结合顺序。
>
> **译注（学习提示）**：这里把解析与输出直接交织起来，避免了建立
> AST，但也意味着解析失败前可能已经向 `out`
> 写入部分结果。若将它发展为接收外部输入的接口，可以先决定失败时是否允许保留部分输出，再选择返回带位置信息的错误、使用临时缓冲区，或先构建
> AST 后输出。这个取舍涉及接口的失败语义，而不只是将 `panic!` 换成 `Result`。

## 讨论

一种常见误解是：解释器设计模式就是为形式语言设计文法，再实现对应的解析器。
实际上，这个模式关注的是以更专门的形式表达具体问题，并实现能够求解它们的函数、类或结构体。
Rust 的 `macro_rules!`
允许我们定义专门的语法，以及将这种语法展开为源代码的规则。

下面创建一个简单的 `macro_rules!` 宏，用于计算 `n` 维向量的
[欧几里得长度](https://en.wikipedia.org/wiki/Euclidean_distance)。 写成
`norm!(x,1,2)`，可能比先把 `x,1,2` 装进
`Vec`，再调用计算长度的函数更易于表达，也更高效。

```rust
macro_rules! norm {
    ($($element:expr),*) => {
        {
            let mut n = 0.0;
            $(
                n += ($element as f64)*($element as f64);
            )*
            n.sqrt()
        }
    };
}

fn main() {
    let x = -3f64;
    let y = 4f64;

    assert_eq!(3f64, norm!(x));
    assert_eq!(5f64, norm!(x, y));
    assert_eq!(0f64, norm!(0, 0, 0));
    assert_eq!(1f64, norm!(0.5, -0.5, 0.5, -0.5));
}
```

> **译注（原文疑点）**：宏体中每个 `$element` 都出现了两次，因此传入
> `norm!(f())` 这样的表达式时，`f()`
> 会被调用两次，结果也未必是同一个值的平方；本例只传入变量或字面量，所以没有暴露这个问题。若要支持任意表达式，应先求值一次并绑定临时变量，再计算平方。另外，普通函数也可以接收数组或切片，不必先分配
> `Vec`；是否更高效需要比较实际实现，不能仅由“使用了宏”得出结论。

## 另请参阅

- [解释器模式](https://en.wikipedia.org/wiki/Interpreter_pattern)
- [上下文无关文法](https://en.wikipedia.org/wiki/Context-free_grammar)
- [macro_rules!](https://doc.rust-lang.org/rust-by-example/macros.html)
