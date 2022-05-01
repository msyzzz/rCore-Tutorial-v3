#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{Ordering, AtomicBool, AtomicUsize};
use config::{CPU_NUM};
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod syscall;
mod trap;
mod loader;
mod config;
mod task;
mod timer;
mod sync;
mod mm;
mod harts;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

static FIRST_BOOT: AtomicBool = AtomicBool::new(false);
static GLOBAL_INIT: AtomicBool = AtomicBool::new(false);
static BOOTED_CPU_NUM: AtomicUsize = AtomicUsize::new(0);

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            sbss as usize as *mut u8,
            ebss as usize - sbss as usize,
        ).fill(0);
    }
}

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main() -> ! {
    let cpu_id = harts::id();
    // 选择最初的核来进行全局初始化
    if select_as_first() {
        println!("I am cpu {}",cpu_id);
        clear_bss();
        println!("[kernel] Hello, world!");
        mm::allocator_init();
        println!("[kernel] back to world!");
        mm::remap_test();
        task::add_initproc();
        println!("after initproc!");
        loader::list_apps();
        finish_global_init();
    }
    wait_global_init();
    mm::kernel_space_init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    println!("Hello world from CPU {:x}!", cpu_id);
    boot_finish();
    wait_all_booted();
    task::run_tasks();
    panic!("Unreachable in rust_main!")
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
