# 使用借用类型作为参数

## 描述

选择函数参数的类型时，使用解引用强制转换的目标类型，可以提高代码的灵活性，让函数接受更多类型的输入。

这种做法不限于可以切片的类型或胖指针类型。事实上，应当始终优先使用**借用类型**，而不是**对拥有型类型的借用**。例如，优先使用
`&str` 而不是 `&String`，使用 `&[T]` 而不是 `&Vec<T>`，使用 `&T` 而不是
`&Box<T>`。

> **译注（原文疑点）**：“始终”应当结合函数需要的能力理解。如果函数只读取字符串内容，`&str`
> 就足够；如果函数确实需要查询 `String` 的容量，`&String`
> 仍有理由存在。同样，`&mut [T]` 允许修改现有元素，却不提供 `Vec::push`
> 所需的调整长度与容量的能力。应选择能表达操作需求的最小接口，而不是机械替换所有参数类型。参见
> [`String::capacity`](https://doc.rust-lang.org/std/string/struct.String.html#method.capacity)
> 和
> [`Vec::push`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push)。

当拥有型类型本身已经包含一层间接访问时，使用借用类型可以避免再增加间接访问层次。例如，`String`
包含一层间接访问，而 `&String` 就会有两层。改用 `&str`，并在调用函数时让
`&String` 强制转换为 `&str`，便可以避免这一点。

## 示例

本例将展示使用 `&String` 和 `&str` 作为函数参数的一些区别。同样的思路也适用于
`&Vec<T>` 与 `&[T]`，以及 `&Box<T>` 与 `&T`。

假设我们要判断一个单词是否包含三个连续的元音字母。完成这个判断不需要拥有字符串，因此我们接收一个引用。

代码可能如下所示：

```rust
fn three_vowels(word: &String) -> bool {
    let mut vowel_count = 0;
    for c in word.chars() {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => {
                vowel_count += 1;
                if vowel_count >= 3 {
                    return true;
                }
            }
            _ => vowel_count = 0,
        }
    }
    false
}

fn main() {
    let ferris = "Ferris".to_string();
    let curious = "Curious".to_string();
    println!("{}: {}", ferris, three_vowels(&ferris));
    println!("{}: {}", curious, three_vowels(&curious));

    // 这样可以正常工作，但下面两行会失败：
    // println!("Ferris: {}", three_vowels("Ferris"));
    // println!("Curious: {}", three_vowels("Curious"));
}
```

这段代码可以正常工作，因为传入的参数类型是
`&String`。如果去掉最后两行的注释，示例就会编译失败，因为 `&str` 不会强制转换为
`&String`。只要修改参数类型，就可以解决这个问题。

例如，把函数声明改为：

```rust, ignore
fn three_vowels(word: &str) -> bool {
```

这样，两种调用方式都能编译，并打印出相同的结果。

```bash
Ferris: false
Curious: true
```

不过，好处还不止这些！你可能会想：这对我没什么影响，反正我不会像传入 `"Ferris"`
那样，把 `&'static str` 作为输入。即使不考虑这个特例，你仍会发现 `&str` 比
`&String` 更灵活。

再来看一个例子：有人给了我们一个句子，我们想判断其中是否有单词包含三个连续的元音字母。我们应该复用已经定义好的函数，把句子中的各个单词依次传给它。

例如，可以这样写：

```rust
fn three_vowels(word: &str) -> bool {
    let mut vowel_count = 0;
    for c in word.chars() {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => {
                vowel_count += 1;
                if vowel_count >= 3 {
                    return true;
                }
            }
            _ => vowel_count = 0,
        }
    }
    false
}

fn main() {
    let sentence_string =
        "Once upon a time, there was a friendly curious crab named Ferris".to_string();
    for word in sentence_string.split(' ') {
        if three_vowels(word) {
            println!("{word} has three consecutive vowels!");
        }
    }
}
```

使用参数类型为 `&str` 的函数运行这个示例，会得到：

```bash
curious has three consecutive vowels!
```

然而，如果函数的参数类型是
`&String`，这个示例就无法运行。原因是字符串切片的类型是 `&str`，而不是
`&String`。要将其转换为
`&String`，就需要分配内存，而这种转换不会隐式发生；相比之下，从 `String` 转换为
`&str` 既廉价，又是隐式的。

> **译注（原文疑点）**：最后一句省略了“先借用”这一步。对普通函数调用而言，若
> `s: String`、参数要求 `&str`，应写 `f(&s)` 或 `f(s.as_str())`；`f(s)`
> 不会自动把拥有型的 `String` 变成借用。此处隐式发生的是 `&String` 到 `&str`
> 的解引用强制转换。反向操作则需要先构造一个拥有数据的 `String`，再借用它。参见
> [Rust Reference 的强制转换规则](https://doc.rust-lang.org/reference/type-coercions.html)。
>
> **译注（学习提示）**：`split`
> 返回的切片仍指向原字符串，因此这些单词可以直接借给
> `three_vowels`，无需为每个单词复制或分配存储。这个接口保留了调用者管理数据的自由，也明确表示函数只需要在调用期间读取内容；代码中的元音判定则仅覆盖小写的
> `a/e/i/o/u`，并非通用的自然语言元音识别。

## 另请参阅

- [Rust Reference：类型强制转换](https://doc.rust-lang.org/reference/type-coercions.html)
- 关于如何处理 `String` 和 `&str` 的更多讨论，可参阅 Herman J. Radtke III
  的[系列博文（2015 年）](https://web.archive.org/web/20201112023149/https://hermanradtke.com/2015/05/03/string-vs-str-in-rust-functions.html)。
- [Steve Klabnik 的博文：何时应使用 String，何时应使用 &str？](https://archive.ph/LBpD0)
