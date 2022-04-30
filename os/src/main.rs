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
#![feature(panic_info_message)]

use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{Ordering, AtomicBool, AtomicUsize};

use config::CPU_NUM;

use crate::config::FIRST_CPU;


#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod harts;
mod config;

global_asm!(include_str!("entry.asm"));


static FIRST_BOOT: AtomicBool = AtomicBool::new(false);
static BOOTED_CPU_NUM: AtomicUsize = AtomicUsize::new(0);


/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

// boot in order
/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> ! {
    let cpu_id = harts::id();
    if cpu_id == FIRST_CPU{
        println!("I am FIRST CPU {:x}", cpu_id);
        clear_bss();
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
        first_booted(cpu_id);
    }
    else{
        while !first_booted(cpu_id) {
            spin_loop();
        }
    } 
    boot_finish();
    println!("Hello world from CPU {:x}!", cpu_id);
    while !all_booted() {
        spin_loop();
    }
    if cpu_id == FIRST_CPU {
        panic!("Shutdown machine!");
    }
    else {
        loop{};
    }
}

pub fn first_booted(cpu_id: usize) -> bool {
    if cpu_id == FIRST_CPU {
        FIRST_BOOT.compare_exchange(false, true, Ordering::Release, Ordering::Relaxed).unwrap();
        true
    } else {
        FIRST_BOOT.load(Ordering::Acquire)
    }
}

pub fn boot_finish() {
    BOOTED_CPU_NUM.fetch_add(1, Ordering::Relaxed);
}

pub fn all_booted() -> bool {
    BOOTED_CPU_NUM.load(Ordering::Relaxed) == CPU_NUM
}