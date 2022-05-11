# rCore-Tutorial多核实现



## Chapter 4

> 支持虚拟内存机制

### 简短的实现

事实上，我们只需要一点点的修改便能使现有的mm结构在SMP下运行起来。

在继承了ch3之后，首先进行基本的修改——把`UPSafeCell`改为`Lock`。此后我们只需要对mm的初始化做一点小小的改动：

```rust
// before
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}


// after
/// initiate heap allocator, frame allocator
pub fn allocator_init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
}

/// initiate kernel space
pub fn kernel_space_init() {
    KERNEL_SPACE.lock().activate();
}
```

将`KERNEL_SPACE`的init独立出来，因为两个allocator是全局的，所有cpu共用，而每个cpu需要单独进行分页机制的启用。于是在main中，我们按照之前的格式进行初始化：

```rust
if cpu_id == CONTROL_CPU{
    ..
    mm::allocator_init();
    println!("[kernel] back to world!");
    mm::remap_test();
    finish_global_init();
}
wait_global_init();
mm::kernel_space_init();
..
```

这样，多核操作系统便能支持虚存机制了。

```
load_fault APP running...

power_5 [power_3 [power_7 [Into Test load_fault, we will insert an invalid load operation...
100001000010000Kernel should kill this application!
///[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x1009c, kernel killed it.
210000300000240000]

store_fault APP running...

]
]
power_5 [Into Test store_fault, we will insert an invalid store operation...
power_3 [power_7 [2000020000//300000210000Kernel should kill this application!
]
]
20000[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x1009c, kernel killed it.
..
Test power_3 OK!
[kernel] Application exited with code 0
[kernel] cpu 3 free
Test sleep OK!
[kernel] Application exited with code 0
[kernel] cpu 2 free
[kernel] Panicked at src/task/mod.rs:170 All applications completed!
```

