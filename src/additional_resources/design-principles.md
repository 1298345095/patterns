# 设计原则

## 常见设计原则概览

---

## [SOLID](https://en.wikipedia.org/wiki/SOLID)

- [单一职责原则（SRP）](https://en.wikipedia.org/wiki/Single-responsibility_principle)：
  一个类应该只有一项职责。也就是说，只有软件规格中某一部分的变化，
  才应该影响这个类的规格。
- [开闭原则（OCP）](https://en.wikipedia.org/wiki/Open%E2%80%93closed_principle)：
  “软件实体……应该对扩展开放，对修改关闭。”
- [里氏替换原则（LSP）](https://en.wikipedia.org/wiki/Liskov_substitution_principle)：
  “程序中的对象应该能够被其子类型的实例替换，而不改变程序的正确性。”
- [接口隔离原则（ISP）](https://en.wikipedia.org/wiki/Interface_segregation_principle)：
  “多个面向特定客户端的接口，优于一个通用接口。”
- [依赖倒置原则（DIP）](https://en.wikipedia.org/wiki/Dependency_inversion_principle)：
  应该“依赖抽象，而不是具体实现”。

> **译注（学习提示）**：将这些原则用于 Rust 时，可以把“类”理解为一组需要一起
> 维护的不变量，把“接口”落实为公开函数和 trait 的契约。例如，两个类型都实现
> `Iterator<Item = T>`，说明它们具有共同的类型层面接口，却不表示它们的终止性、
> 顺序或副作用都相同。替换是否合理，还要看调用者依赖了哪些行为。
> 这是一种阅读这些原则的方法，并非把面向对象的子类型继承原样搬进 Rust。

## [组合复用原则（CRP），或组合优于继承](https://en.wikipedia.org/wiki/Composition_over_inheritance)

“类应该优先通过组合实现多态行为与代码复用，也就是在内部包含实现所需功能的
其他类的实例，而不是通过继承基类或父类来实现。” ——Kirk Knoernschild，*Java
Design — Objects, UML, and Process*（2002）。

## [DRY：不要重复自己](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)

“系统中的每一项知识，都必须有且仅有一个明确、权威的表达。”

> **译注（学习提示）**：这里重复的是“知识”，不只是相似的代码文本。
> 两段暂时相同、却因不同业务原因而变化的代码，不一定应该合并。
> 提炼抽象前，可以先问：它们未来是否必须一起修改？如果答案是否定的，
> 合并可能把本来独立的变化强行耦合起来。

## [KISS：保持简单](https://en.wikipedia.org/wiki/KISS_principle)

多数系统在保持简单、没有被复杂化时运行得最好。因此，简单性应该是设计的重要目标，
应当避免不必要的复杂性。

## [迪米特法则（LoD）](https://en.wikipedia.org/wiki/Law_of_Demeter)

一个对象应尽可能少地假设其他事物的结构或属性，包括其子组件的结构或属性。
这与“信息隐藏”原则相一致。

## [契约式设计（DbC）](https://en.wikipedia.org/wiki/Design_by_contract)

软件设计者应该为软件组件定义形式化、精确且可验证的接口规格，
在抽象数据类型通常的定义之外，进一步规定前置条件、后置条件和不变量。

## [封装](https://en.wikipedia.org/wiki/Encapsulation_(computer_programming))

把数据与操作这些数据的方法组织在一起，或者限制对对象某些组成部分的直接访问。
封装用于将结构化数据对象的值或状态隐藏在类的内部，防止未经许可的直接访问。

## [命令与查询分离（CQS）](https://en.wikipedia.org/wiki/Command%E2%80%93query_separation)

“函数不应该产生抽象层面的副作用……只有命令（过程）才允许产生副作用。” ——Bertrand
Meyer，*Object-Oriented Software Construction*。

> **译注（学习提示）**：不要把 CQS 简单等同于 `&self` 和 `&mut self` 的区别。
> `&self` 方法仍可能通过内部可变性更新状态，或者执行 I/O；反过来，
> 某些修改状态的方法也会返回有用的信息，例如 `Vec::pop`。
> 判断命令与查询时，应看方法对调用者可观察的状态产生了什么影响。
> 关于共享引用下的可变性，参见[标准库 `std::cell`](https://doc.rust-lang.org/std/cell/)。

## [最小惊讶原则（POLA）](https://en.wikipedia.org/wiki/Principle_of_least_astonishment)

系统组件的行为应符合大多数用户的预期，不应使用户感到意外或吃惊。

## 语言层面的模块单元（Linguistic-Modular-Units）

“模块必须与所使用语言中的语法单元相对应。” ——Bertrand Meyer，*Object-Oriented
Software Construction*。

## 自描述（Self-Documentation）

“模块的设计者应该努力使关于模块的所有信息，都成为模块自身的一部分。” ——Bertrand
Meyer，*Object-Oriented Software Construction*。

## 统一访问（Uniform-Access）

“模块提供的所有服务，都应该能够通过统一的记法访问；这种记法不应暴露
服务究竟是通过存储还是通过计算实现的。” ——Bertrand Meyer，*Object-Oriented
Software Construction*。

## 单一选择（Single-Choice）

“当软件系统必须支持一组选项时，系统中应该有且仅有一个模块知道这些选项的完整列表。”
——Bertrand Meyer，*Object-Oriented Software Construction*。

## 持久化闭包（Persistence-Closure）

“当存储机制存储一个对象时，必须连同该对象所依赖的对象一起存储。
当检索机制取回一个先前存储的对象时，也必须取回它所依赖且尚未被取回的对象。”
——Bertrand Meyer，*Object-Oriented Software Construction*。
