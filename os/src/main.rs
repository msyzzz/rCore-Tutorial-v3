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
#![feature(alloc_error_handler)]

use core::arch::global_asm;
use core::hint::spin_loop;
extern crate lock;
extern crate lazy_static;
extern crate alloc;
use alloc::sync::Arc;
use lock::Mutex;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod harts;
mod heap_alloc;
mod config;

global_asm!(include_str!("entry.asm"));

lazy_static::lazy_static! {
    static ref NOW_HART: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

static mut FIRST_BOOT: bool = false;


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
    if cpu_id == 0{
        println!("I AM CPU {:x}", cpu_id);
        clear_bss();
        heap_alloc::init_heap();
        // extern "C" {
        //     fn stext();               // begin addr of text segment
        //     fn etext();               // end addr of text segment
        //     fn srodata();             // start addr of Read-Only data segment
        //     fn erodata();             // end addr of Read-Only data ssegment
        //     fn sdata();               // start addr of data segment
        //     fn edata();               // end addr of data segment
        //     fn sbss();                // start addr of BSS segment
        //     fn ebss();                // end addr of BSS segment
        //     fn boot_stack();          // stack bottom
        //     fn boot_stack_top();      // stack top
        // }
        // println!("Hello, world!");
        // println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        // println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        // println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        // println!(
        //     "boot_stack [{:#x}, {:#x})",
        //     boot_stack as usize, boot_stack_top as usize
        // );
        // println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
        // panic!("Shutdown machine!");
        check_and_init(cpu_id);
    }
    else{
        while !check_and_init(cpu_id) {
            unsafe{
                println!("{}", FIRST_BOOT);
            }
            spin_loop();
        }
        println!("Hello world from CPU {:x}!", cpu_id);
    } 
    println!("{:x}", cpu_id);
    unsafe{
        println!("{}", FIRST_BOOT);
    }
    while !check_all_cpu_started() {
        spin_loop();
    }
    if cpu_id == 3 {
        panic!("Shutdown machine!");
    }
    else {
        loop{};
    }
}

pub fn check_and_init(cpu_id: usize) -> bool {
    unsafe{
        if cpu_id == 0 {
            FIRST_BOOT = true;
        }
        if !FIRST_BOOT {
            return false;
        }
    }
    let mut id_now = NOW_HART.lock();
    if cpu_id != *id_now {
        false
    } else {
        //println!("idddddd{:x}", cpu_id);
        *id_now += 1;
        true
    }
}

pub fn check_all_cpu_started() -> bool {
    *NOW_HART.lock() == 4
}