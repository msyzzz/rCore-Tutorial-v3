# rCore-Tutorial多核实现



## Chapter 3

> 实现多核分时多任务系统



### rust_main的boot过程

在ch1中我们搭建的对于不同核要进行不同的行为设定的框架要开始填充本ch的内容了。具体地说，除了clear_bss外，用户程序加载也是只进行一次的任务，而trap初始化和时钟中断、定时器的设置则要每个核自行设定。

```rust
if cpu_id == CONTROL_CPU{
    println!("Global initialization start...");
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
```

在`wait_all_booted`后，我们由ch1的直接shutdown变为开始运行用户程序，让各个cpu同时启动task的执行。

### TaskManager的变化

保证主要结构不变，于是本阶段的多核仍然是按照单核基本逻辑来进行的一个单队列调度。

我们首先要做的是把单核下的安全结构`UPSafeCell`改为多核互斥锁`Mutex`：

```rust
inner: Mutex<TaskManagerInner>, //定义
self.inner.lock();	//获取
```

其后，对于inner中的current，将其扩展为记录所有核的current的数组。

```
current_task: [usize; CPU_NUM],
```

最后，添加一个记录已完成任务的核数量的变量：

```rust
free_cpu: usize,
```

其用途将在下面描述。

### run_first_task

单核的run_first_task直接指定从零运行即可，而多核则需要费点功夫。直接给每个核指定任务是不合理的，原因之一便是每次运行每个核进入任务执行模块的快慢是不确定的。我们利用`find_next_task`方法，让处理机自己寻找要执行的任务，填充到`current[id]`里面，之后正常切换执行流开始执行。

```rust
fn run_first_task(&self) -> ! {
    let mut inner = self.inner.lock();
    let cpu_id = id();
    if let Some(next) = self.find_next_task() {
        inner.current_task[cpu_id] = next;
    }
    let first_task = inner.current_task[cpu_id];
    // println!("[kernel] cpu{} run task {}",cpu_id,first_task);
    let task0 = &mut inner.tasks[first_task];
    ..
}
```

这里需要注意的一点是，我们应该保证任务执行的大致顺序不变，于是要使find从0开始，结合find的逻辑，对`current_task`的初始化需要设置为`num_app - 1`,这样遍历时才会从`(num_app - 1 + 1) % num_app = 0 `开始进行。

### run_next_task

`run_next_task`的主要逻辑没有需要改变的，需要变化的是下面的else也即执行结束部分。类似ch1的main，单核直接panic，而多核则需要确保所有处理机执行完毕。这就要用到上面提到的`free_cpu`了。

```rust
else{
    inner.free_cpu += 1;
    println!("[kernel] cpu {} free",id());
    if inner.free_cpu == CPU_NUM{
        panic!("All applications completed!");
    }
    else {
        drop(inner);
        loop{};
    }
}
```

类似地，每次找不到新任务执行，则置当前cpu为空闲，计数加一，并原地loop，直到最后一个完成的cpu触发panic。

要注意在`run_first_task`，当任务数不足，会有cpu直接空闲，同样要添加计数，但由于任务数不会为0不用考虑panic。

```rust
else{
    inner.free_cpu += 1;
    println!("[kernel] cpu {} free",id());
    drop(inner);
    loop{};
}
```

### 确保加锁

看上去已经完成了多核的任务执行，尝试运行或许也可以得到正常的结果，但多运行几次便会发现死掉的情况。调试一下，可以发现实际上是有多个核抢到了同一个任务，而这显然会发生异常。这个现象的原因是按照原有的设计，在run_first/next_task中会调用find_next_task取出任务，但是在这一调用过程中，会有短暂的锁释放，于是当前核的任务调度可能被打断，发生当前核取出了任务但没来得及更改状态而下一个核同样取出的情况。

于是，我们将run中已经acquire lock的inner传入find_next_task中而不重新获得锁，让inner在整个run的过程中一直被当前cpu持有。

```rust
fn find_next_task(&self, inner: &TaskManagerInner) -> Option<usize>
if let Some(next) = self.find_next_task(&inner)
```

对于`suspend/exit_current_and_run_next`的原本分两步进行的做法，我们同样将其整合到run_next_task中，以保证一次调度的完整性。

```rust
fn mark_current_and_run_next(status: TaskStatus) {
    TASK_MANAGER.run_next_task(status);
}

fn run_next_task(&self, status:TaskStatus) {
    let mut inner = self.inner.lock();
    let cpu_id = id();
    let current = inner.current_task[cpu_id];
    inner.tasks[current].task_status = status;
    ..
}
```

顺利地实现多核的正常执行与停止：

```
..
Test power_3 OK!
[kernel] Application exited with code 0
[kernel] cpu 0 free
Test sleep OK!
[kernel] Application exited with code 0
[kernel] cpu 3 free
[kernel] Panicked at src/task/mod.rs:160 All applications completed!
```

