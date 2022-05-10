# rCore-Tutorial多核实现



## Chapter 5

> 多处理机调度



### 继承的基础改动

继承ch4的SMP改动，我们在`rust_main`中需要把

```rust
task::add_initproc();
println!("after initproc!");
loader::list_apps();
```

这些全局初始化代码放在`CONTROL_CPU`负责的部分里。

此外，对于`task_inner`、`TASK_MANAGER`和`PID_ALLOCATOR`也进行Mutex转换，并修改`task_inner`的`inner_exclusive_access`方法。

```rust
pub fn inner_exclusive_access(&self) -> MutexGuard<TaskControlBlockInner> {
    self.inner.lock()
}
```

### Multiprocessor

检查`UPSafeCell`的使用，会发现除了上面提到的几个，还有一个`PROCESSOR`。`processor.rs`里是我们的处理器管理结构，它直接监听并管理了CPU的运行状态，因此我们要想进行多核调度，这里是主要的实现之处。

首先要把单CPU实例变为多CPU实例，采用Vec数据结构，将多个Mutex保护的`Processor`放入队列之中。

```rust
lazy_static! {
    pub static ref PROCESSOR:Vec<Mutex<Processor>> = {
        let mut pro_vec = Vec::new();
        for i in 0..CPU_NUM{
            pro_vec.push(Mutex::new(Processor::new()))
        }
        pro_vec
    };
}
```

每次使用时，需要首先用`harts::id()`获取当前cpu ID，再在Vec中取出对应位置的`Processor`。这样的话，所有的`PROCESSOR.exclusive_access()`将变为`PROCESSOR[id()].lock()`

看上去好像是完成了，较为完备的单核rCore让我们只需要很少的改动便可以转换为多核情况，但尝试运行测例，会发现有时可能有内核的PageFault，有时可能会卡死不动，这就需要我们再去寻找多核环境的冲突部分。

### switch的安全保证

重新回顾进程的执行与切换过程，switch是很重要的一部分。`__switch`发生在`run_tasks()`和`suspend/exit_current_and_run_next()`几处。

事实上，细心观察可以发现，在`run_tasks()`的`__switch`之前有drop操作，由于switch后的地点不确定，需要在switch前放弃锁；但是这样一来，switch就不在当前cpu的保护之下了。另一方面，`suspend`的switch同样不涉及当前cpu的锁，不被保护。于是就会发生这种可能：cpu进入`suspend`，放弃了一个task，然后进入`__switch`返回idle；但在`__switch`执行过程中，被放回任务队列的该task被另一个cpu的`run_tasks()`取出，同样进入`__switch`，这样如果后者更快，它将读到还未被保存的错误的`task_cx`，于是它的运行出现了问题。

为了解决这一问题，我们对放回队列这一行为下手。只要在写好task_cx后再放回任务队列，就不会产生switch冲突。

将`suspend`中的`add_task`删除，在`run_tasks()`的`__switch`之后等待控制流回到idle再进行`add_task`操作。

```rust
// fn run_tasks()
..
unsafe {
    __switch(
        idle_task_cx_ptr,
        next_task_cx_ptr,
    );
}
// add task when back to idle
let mut processor = PROCESSOR[cpu_id].lock();
if let Some(c_task) = processor.take_current() {
    //println!("cpu {} drop task {}", cpu_id, c_task.pid.0);
    let inner = c_task.inner_exclusive_access();
    if inner.task_status == TaskStatus::Ready{
        drop(inner);
        add_task(c_task);
    }
}
drop(processor);
```

至于`exit`中的switch，由于不涉及当前task，故不需要改动。

### 潜在的死锁

现在尝试运行测例，应该是可以正常执行的。但如果我们继续检查用到多个lock的地方，会发现一些潜在的死锁情况。

在`exit_current_and_run_next()`中，我们有这样一段挂载当前进程的children到`INITPROC`的操作:

```rust
let mut inner = task.inner_exclusive_access();
.. #1
{
    let mut initproc_inner = INITPROC.inner_exclusive_access();
    for child in inner.children.iter() {
        child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
        initproc_inner.children.push(child.clone());
    }
}
```

考虑一种情况，父子进程同时进入了exit中。父进程稍快一些，它拿到了`INITPROC`的锁，准备访问子进程；但此时子进程刚执行到#1，在上面持有着自己的锁，于是父进程将等待。但是子进程的下一步是获取`INITPROC`，而它又被父进程占有，于是产生死锁。

因此我们需要调整一下获取锁的顺序，比如将获取`INITPROC`放到获取当前任务inner的前面进行，这样同一时间只能有一个处理器获取到initproc的锁来进行下面的操作，从而解决的父子锁的冲突。

```rust
let mut initproc_inner = INITPROC.inner_exclusive_access();
..
let mut inner = task.inner_exclusive_access();
..
for child in inner.children.iter() {
    child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
    initproc_inner.children.push(child.clone());
}
..
drop(initproc_inner);
```

由于多核情况下可能initproc会同时运行，不要忘记drop对应的结构。

现在我们解决了不少问题。运行一下测例看看：

```
after initproc!
/**** APPS ****
exit
fantastic_text
forktest
forktest2
forktest_simple
forktree
hello_world
initproc
matrix
sleep
sleep_simple
stack_overflow
user_shell
usertests
yield
**************/
Hello world from CPU 0!
Hello world from CPU 1!
Hello world from CPU 3!
Hello world from CPU 2!
Rust user shell
>> usertests
Usertests: Running exit
I am the parent. Forking the child...
..
yield pass.
Usertests: Test yield in Process 3 exited with code 0
Usertests passed!
Shell: Process 2 exited with code 0
>>
```

