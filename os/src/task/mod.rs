//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::{get_app_data, get_num_app};
use crate::trap::TrapContext;
use alloc::vec::Vec;
use crate::harts::id;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use crate::config::{CPU_NUM};
use spin::Mutex;

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: Mutex<TaskManagerInner>,
}

/// Inner of Task Manager
struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: [usize; CPU_NUM],
    free_cpu: usize,
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: {
                Mutex::new(TaskManagerInner {
                    tasks,
                    current_task: [num_app - 1; CPU_NUM],  // 保证任务的执行顺序
                    free_cpu: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.lock();
        let cpu_id = id();
        if let Some(next) = self.find_next_task(&inner) {
            inner.current_task[cpu_id] = next;
        } else {
            inner.free_cpu += 1;
            println!("[kernel] cpu {} free", id());
            drop(inner);
            loop {};
        }
        let first_task = inner.current_task[cpu_id];
        // println!("[kernel] cpu{} run task {}",cpu_id,first_task);
        let task0 = &mut inner.tasks[first_task];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    // /// Change the status of current `Running` task into `Ready`.
    // fn mark_current_suspended(&self) {
    //     let mut inner = self.inner.lock();
    //     let cpu_id = id();
    //     let current = inner.current_task[cpu_id];
    //     inner.tasks[current].task_status = TaskStatus::Ready;
    // }
    //
    // /// Change the status of current `Running` task into `Exited`.
    // fn mark_current_exited(&self) {
    //     let mut inner = self.inner.lock();
    //     let current = inner.current_task[id()];
    //     inner.tasks[current].task_status = TaskStatus::Exited;
    // }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self, inner: &TaskManagerInner) -> Option<usize> {
        // let inner = self.inner.lock();
        let current = inner.current_task[id()];
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.lock();
        inner.tasks[inner.current_task[id()]].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.lock();
        inner.tasks[inner.current_task[id()]].get_trap_cx()
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self, status: TaskStatus) {
        let mut inner = self.inner.lock();
        let cpu_id = id();
        let current = inner.current_task[cpu_id];
        inner.tasks[current].task_status = status;
        if let Some(next) = self.find_next_task(&inner) {
            //println!("[kernel] cpu{} run task {}",cpu_id,next);
            let current = inner.current_task[cpu_id];
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task[cpu_id] = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            // let mut inner = self.inner.lock();
            inner.free_cpu += 1;
            println!("[kernel] cpu {} free", id());
            if inner.free_cpu == CPU_NUM {
                panic!("All applications completed!");
            } else {
                drop(inner);
                loop {};
            }
        }
    }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

// /// rust next task
// fn run_next_task() {
//     TASK_MANAGER.run_next_task();
// }
//
// /// suspend current task
// fn mark_current_suspended() {
//     TASK_MANAGER.mark_current_suspended();
// }
//
// /// exit current task
// fn mark_current_exited() {
//     TASK_MANAGER.mark_current_exited();
// }

/// mark current task with status
fn mark_current_and_run_next(status: TaskStatus) {
    TASK_MANAGER.run_next_task(status);
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_and_run_next(TaskStatus::Ready);
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_and_run_next(TaskStatus::Exited);
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}