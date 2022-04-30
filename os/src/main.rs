//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![no_std]
#![no_main]
#![deny(missing_docs)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{Ordering, AtomicBool, AtomicUsize};
use config::{CPU_NUM, CONTROL_CPU};

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod harts;
mod config;
mod heap_allocator;

global_asm!(include_str!("entry.asm"));


static FIRST_BOOT: AtomicBool = AtomicBool::new(false);
static GLOBAL_INIT: AtomicBool = AtomicBool::new(false);
static BOOTED_CPU_NUM: AtomicUsize = AtomicUsize::new(0);


/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> ! {
    let cpu_id = harts::id();
    // 选择最初的核来进行全局初始化
    if select_as_first(){
        println!("I am FIRST CPU {:x}", cpu_id);
        clear_bss();
        heap_allocator::init_heap();
        extern "C" {
            fn stext();               // begin addr of text segment
            fn etext();               // end addr of text segment
            fn srodata();             // start addr of Read-Only data segment
            fn erodata();             // end addr of Read-Only data ssegment
            fn sdata();               // start addr of data segment
            fn edata();               // end addr of data segment
            fn sbss();                // start addr of BSS segment
            fn ebss();                // end addr of BSS segment
            fn boot_stack();          // stack bottom
            fn boot_stack_top();      // stack top
        }
        println!("Hello, world!");
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            "boot_stack [{:#x}, {:#x})",
            boot_stack as usize, boot_stack_top as usize
        );
        println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
        finish_global_init();
    }
    wait_global_init();
    println!("Hello world from CPU {:x}!", cpu_id);
    boot_finish();
    wait_all_booted();
    if cpu_id == CONTROL_CPU{
        panic!("Shutdown machine!");
    }
    else { loop{} }
}

/// select FIRST_CPU
pub fn select_as_first() -> bool {
    FIRST_BOOT.compare_exchange(false, true, Ordering::Release, Ordering::Relaxed).is_ok()
}

/// FIRST_CPU finish global init
pub fn finish_global_init() {
    GLOBAL_INIT.store(true, Ordering::Relaxed)
}

/// wait until global init finished
pub fn wait_global_init() {
    while !GLOBAL_INIT.load(Ordering::Relaxed){
        spin_loop();
    }
}

/// count booted cpu
pub fn boot_finish() {
    BOOTED_CPU_NUM.fetch_add(1, Ordering::Relaxed);
}

/// wait until ALL booted
pub fn wait_all_booted() {
    while !BOOTED_CPU_NUM.load(Ordering::Relaxed) == CPU_NUM{
        spin_loop();
    }
}