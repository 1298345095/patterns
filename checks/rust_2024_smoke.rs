//! Rust Design Patterns 中文译本的 Rust 2024 / stable 自检用例。
//! 仅依赖标准库；不包含原书那些有意不完整的 FFI 示意代码。
//! 运行：rustc --edition=2024 --test checks/rust_2024_smoke.rs -o /tmp/rust_2024_smoke && /tmp/rust_2024_smoke
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, CStr, CString};

/// # Safety
/// `message` 必须是调用期间有效、不可变、非空且以 NUL 结尾的字符串。
// SAFETY: 此示例的导出符号以项目命名空间为前缀；集成时仍须检查符号冲突。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_patterns_zh_is_utf8(message: *const c_char) -> bool {
    // SAFETY: 函数调用者负责满足上述指针与存储约束。
    unsafe { CStr::from_ptr(message) }.to_str().is_ok()
}

/// 返回一个只应由 rust_patterns_zh_free_message 释放的拥有型 C 字符串。
// SAFETY: 集成时须确认此符号与其他导出符号不冲突。
#[unsafe(no_mangle)]
pub extern "C" fn rust_patterns_zh_make_message() -> *mut c_char {
    CString::new("你好").expect("常量字符串不含 NUL").into_raw()
}

/// # Safety
/// `message` 必须为空或来自 rust_patterns_zh_make_message，且尚未被释放；
/// 不得改变该 NUL 终止字符串长度。
// SAFETY: 集成时须确认此符号与其他导出符号不冲突。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_patterns_zh_free_message(message: *mut c_char) {
    if !message.is_null() {
        // SAFETY: 调用者负责遵守来源及唯一释放的约定。
        drop(unsafe { CString::from_raw(message) });
    }
}

pub struct RustPatternsHandle {
    _reserved: u8,
}

// SAFETY: 导出的 C 符号名在集成时需要全局唯一。
#[unsafe(no_mangle)]
pub extern "C" fn rust_patterns_zh_handle_new() -> *mut RustPatternsHandle {
    Box::into_raw(Box::new(RustPatternsHandle { _reserved: 0 }))
}

/// # Safety
/// 指针必须为空或来自 rust_patterns_zh_handle_new，且尚未释放，也没有外部继续使用。
// SAFETY: 导出的 C 符号名在集成时需要全局唯一。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_patterns_zh_handle_free(handle: *mut RustPatternsHandle) {
    if !handle.is_null() {
        // SAFETY: 调用者保证此指针源于 Box::into_raw，且只释放一次。
        drop(unsafe { Box::from_raw(handle) });
    }
}

// 仅演示 Rust 2024 的外部声明语法；缺少外部实现，不能调用。
#[allow(dead_code)]
unsafe extern "C" {
    fn rust_patterns_zh_foreign_logger(message: *const c_char);
}

trait Base {
    fn id(&self) -> u32;
}
trait Extended: Base {}
struct Record;
impl Base for Record { fn id(&self) -> u32 { 42 } }
impl Extended for Record {}
fn upcast(x: &dyn Extended) -> &dyn Base { x }

trait ViewSource {
    type View<'a> where Self: 'a;
    fn view(&self) -> Self::View<'_>;
}
struct Bytes(Vec<u8>);
impl ViewSource for Bytes {
    type View<'a> = &'a [u8] where Self: 'a;
    fn view(&self) -> Self::View<'_> { &self.0 }
}

fn c_buffer_to_str(bytes: &[u8]) -> Result<&str, Box<dyn std::error::Error>> {
    Ok(CStr::from_bytes_until_nul(bytes)?.to_str()?)
}

#[test]
fn cstr_contract_for_owned_value() {
    let message = CString::new("中文测试").unwrap();
    // SAFETY: CString 存活到调用结束，内容不变。
    assert!(unsafe { rust_patterns_zh_is_utf8(message.as_ptr()) });
}

#[test]
fn message_is_freed_by_matching_allocator() {
    let message = rust_patterns_zh_make_message();
    assert!(!message.is_null());
    // SAFETY: 刚取得的原始指针只会释放一次。
    unsafe { rust_patterns_zh_free_message(message) };
}

#[test]
fn opaque_handle_round_trip() {
    let handle = rust_patterns_zh_handle_new();
    assert!(!handle.is_null());
    // SAFETY: 尚未释放，也无其他使用者。
    unsafe { rust_patterns_zh_handle_free(handle) };
}

#[test]
fn bounded_cstr_avoids_unbounded_scan() {
    assert_eq!(c_buffer_to_str(b"hello\0ignored").unwrap(), "hello");
}

#[test]
fn trait_upcasting_and_lending_view() {
    let record = Record;
    assert_eq!(upcast(&record).id(), 42);
    let data = Bytes(vec![1, 2, 3]);
    assert_eq!(data.view(), &[1, 2, 3]);
}

#[test]
fn tail_expression_temporary_drops_before_local_in_2024() {
    use std::cell::RefCell;
    fn len() -> usize {
        let cell = RefCell::new(vec![10, 20, 30]);
        cell.borrow().len()
    }
    assert_eq!(len(), 3);
}
