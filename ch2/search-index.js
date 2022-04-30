var searchIndex = JSON.parse('{\
"os":{"doc":"The main module and entrypoint","t":[0,5,0,0,14,14,5,0,0,0,0,17,3,17,3,7,17,3,17,7,17,3,12,12,11,11,11,11,11,11,11,11,12,12,12,11,11,11,11,11,11,11,11,5,11,11,11,11,11,11,12,5,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,5,11,11,11,11,5,17,17,5,5,5,0,3,11,11,11,11,12,11,11,11,11,11,17,17,0,0,5,17,5,5,3,0,5,12,12,5,12,3,11,11,11,11,11,12,11,12,11,11,11,12],"n":["batch","clear_bss","console","lang_items","print","println","rust_main","sbi","sync","syscall","trap","APP_BASE_ADDRESS","APP_MANAGER","APP_SIZE_LIMIT","AppManager","KERNEL_STACK","KERNEL_STACK_SIZE","KernelStack","MAX_APP_NUM","USER_STACK","USER_STACK_SIZE","UserStack","__private_field","app_start","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","current_app","data","data","deref","from","from","from","from","get_current_app","get_sp","get_sp","init","into","into","into","into","load_app","move_to_next_app","num_app","print_app_info","print_app_info","push_context","run_next_app","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","Stdout","borrow","borrow_mut","from","into","print","try_from","try_into","type_id","write_str","panic","SBI_CONSOLE_PUTCHAR","SBI_SHUTDOWN","console_putchar","sbi_call","shutdown","up","UPSafeCell","borrow","borrow_mut","exclusive_access","from","inner","into","new","try_from","try_into","type_id","SYSCALL_EXIT","SYSCALL_WRITE","fs","process","syscall","FD_STDOUT","sys_write","sys_exit","TrapContext","context","init","sepc","sstatus","trap_handler","x","TrapContext","app_init_context","borrow","borrow_mut","from","into","sepc","set_sp","sstatus","try_from","try_into","type_id","x"],"q":["os","","","","","","","","","","","os::batch","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","os::console","","","","","","","","","","os::lang_items","os::sbi","","","","","os::sync","os::sync::up","","","","","","","","","","","os::syscall","","","","","os::syscall::fs","","os::syscall::process","os::trap","","","","","","","os::trap::context","","","","","","","","","","","",""],"d":["batch subsystem","clear BSS segment","SBI console driver, for text output","The panic handler","print string macro","println string macro","the rust entry-point of os","SBI call wrappers","Synchronization and interior mutability primitives","Implementation of syscalls","Trap handling functionality","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","","","init batch subsystem","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","print apps info","","","run next app","","","","","","","","","","","","","","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","","","","","","use sbi call to putchar in console (qemu uart handler)","handle SBI call with <code>which</code> SBI_id and other arguments","use sbi call to getchar from console (qemu uart handler) …","Uniprocessor interior mutability primitives","Wrap a static data structure inside it so that we are able …","","","Exclusive access inner data in UPSafeCell. Panic if the …","Returns the argument unchanged.","inner data","Calls <code>U::from(self)</code>.","User is responsible to guarantee that inner struct is only …","","","","","","File and filesystem-related syscalls","App management syscalls","handle syscall exception with <code>syscall_id</code> and other …","","write buf of length <code>len</code>  to a file with <code>fd</code>","task exits and submit an exit code","Trap Context","","initialize CSR <code>stvec</code> as the entry of <code>__alltraps</code>","CSR sepc","CSR sstatus      ","handle an interrupt, exception, or system call from user …","general regs[0..31]","Trap Context","init app context","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","CSR sepc","set stack pointer to x_2 reg (sp)","CSR sstatus      ","","","","general regs[0..31]"],"i":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,3,4,2,1,3,4,2,1,2,3,4,1,3,4,2,1,2,3,4,0,3,4,2,1,2,2,2,0,2,3,0,3,4,2,1,3,4,2,1,3,4,2,1,0,5,5,5,5,0,5,5,5,5,0,0,0,0,0,0,0,0,6,6,6,6,6,6,6,6,6,6,0,0,0,0,0,0,0,0,0,0,0,7,7,0,7,0,7,7,7,7,7,7,7,7,7,7,7,7],"f":[null,[[]],null,null,null,null,[[],["never",0]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],null,null,null,[[["",0]],["upsafecell",3]],[[]],[[]],[[]],[[]],[[["",0]],["usize",0]],[[["",0]],["usize",0]],[[["",0]],["usize",0]],[[]],[[]],[[]],[[]],[[]],[[["",0],["usize",0]]],[[["",0]]],null,[[]],[[["",0]]],[[["",0],["trapcontext",3]],["trapcontext",3]],[[],["never",0]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],null,[[["",0]],["",0]],[[["",0]],["",0]],[[]],[[]],[[["arguments",3]]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0],["str",0]],["result",6]],[[["panicinfo",3]],["never",0]],null,null,[[["usize",0]]],[[["usize",0],["usize",0],["usize",0],["usize",0]],["usize",0]],[[],["never",0]],null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["refmut",3]],[[]],null,[[]],[[]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],null,null,null,null,[[["usize",0]],["isize",0]],null,[[["usize",0],["usize",0]],["isize",0]],[[["i32",0]],["never",0]],null,null,[[]],null,null,[[["trapcontext",3]],["trapcontext",3]],null,null,[[["usize",0],["usize",0]]],[[["",0]],["",0]],[[["",0]],["",0]],[[]],[[]],null,[[["",0],["usize",0]]],null,[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],null],"p":[[3,"APP_MANAGER"],[3,"AppManager"],[3,"KernelStack"],[3,"UserStack"],[3,"Stdout"],[3,"UPSafeCell"],[3,"TrapContext"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};