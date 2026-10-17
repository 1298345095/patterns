# 向闭包传递变量

## 描述

默认情况下，闭包通过借用来捕获其环境。也可以使用 `move`
闭包，将整个环境移入闭包。不过，我们往往希望只移动部分变量，给闭包某些数据的副本，通过引用传递另一些数据，或者先进行其他转换。

> **译注（原文疑点）**：这段描述做了简化。没有 `move`
> 时，编译器会根据闭包体如何使用变量来选择捕获方式；如果闭包体需要消耗一个非
> `Copy` 值，也可能按值捕获。`move`
> 则要求按值捕获闭包实际用到的环境内容，并不会移动周围所有局部变量；被捕获的若是引用，移入的也是引用本身。参见
> [闭包捕获规则](https://doc.rust-lang.org/reference/expressions/closure-expr.html)。

可以通过在一个独立作用域内重新绑定变量来实现。

## 示例

使用下面的写法：

```rust
use std::rc::Rc;

let num1 = Rc::new(1);
let num2 = Rc::new(2);
let num3 = Rc::new(3);
let closure = {
    // `num1` 被移动
    let num2 = num2.clone();  // `num2` 被克隆
    let num3 = num3.as_ref();  // `num3` 被借用
    move || {
        *num1 + *num2 + *num3;
    }
};
```

而不是：

```rust
use std::rc::Rc;

let num1 = Rc::new(1);
let num2 = Rc::new(2);
let num3 = Rc::new(3);

let num2_cloned = num2.clone();
let num3_borrowed = num3.as_ref();
let closure = move || {
    *num1 + *num2_cloned + *num3_borrowed;
};
```

> **译注（学习提示）**：这里克隆 `Rc`
> 只会增加引用计数，并不会深拷贝其中的整数；`num3.as_ref()`
> 则产生借用，不增加引用计数。因此，即使闭包用了 `move`，它仍然受外部 `num3`
> 的存活时间约束，不能据此认定闭包满足
> `'static`。另外，闭包体的加法后有分号，计算结果会被丢弃，闭包返回的是
> `()`；示例的重点是捕获方式。参见
> [`Rc` 的克隆说明](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.clone)。

## 优点

复制数据的代码与闭包定义放在一起，意图更加清楚；即使这些数据没有被闭包消耗，也会立即被释放。

> **译注（学习提示）**：此处“立即被释放”指没有被闭包捕获、因而留在内层代码块中的局部值，会在代码块结束时析构。已按值捕获的值成为闭包对象的一部分，会随闭包对象的存活时间保留，而不会在构造闭包的代码块结束时一并释放。这个小作用域明确了“准备捕获值”的边界。

不论数据是复制还是移动，闭包内部都能使用与外层代码相同的变量名。

## 缺点

闭包体需要多缩进一层。
