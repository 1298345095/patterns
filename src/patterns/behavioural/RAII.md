# 借助守卫实现 RAII

## 描述

[RAII][wikipedia] 是“Resource Acquisition is
Initialisation”（资源获取即初始化）的缩写，
这个名称实在不够贴切。该模式的核心是：在对象的构造过程中初始化资源，在析构过程中完成资源的清理。
Rust 进一步扩展了这个模式：让一个 RAII 对象充当某项资源的守卫（guard），
并依靠类型系统确保所有访问都必须经过这个守卫对象。

## 示例

标准库中的互斥锁守卫就是这个模式的经典例子（下面是真实实现的简化版本）：

```rust,ignore
use std::ops::Deref;

struct Foo {}

struct Mutex<T> {
    // 这里保存对数据 T 的引用。
    //..
}

struct MutexGuard<'a, T: 'a> {
    data: &'a T,
    //..
}

// 显式地对互斥锁加锁。
impl<T> Mutex<T> {
    fn lock(&self) -> MutexGuard<T> {
        // 锁定底层操作系统的互斥锁。
        //..

        // MutexGuard 保存对 self 的引用
        MutexGuard {
            data: self,
            //..
        }
    }
}

// 用于解锁互斥锁的析构函数。
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        // 解锁底层操作系统的互斥锁。
        //..
    }
}

// 实现 Deref 后，就可以把 MutexGuard 当作指向 T 的指针使用。
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

fn baz(x: Mutex<Foo>) {
    let xx = x.lock();
    xx.foo(); // foo 是 Foo 的一个方法。
              // 借用检查器确保我们无法保存一个比守卫 xx
              // 活得更久的、指向底层 Foo 的引用。

    // 退出这个函数时会执行 xx 的析构函数，x 随之解锁。
}
```

> **译注（原文疑点）**：这段标为 `ignore`
> 的代码是原文的概念示意，不能直接编译。它省略了 `Mutex<T>` 实际存放数据的字段和
> `Foo::foo` 的实现；此外，`data` 声明为 `&T`，而初始化时的 `self` 是
> `&Mutex<T>`，两者类型不匹配。阅读时应关注“取得锁 → 获得守卫 → 通过守卫访问数据
> → 守卫析构时解锁”这一关系，不要把它当作可用的互斥锁实现。

## 动机

如果某项资源必须在使用后完成清理，就可以用 RAII 来执行这项工作。
如果资源清理之后再访问它属于错误，也可以用这个模式来防止此类错误。

## 优点

既能防止资源未被清理的错误，也能防止在清理后继续使用资源的错误。

> **译注（原文疑点）**：这里对“防止资源未被清理”的保证需要限定：Rust
> 并不保证析构函数在所有情况下都会运行，例如安全代码也可以调用
> `mem::forget`，进程直接退出时也可能不执行析构。因此，实现安全抽象时，不能把“调用者一定会执行守卫的析构函数”当作内存安全的前提。RAII
> 让正常的所有权释放路径自动清理资源，却不是绝对的资源回收保证。参见
> [`mem::forget` 的安全性说明](https://doc.rust-lang.org/std/mem/fn.forget.html#safety)。

## 讨论

RAII 是确保资源得到正确释放或清理的一种实用模式。 借助 Rust
的借用检查器，我们可以在编译期防止资源清理后仍被使用所导致的错误。

借用检查器的核心目标，是确保指向数据的引用不会比数据本身活得更久。 RAII
守卫模式之所以有效，是因为守卫对象持有底层资源的引用，并且只通过自身向外暴露对该资源的引用。
Rust
保证守卫的生命周期不会超过底层资源，也保证经由守卫取得的资源引用不会比守卫活得更久。
要理解它如何工作，可以看一下不省略生命周期时的 `deref` 签名：

```rust,ignore
fn deref<'a>(&'a self) -> &'a T {
    //..
}
```

返回的资源引用与 `self` 的引用具有相同的生命周期（`'a`）。
因此，借用检查器会保证指向 `T` 的引用的生命周期短于 `self` 的生命周期。

> **译注（学习提示）**：这里“短于”应理解为“不超过”，并不要求严格更短。关键在于：返回引用绑定的是本次对守卫的借用
> `&'a self`，而不是守卫内部保存的、可能更长的资源引用的生命周期。这样，只要调用者还要使用返回引用，就不能先销毁守卫并解锁；这正是守卫作为访问凭证所维护的不变量。

注意，实现 `Deref` 并不是这个模式的核心，它只是让守卫对象更易于使用。
在守卫上实现一个 `get` 方法也同样有效。

### 使用 Rust 1.99.1 的标准库守卫

> **译注（版本说明）**：前面的 `MutexGuard` 是保留自原书的概念示意，
> 不是可编译的互斥锁实现。下面是无需自写 `unsafe` 的标准库对照例子：

```rust
use std::sync::Mutex;

fn main() {
    let items = Mutex::new(vec![1]);
    {
        let mut guard = items.lock().expect("mutex poisoned");
        guard.push(2);
    } // guard 被销毁时解除锁定
    assert_eq!(*items.lock().unwrap(), vec![1, 2]);
}
```

> **译注（学习提示）**：`Mutex::lock` 返回 `LockResult<MutexGuard<_>>`；
> 遇到毒化状态时，`expect` 会 panic，但不是说底层锁永远不能再使用。
> 调用方可以基于不变量审查决定是否通过 `PoisonError::into_inner`
> 继续访问。`Drop` 保证正常退出或发生栈展开时释放守卫，
> 不保证异常退出、`panic = "abort"` 或 OS 强制终止时运行。
> 本章引用的 Rust 1.0 风格指南占位页是**历史资料**，
> 请以当前 [`MutexGuard`](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html)
> 和 [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
> 文档判断现行行为。

## 另请参阅

[在析构函数中执行收尾工作的惯用法](../../idioms/dtor-finally.md)

RAII 在 C++ 中也是一种常见模式：
[cppreference.com](https://web.archive.org/web/20260918073742/https://en.cppreference.com/cpp/language/raii)、
[维基百科][wikipedia]。

[wikipedia]: https://en.wikipedia.org/wiki/Resource_Acquisition_Is_Initialization

[风格指南中的相关条目](https://doc.rust-lang.org/1.0.0/style/ownership/raii.html)
（目前还只是一个占位条目）。
