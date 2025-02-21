use crate::malloc::{kmalloc, kfree};
use crate::paging::*;
use crate::queue::*;
use crate::signals::*;
use core::arch::asm;

type ReadyQueue = Queue<ProcessControlBlock>;

const MAX_THREAD: usize = 5;
pub const NUMBER_OF_QUEUES: usize = 3;

static mut PID: u32 = 0;

pub static mut QUEUES: [ReadyQueue; NUMBER_OF_QUEUES] = [ReadyQueue::new(), ReadyQueue::new(), ReadyQueue::new()];

pub static mut CURRENT_PROC: Option<*const ProcessControlBlock> = None;
pub static mut CURRENT_TASK: *mut Option<*const ProcessControlBlock> = unsafe{&mut CURRENT_PROC as *mut Option<*const ProcessControlBlock>};
// pub static mut CURRENT_THREAD: Thread = Thread::new();

extern "C" {
    fn switch_to_user_mode(esp: u32, eip: u32);
}

#[derive(Copy, Clone)]
enum ProcStatus{
    Unused,
    Zombie,
    Embryo,
    Sleeping,
    Runnable,
    Running,
}

pub struct ProcessControlBlock {
    pid: u32,
    priority: usize,
    dir: *mut PageDirectory,
    state: ProcStatus,
    next: *const ProcessControlBlock,
    parent: *const ProcessControlBlock,
    child: *const ProcessControlBlock,
    code_text: u32,
    code_data: u32,
    code_bss: u32,
    code_size: u32,
    pendng: SignalQueue,
    blocked: SignalQueue,
    owner: u32,
    heap: u32,
    brk: u32,
    stack_begin: u32,
    stack_limit: u32,
    exit_code: u32,
    // thread_count: u32,
    // threadList: *const Thread,
    // threads: [Thread; MAX_THREAD],
    esp: u32,
    ss:  u32,
    kernel_esp: u32,
    kernel_ss: u32,
}

impl ProcessControlBlock {

    pub fn new(code_init: u32, code_size: u32) -> Option<*mut Self> {
        if let Some(addr) = kmalloc(size_of::<ProcessControlBlock>()) {
            let ptr = addr as *mut Self;
            unsafe{

                if let Ok(..) = (*ptr).init(code_init, code_size){
                    return Some(ptr);
                }
            }
        }
        None
    }

    fn init(&mut self, code_init: u32, code_size: u32) -> Result<(),()> {
        let stack_size = 0x1000;
        let stack_npage = ((stack_size + FRAME_SIZE - 1) / FRAME_SIZE) as u32;
        let dir_data = FRAME_SIZE;
        let stack_frame = FRAME_SIZE;
        let heap_page = FRAME_SIZE;
        let heap_frame = FRAME_SIZE;
        const array_size: usize = 6;
        let bytes_array: [u32; array_size-1] = [dir_data, (stack_npage * (size_of::<PageTable>() as u32)), stack_frame, heap_page, heap_frame];
        let mut ptr_array: [u32; array_size] = [0;array_size];

        if let Some(addr) = kmalloc(size_of::<PageDirectory>()){
            ptr_array[0] = addr;
        }
        else{
            return Err(());
        }

        for i in 1..array_size {
            if let Some( mut addr) = alloc_page(bytes_array[i-1] as usize) {
                ptr_array[i] = addr;
            }
            else{
                return Err(());
            }
        }
        unsafe{

            let dir_ptr = ptr_array[0] as *mut PageDirectory;
            (*dir_ptr).init(ptr_array[1]);

            let stack = ptr_array[2];
            let heap = ptr_array[4];
            
            if let Err(..) = map_code(dir_ptr, code_init, code_size){
                return Err(());
            }

            let tb = stack as *mut PageTable;
            (*tb).set_frame(1023, ptr_array[3] - 0xC0000000, 0x7);
            (*dir_ptr).set_page(UserSpace::Kernel as usize - 1, stack, 0x7);

            let code_npage = ((code_size + FRAME_SIZE - 1) / FRAME_SIZE) as usize;

            let hp = heap as *mut PageTable;
            (*hp).set_frame(0, ptr_array[5] - 0xC0000000, 0x7);
            (*dir_ptr).set_page(code_npage, heap, 0x7);

            PID += 1;
            self.pid = PID;
            self.priority = 1;
            self.dir = dir_ptr;
            self.state = ProcStatus::Runnable;
            self.code_text = 0x00000000;
            self.code_size = code_size;
            self.heap = (code_npage as u32) << 22;
            self.brk = self.heap + FRAME_SIZE;
            self.stack_begin = 0xBFFFFFFC;
            self.stack_limit = 0xC0000000 - FRAME_SIZE;
            // let mut th = Thread::new();
            // th.parent = self;
            // self.threads[0] = th;
            // self.thread_count = 1;
            
        }
        Ok(())
    }

