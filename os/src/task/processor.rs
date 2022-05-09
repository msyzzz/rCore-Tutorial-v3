use super::{TaskContext, TaskControlBlock};
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
use super::{fetch_task, TaskStatus};
use super::__switch;
use crate::trap::TrapContext;
use spin::Mutex;
use crate::config::CPU_NUM;
use crate::harts::id;
use crate::task::add_task;

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
}

// lock each processor, not the vec
lazy_static! {
    pub static ref PROCESSOR:Vec<Mutex<Processor>> = {
        let mut pro_vec = Vec::new();
        for i in 0..CPU_NUM{
            pro_vec.push(Mutex::new(Processor::new()))
        }
        pro_vec
    };
}

pub fn run_tasks() {
    loop {
        let cpu_id = id();
        if let Some(task) = fetch_task() {
            let mut processor = PROCESSOR[cpu_id].lock();
            //println!("cpu {} get task {}", cpu_id, task.pid.0);
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(
                    idle_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
            // add task when back to idle
            let mut processor = PROCESSOR[cpu_id].lock();
            if let Some(c_task) = processor.take_current() {
                let inner = c_task.inner_exclusive_access();
                if inner.task_status == TaskStatus::Ready{
                    drop(inner);
                    add_task(c_task);
                }
            }
            drop(processor);
        }
    }

}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR[id()].lock().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR[id()].lock().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().inner_exclusive_access().get_trap_cx()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR[id()].lock();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(
            switched_task_cx_ptr,
            idle_task_cx_ptr,
        );
    }
}
