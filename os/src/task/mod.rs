mod context;
mod switch;
mod task;
mod manager;
mod processor;
mod pid;

use crate::loader::get_app_data_by_name;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
use manager::fetch_task;
use lazy_static::*;

pub use context::TaskContext;
pub use processor::{
    run_tasks,
    current_task,
    current_user_token,
    current_trap_cx,
    take_current_task,
    schedule,
};
pub use manager::add_task;
pub use pid::{PidHandle, pid_alloc, KernelStack};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = current_task().unwrap();
    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    // println!("cpu {} sus task {}", id(), task.pid.0);
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    for child in inner.children.iter() {
        loop {
            // child first, INITPROC next
            if let Some(mut child_inner) = child.try_inner_access() {
                if let Some(mut start_proc_tcb_inner) = INITPROC.try_inner_access() {
                    child_inner.parent = Some(Arc::downgrade(&INITPROC));
                    start_proc_tcb_inner.children.push(child.clone());
                    // 拿到锁并修改完成后，退到外层循环去修改下一个子进程
                    break;
                }
            }
            // 只要没拿到任意一个锁，就继续循环
        }
    }
    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    // println!("cpu {} exit task {}", id(), task.pid.0);
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}