    pub fn get_dir(&self) -> usize {
        unsafe{(*self.dir).get_directory()}
    }

    pub fn set_space(&mut self, u: UserSpace){
        unsafe{

            (*self.dir).set_whoami(u);
        }
    }

    pub fn set_esp(&mut self, esp: u32){
        self.esp = esp;
    }

    pub fn set_parent(&mut self, parent: *const ProcessControlBlock){
        self.parent = parent;
    }

    pub fn set_child(&mut self, child: *const ProcessControlBlock){
        self.child = child;
    }

    pub fn get_esp(&self) -> u32{
        self.esp
    }

    pub fn get_text(&self) -> u32{
        self.code_text
    }

    pub fn get_size(&self) -> u32{
        self.code_size
    }

    pub fn get_priority(&self) -> usize{
        self.priority
    }

    pub fn get_pid(&self) -> u32{
        self.pid
    }

    pub fn clean(&self) {
        clean(self.code_text, self.heap);
        clean(self.stack_limit, 0xC0000000);
        clean(self.heap, self.brk);
        unsafe {free_page((*self.dir).get_directory() as u32)};
        kfree(self.dir as u32);
    }

    pub fn fclean(&self) {
        self.clean();
        kfree(self as *const Self as u32);
    }

}

impl Drop for ProcessControlBlock {
    fn drop(&mut self) {
        self.fclean();
    }
}

impl Queuable for ProcessControlBlock {
    type Ptr = ProcessControlBlock;
    fn get_next(&self) -> *const ProcessControlBlock{
        self.next
    }
    fn set_next(&mut self, value: *const ProcessControlBlock){
        self.next = value;
    }
}

// #[derive(Copy, Clone)]
// struct Thread {
//     esp: u32,
//     ss:  u32,
//     kernel_esp: u32,
//     kernel_ss: u32,
//     parent: *const ProcessControlBlock,
//     priority: u32,
//     state: ProcStatus,
// }

// impl Thread {
//     fn new() -> Self {
//         Self{
//             esp: 0,
//             ss: 0,
//             kernel_esp: 0,
//             kernel_ss: 0,
//             parent: 0x0 as *const ProcessControlBlock,
//             priority: 1,
//             state: ProcStatus::Runnable,
//         }
//     }
// }

// #[derive(Default, Copy, Clone)]
// struct TrapFrame {
//     eax: u32,
//     ebx: u32,
//     ecx: u32,
//     edx: u32,
//     esi: u32,
//     edi: u32,
//     esp: u32,
//     ebp: u32,
//     eip: u32,
//     cs: u32,
//     flags: u32,
// }

pub unsafe fn fork() -> Option<u32>{
    let parent_proc = *CURRENT_PROC.as_mut().unwrap() as *mut ProcessControlBlock;
    let start = (*parent_proc).get_text();
    let size = (*parent_proc).get_size();
    let parent_pid = (*parent_proc).get_pid();

    if let Some(child_proc) = ProcessControlBlock::new(start, size){
        unsafe{
            (*child_proc).set_parent(parent_proc);
            (*parent_proc).set_child(child_proc);

           //copy stack
           //copy registers
           QUEUES[(*child_proc).get_priority()].insert(child_proc);
        }
        let current_proc = *CURRENT_PROC.as_ref().unwrap();
        if (*current_proc).get_pid() == parent_pid {
            return Some((*child_proc).get_pid());
        }
        return Some(0);
    }
    None
}

pub fn exec_fn(start: u32, func: u32, size: u32) -> Result<(),()> {
    if let Some(my_proc) = ProcessControlBlock::new(start, size){
        unsafe{

            let dir = (*my_proc).get_dir() - 0xC0000000;
            asm!("mov cr3, {}", in(reg) dir);

            CURRENT_PROC = Some(my_proc);
            QUEUES[(*my_proc).get_priority()].insert(my_proc);
            
            switch_to_user_mode(0xBFFFFFFC, func - 0xc0000000);
        }
    }
    else{
        return Err(());
    }
    Ok(())
}