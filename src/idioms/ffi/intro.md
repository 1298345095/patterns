# FFI 惯用法

编写 FFI
代码本身就足以构成一门完整的课程。不过，这里的一些惯用法可以提供指引，帮助尚不熟悉
`unsafe` Rust 的使用者避开陷阱。

本节介绍一些在进行 FFI 开发时可能有用的惯用法。

1. [符合惯用法的错误处理](./errors.md)——使用整数错误码和哨兵返回值（例如 `NULL`
   指针）处理错误。

2. 用尽量少的 unsafe 代码[接收字符串](./accepting-strings.md)。

3. 向 FFI 函数[传递字符串](./passing-strings.md)。
