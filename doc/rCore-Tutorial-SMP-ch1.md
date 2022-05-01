# rCore-Tutorial多核实现



## Chapter 1

> 实现多核操作系统的正常启动执行



### 从Qemu启动开始

要使Qemu模拟的处理器变为多核，其实很简单，为其添加参数`smp`

```
-smp $(SMP)
```

我们设定SMP=4，Qemu就启动了一个四核处理器。接下来便要求我们修改我们操作系统启动流程，为每个核提供正常的启动空间。

### 修改entry.asm

`entry.asm`是我们内核的入口。在这里我们分配了启动栈空间并跳转到Rust函数入口。在多核条件下，我们需要为每个核生成其自己的启动栈空间，各块互不干扰。于是`entry.asm`变为了：

```assembly
    .section .text.entry
    .globl _start
_start:
    mv tp, a0

    add t0, a0, 1
    slli t0, t0, 16
    la sp, boot_stack
    add sp, sp, t0
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 16 * 4
    #.space 64 * 1024 * 2    # 64 K/cores * 2
    .globl boot_stack_top
boot_stack_top:
```

开始时，a0寄存器里有当前核的id，将其存入tp(thread pointer)中便于之后的获取使用；`boot_stack`由单一的64K变为了64K*4，为四个核提供等同的启动栈。主体部分便是按照id确定当前核所属栈的位置，设置好栈指针，之后跳转入rust_main中。

对于tp的使用，我们在`hart.rs`中通过汇编方法来读取cpu ID，暴露id接口供其他模块使用。

```rust
pub fn id() -> usize {
    let mut cpu_id;
    unsafe { asm!("mv {0}, tp", out(reg) cpu_id) };
    cpu_id
}
```

### 在rust_main进行多个核的boot

即使系统变为了多核，我们的Rust函数仍然只有一个。这要求我们对于不同核要进行不同的行为设定，比如`clear_bss`之类的全局初始化只要求一个核来进行而其他核不要进行，在之后还会有app的加载、分配器的初始化等等，这些全局行为都是只要求进行一次的。当然，各个核的启动也有共通的地方，在这里我们是一个`println!("Hello world")`，在之后的系统中可以是trap的init、时钟中断的开启等等。

因此我们指定一个`CONTROL_CPU`来进行全局初始化，这里选择CPU0。只有指定CPU能进入Global模块中，而其他核通过`wait_global_init`循环等待一个代表完成的flag，直到第一个核完成，通过`finish_global_init`修改flag，之后所有核开始共同的boot过程。

```rust
if cpu_id == CONTROL_CPU{
    println!("Global initialization start...");
    clear_bss();
    ..
    finish_global_init();
}
wait_global_init();
println!("Hello world from CPU {:x}!", cpu_id);
boot_finish();
```

这里的flag有同步互斥的要求。我们选取的是`AtomicBool`原子变量来实现各个核之间的同步：

```rust
static GLOBAL_INIT: AtomicBool = AtomicBool::new(false);

pub fn finish_global_init() {
    GLOBAL_INIT.store(true, Ordering::Relaxed)
}

/// wait until global init finished
pub fn wait_global_init() {
    while !GLOBAL_INIT.load(Ordering::Relaxed){
        spin_loop();
    }
}
```

采用`store`和`load`方法进行原子变量的读写操作，保证逻辑的正确性。

### 正常shutdown

有了启动我们还需要正常停止。按照原本的方法是在最后panic!，但是如果我们直接将panic!放在`boot_finish`下面，由于各个核的boot时长有长有短，可能有的还没有finish就被最先的panic掉了。那么还需要另一个方法来确保所有核均启动完成，我们利用另一个原子变量来统计已启动的核数，直至其与预设的`CPU_NUM`相等。

```rust
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
```

每个核`boot_finish`都会让`BOOTED_CPU_NUM`++，之后完成的核开始等待直到完成数达到预期。这时再panic则保证了所有核启动完成。

实际运行时出现了另一个问题，虽然保证了启动正常，但是同样也产生了多个核同时panic的可能，于是我们或许会看到

```
[kernel] Panicked at src/main.rs:80 Shutdown machine!
[kernel] Panicked at
```

这种“狗尾续貂”的显示。这里采用的做法是让`CONTROL_CPU`负责总控，仅它进行shutdown，其他核则一直loop。

```rust
if cpu_id == CONTROL_CPU{
    panic!("Shutdown machine!");
}
else { loop{} }
```

### 解决混乱的输出

事实上的启动逻辑我们基本已经完成了，但运行结果好像是天书：

```
HelloH HHweeeorllllo wordldlll from Co o wPor fld rowUfm  CPUo 3!r0
om CPUr 2l!
!d
 from C[PU
```

仔细分辨，发现应该是几个核的输出交汇重叠了，需要让每个核的每句print连贯起来。查看print的实现

```rust
Stdout.write_fmt(args).unwrap();
```

直接使用结构体函数而非某个确定实例的方法，同时`write_fmt`还是一个字符一个字符输出的，就导致多个核可以同时write，使得字符混乱。于是我们设置一个全局`STDOUT`实例，同时为保证同步互斥加上锁`spin::Mutex`：

```rust
static STDOUT: Mutex<Stdout> = Mutex::new(Stdout);

pub fn print(args: fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
    //Stdout.write_fmt(args).unwrap();
}
```

这样就保证了写操作的互斥。最终我们有着正常的多核输出：

```
[rustsbi] Implementation: RustSBI-QEMU Version 0.0.2
[rustsbi-dtb] Hart count: cluster0 with 4 cores
[rustsbi] misa: RV64ACDFIMSU
[rustsbi] mideleg: ssoft, stimer, sext (0x222)
[rustsbi] medeleg: ima, ia, bkpt, la, sa, uecall, ipage, lpage, spage (0xb1ab)
[rustsbi] pmp0: 0x10000000 ..= 0x10001fff (rwx)
[rustsbi] pmp1: 0x80000000 ..= 0x8fffffff (rwx)
[rustsbi] pmp2: 0x0 ..= 0xffffffffffffff (---)
qemu-system-riscv64: clint: invalid write: 00000010
[rustsbi] enter supervisor 0x80200000
I am FIRST CPU 2
Hello, world!
.text [0x80200000, 0x80202000)
.rodata [0x80202000, 0x80203000)
.data [0x80203000, 0x80204000)
boot_stack [0x80204000, 0x80244000)
.bss [0x80244000, 0x80245000)
Hello world from CPU 2!
Hello world from CPU 1!
Hello world from CPU 3!
Hello world from CPU 0!
[kernel] Panicked at src/main.rs:80 Shutdown machine!
```

