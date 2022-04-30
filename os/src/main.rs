//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`task`]: Task management
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`task::run_first_task()`] and for the first time go to
//! userspace.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;
use core::hint::spin_loop;
use core::sync::atomic::{Ordering, AtomicBool, AtomicUsize};
use config::{CPU_NUM};

#[cfg(feature = "board_k210")]
#[path = "boards/k210.rs"]
mod board;
#[cfg(not(any(feature = "board_k210")))]
#[path = "boards/qemu.rs"]
mod board;
#[macro_use]
mod console;
mod lang_items;
mod loader;
mod sbi;
mod harts;
mod config;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));


static FIRST_BOOT: AtomicBool = AtomicBool::new(false);
static GLOBAL_INIT: AtomicBool = AtomicBool::new(false);
static BOOTED_CPU_NUM: AtomicUsize = AtomicUsize::new(0);


/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> ! {
    let cpu_id = harts::id();
    // 选择最初的核来进行全局初始化
    if select_as_first(){
        clear_bss();
        loader::load_apps();
        finish_global_init();
    }
    wait_global_init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    println!("Hello world from CPU {:x}!", cpu_id);
    boot_finish();
    wait_all_booted();
    task::run_first_task();
    panic!("unreachable")
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