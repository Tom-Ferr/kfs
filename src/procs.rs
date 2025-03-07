use crate::malloc::{kmalloc, kfree};
use crate::paging::*;
use crate::queue::*;
use crate::signals::*;
use core::arch::asm;
use crate::idt::IntReg;
use crate::get_reg;

type ReadyQueue = Queue<ProcessControlBlock>;
type ChildQueue = Queue<Child>;

const MAX_THREAD: usize = 5;
const MAX_PROCS: usize = 63;

pub static mut MY_PROCS: [u32; MAX_PROCS] = [0; MAX_PROCS];

pub const NUMBER_OF_QUEUES: usize = 3;

static mut PID: u32 = 0;

pub static mut QUEUES: [ReadyQueue; NUMBER_OF_QUEUES] = [ReadyQueue::new(), ReadyQueue::new(), ReadyQueue::new()];

pub static mut CURRENT_PROC: Option<*const ProcessControlBlock> = None;
pub static mut CURRENT_TASK: *mut Option<*const ProcessControlBlock> = unsafe{&mut CURRENT_PROC as *mut Option<*const ProcessControlBlock>};
// pub static mut CURRENT_THREAD: Thread = Thread::new();

extern "C" {
    fn switch_to_user_mode(esp: u32, eip: u32);
}

#[derive(Copy, Clone, PartialEq)]
pub enum ProcStatus{
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
    children: Queue<Child>,
    simbling: *const Child,
    code_text: u32,
    code_data: u32,
    code_bss: u32,
    code_size: u32,
    pending: SignalQueue,
    blocked: SignalQueue,
    sig_handlers: [u32; 31],
    owner: u32,
    heap: u32,
    brk: u32,
    stack_begin: u32,
    stack_limit: u32,
    kernel_stack_begin: u32,
    kernel_stack_limit: u32,
    exit_code: i32,
    regs: IntReg,
    // thread_count: u32,
    // threadList: *const Thread,
    // threads: [Thread; MAX_THREAD],
    ss:  u32,
    kernel_ss: u32,
}

impl ProcessControlBlock {

    const ARRAY_SIZE: usize = 8;
    const ARRAY_OFFSET: usize = 1;

    pub fn new(code_init: u32, code_size: u32) -> Option<*mut Self> {
        if let Some(addr) = kmalloc(size_of::<ProcessControlBlock>()) {
            let ptr = addr as *mut Self;
            unsafe{
                
                if let Err(result) = (*ptr).init(code_init, code_size){
                    if result > 0{
                        kfree((*ptr).dir as u32);
                    }
                    if result > 1{
                        free_page((*ptr).get_cr3() as u32);
                    }
                    if result > 2{
                        (*ptr).clean_memory((*ptr).kernel_stack_limit, (*ptr).kernel_stack_begin + 4);
                    }
                    (*ptr).pending.clean();
                    (*ptr).blocked.clean();
                    (*ptr).clean_memory((*ptr).code_text, (*ptr).heap);
                    (*ptr).clean_memory((*ptr).stack_limit, (*ptr).stack_begin + 4);
                    (*ptr).clean_memory((*ptr).heap, (*ptr).brk);
                    kfree(addr);
                }
                else{
                    if let Ok((..)) = Self::register(addr){
                        return Some(ptr);
                    }
                }
            }
        }
        None
    }

    pub fn clone(src: &Self) -> Option<*mut Self> {
        if let Some(addr) = kmalloc(size_of::<ProcessControlBlock>()) {
            let ptr = addr as *mut Self;
            unsafe{

                if let Err(result) = (*ptr).import(src){
                    if result > 0{
                        kfree((*ptr).dir as u32);
                    }
                    if result > 1{
                        free_page((*ptr).get_cr3() as u32);
                    }
                    if result > 2{
                        (*ptr).clean_memory((*ptr).kernel_stack_limit, (*ptr).kernel_stack_begin + 4);
                    }
                    (*ptr).pending.clean();
                    (*ptr).blocked.clean();
                    (*ptr).clean_memory((*ptr).code_text, (*ptr).heap);
                    (*ptr).clean_memory((*ptr).stack_limit, (*ptr).stack_begin + 4);
                    (*ptr).clean_memory((*ptr).heap, (*ptr).brk);
                    kfree(addr);
                }
                else{
                    if let Ok((..)) = Self::register(addr){
                        return Some(ptr);
                    }
                }
            }
        }
        None
    }

    fn init_dir(&self, dir_data: u32, code_init: u32, code_size: u32) -> Result<(),u8>{
        unsafe{
            (*self.dir).init(dir_data);
            
            if let Err(..) = self.map_code(code_init, code_size){
                return Err(3);
                }
        }
        Ok(())
        
    }

    fn init(&mut self, code_init: u32, code_size: u32) -> Result<(),u8> {
        let stack_size = FRAME_SIZE * 4;
        let code_npage = ((code_size + PAGE_SIZE - 1) / PAGE_SIZE) as usize;

        self.code_text = 0x00000000;
        self.heap = (code_npage as u32) << 22;
        self.brk = self.heap + FRAME_SIZE;
        self.stack_begin = 0xC0000000 - 4;
        self.stack_limit = 0xC0000000 - stack_size;

        if let Some(addr) = kmalloc(size_of::<PageDirectory>()){
            unsafe{
                
                let dir_ptr = addr as *mut PageDirectory;
                self.dir = dir_ptr;

                let mut dir_data;
                if let Some(data) = alloc_page(FRAME_SIZE as usize) {
                    dir_data = data;
                }
                else{
                    return Err(1);
                }

                if let Some(stack) = alloc_page(stack_size as usize){
                    self.kernel_stack_limit = stack;
                    self.kernel_stack_begin = stack + stack_size - 4;
                }
                else{
                    return Err(2);
                }
                
                self.init_dir(dir_data, code_init, code_size)?;

                self.map_memory(self.stack_limit, self.stack_begin + 4, 0x7, false)?;
                self.map_memory(self.heap, self.brk, 0x7, false)?;
                
                PID += 1;
                self.pid = PID;
                self.priority = 1;
                self.state = ProcStatus::Embryo;
                self.code_size = code_size;
                self.sig_handlers = DEFAULT_SIG_HANDLERS;
                self.pending = SignalQueue::new();
                self.blocked = SignalQueue::new();
                self.children = ChildQueue::new();
                self.owner = 42;
                self.ss = 0x33;
                self.kernel_ss = 0x18;
                // let mut th = Thread::new();
                // th.parent = self;
                // self.threads[0] = th;
                // self.thread_count = 1;
                
            }
        }
        else{
            return Err(0);
        }
        Ok(())
    }

    fn import(&mut self, src: &Self) -> Result<(),u8> {
        
        let stack_size = (src.stack_begin + 4) - src.stack_limit;
        self.stack_begin = src.stack_begin;
        self.stack_limit = src.stack_limit;
        self.code_text = src.code_text;
        self.heap = src.heap;
        self.brk = src.brk;
        self.regs = src.regs;

        if let Some(addr) = kmalloc(size_of::<PageDirectory>()){
            unsafe{
                
                let dir_ptr = addr as *mut PageDirectory;
                self.dir = dir_ptr;

                self.copy_queue(&mut (*(src as *const Self as *mut Self)).pending)?;

                if let Some(stack) = alloc_page(stack_size as usize){
                    self.kernel_stack_limit = stack;
                    self.kernel_stack_begin = stack + stack_size - 4;
                }
                else{
                    return Err(1);
                }

                for i in (0..stack_size).step_by(4){
                    *((self.kernel_stack_limit + i) as *mut u32) = *((src.kernel_stack_limit + i) as *mut u32);
                }
                

                let mut dir_data;
                if let Some(data) = alloc_page(FRAME_SIZE as usize) {
                    dir_data = data;
                }
                else{
                    return Err(1);
                }
                
                self.init_dir(dir_data, src.code_text + 0xC0000000, src.code_size)?;

                self.map_memory(src.stack_limit, src.stack_begin + 4, 0x7, true)?;
                self.map_memory(src.heap, src.brk, 0x7, true)?;
                
                PID += 1;
                self.pid = PID;
                self.state = ProcStatus::Embryo;
                self.priority = src.priority;
                self.code_size = src.code_size;
                self.sig_handlers = src.sig_handlers;
                self.pending = SignalQueue::new();
                self.blocked = SignalQueue::new();
                self.children = ChildQueue::new();
                self.owner = src.owner;
                self.ss = 0x33;
                self.kernel_ss = 0x18;
                // self.kernel_stack = kernel_stack;
            }
        }
        else{
            return Err(0);
        }
        Ok(())
    }
}

impl ProcessControlBlock{

    pub fn map_code(&self, src: u32, size: u32) -> Result<(),()> {
        let mut vaddr = src & 0xFFFFF000;
        let code_npage = ((size + PAGE_SIZE - 1) / PAGE_SIZE) as u32;
        let code_end = src + size;
        
        for dir_index in 0..code_npage
        {
            unsafe{
                
                if let Some(page) = alloc_page(FRAME_SIZE as usize) {
                    (*self.dir).set_page(dir_index as usize, page, 0x5);
                    let table = page as *mut PageTable;
                    for table_index in 0..1024
                    {
                        (*table).set_frame(table_index as usize, vaddr - 0xC0000000, 0x5);
                        vaddr += FRAME_SIZE;
                        if vaddr >= code_end{
                            break;
                        }
                    }
                }
                else{
                    return Err(());
                }
            }
        }
        Ok(())
    }

    fn map_memory(&self, start: u32, end: u32, flags: u32, copy: bool) -> Result<(), u8>{
        let mut curr = start;
        let mut start_index: usize = (start >> 22) as usize;
        let mut end_index: usize = (end >> 22) as usize;

        if start_index == end_index{
            end_index += 1;
        }
    
        unsafe{
            let dir = &mut *self.dir;
            while start_index < end_index {
                if let Some(page) = alloc_page(FRAME_SIZE as usize){
    
                    dir.set_page(start_index, page, flags);
                    start_index += 1;
                }
                else{
                    return Err(3);
                }
            }
            
            while curr < end{
                
                if let Some(frame) = alloc_page(FRAME_SIZE as usize){
                    let dir_index = curr >> 22;
                    let tab_index = (curr >> 12) & 0x3FF;
                    let table = dir.get_page(dir_index as usize) as *mut PageTable;
                    
                    (*table).set_frame(tab_index as usize, frame - 0xC0000000, flags);
                    if copy == true{
                        for i in (0..FRAME_SIZE).step_by(4){
                            let target = (curr + i) as *const u32;
                            *((frame + i) as *mut u32) = *target;
                        }
                    }
                    curr += FRAME_SIZE;
                }
                else{
                    return Err(3);
                }
            }
        }
        Ok(())
    }

    fn  clean_memory(&self, start: u32, end: u32){
        let mut curr = start;
        let mut start_index: usize = (start >> 22) as usize;
        let end_index: usize = (end >> 22) as usize;
        while curr < end{
            let target = get_physical_addr(curr as usize) + 0xC0000000;
            free_page(target);
            curr += FRAME_SIZE;
        }
        unsafe{
            let dir = &mut *DIR;
            while start_index < end_index {
                let target = dir.get_page(start_index);
                free_page(target);
                start_index += 1;
            }
        }
    }

    fn copy_queue(&mut self, src: &mut SignalQueue) -> Result<(),u8>{
        unsafe{
            if let Some(head) = src.get() {
                let mut curr: Option<*const Signal> = None;
                let mut ptr = head;
                while curr != Some(head) {
                    let sig = (*ptr).get_signal();
                    if let Some(signal) = Signal::new(sig){
                        self.recv_sig(signal);
                    }
                    else{
                        return Err(1);
                    }
                    src.roll();
                    curr = src.get();
                    ptr = *curr.as_mut().unwrap();
                }
            }
        }
        Ok(())
    }
}

impl ProcessControlBlock {
    pub fn load(kernel_start: u32, kernel_end: u32) -> Option<*mut Self> {
        if let Some(addr) = kmalloc(size_of::<ProcessControlBlock>()) {
            let ptr = addr as *mut Self;
            unsafe{
                
                (*ptr).set(kernel_start, kernel_end);

                if let Ok((..)) = Self::register(addr){
                   return Some(ptr);
                }
            }
        }
        None
    }

    unsafe fn set(&mut self, kernel_start: u32, kernel_end: u32) {

        self.code_text = kernel_start;
        self.kernel_stack_begin = get_reg!(ebp) as u32 -4;
        self.kernel_stack_limit = self.kernel_stack_begin - (4 * FRAME_SIZE);
        self.stack_begin = self.kernel_stack_begin;
        self.stack_limit = self.kernel_stack_limit;
        self.dir = DIR;
   
        PID += 1;
        self.pid = PID;
        self.priority = 1;
        self.state = ProcStatus::Running;
        self.code_size = kernel_end - kernel_start;
        self.sig_handlers = DEFAULT_SIG_HANDLERS;
        self.pending = SignalQueue::new();
        self.blocked = SignalQueue::new();
        self.children = ChildQueue::new();
        self.owner = 42;
        self.ss = 0x33;
        self.kernel_ss = 0x18;
    }
}

impl ProcessControlBlock{
    pub fn get_pid(&self) -> u32{
        self.pid
    }

    pub fn get_priority(&self) -> usize{
        self.priority
    }

    pub fn get_dir(&self) -> &mut PageDirectory {
        unsafe{
            &mut (*self.dir)
        }
    }

    pub fn get_state(&self) -> ProcStatus{
        self.state
    }

    pub fn get_parent(&self) -> *const ProcessControlBlock{
        self.parent
    }

    pub fn get_text(&self) -> u32{
        self.code_text
    }

    pub fn get_code_size(&self) -> u32{
        self.code_size
    }

    pub fn get_heap(&self) -> usize {
        self.heap as usize
    }

    pub fn get_owner(&self) -> u32 {
        self.owner
    }

    pub fn get_brk(&self) -> usize {
        self.brk as usize
    }

    pub fn get_exit_code(&self) -> i32{
        self.exit_code
    }

    pub fn get_cr3(&self) -> usize {
        unsafe{(*self.dir).get_directory()}
    }

    
    pub fn get_esp(&self) -> u32{
        self.stack_begin
    }

    pub fn get_kernel_esp(&self) -> u32{
        self.kernel_stack_begin
    }

    pub fn get_user_esp(&self) -> u32{
        self.stack_begin
    }

    pub fn get_kernel_ss(&self) -> u32{
        self.kernel_ss
    }


    pub fn get_regs(&self) -> IntReg{
        self.regs
    }
    
    pub fn get_signal(&self) -> Option<Sig> {
        if let Some(pending_signal) = self.pending.get(){
            unsafe{
                let signal = (*pending_signal).get_signal();
                return Some(signal);
            } 
        }
        None
    }
    
    pub fn get_handler(&self, sig: Sig) -> u32{
        self.sig_handlers[sig as usize - 1]
    }

    fn get_stack_size(&self) -> u32 {
        self.stack_begin + 4 - self.stack_limit
    }

    fn get_heap_size(&self) -> u32 {
        self.brk - self.heap
    }

    pub fn set_exit_code(&mut self, value: i32){
        self.exit_code = value;
    }

    pub fn set_state(&mut self, new_state: ProcStatus){
        self.state = new_state;
    }

    pub fn set_regs(&mut self, other: IntReg) {
        self.regs = other;
    }

    pub fn set_parent(&mut self, parent: *const ProcessControlBlock){
        self.parent = parent;
    }

    pub fn add_handler(&mut self, sig: Sig, handler: u32){
        self.sig_handlers[sig as usize - 1] = handler;
    }

    pub fn recv_sig(&mut self, signal: *const Signal){
        self.pending.insert(signal as *mut Signal);
    }

    pub fn export_ebp(&self){
        unsafe{
            asm!("mov ebp, {}", in(reg)self.kernel_stack_begin + 4);
        }
    }

    pub fn clean(&mut self) {
        self.pending.clean();
        self.blocked.clean();
        self.clean_memory(self.code_text, self.heap);
        self.clean_memory(self.stack_limit, self.stack_begin + 4);
        self.clean_memory(self.kernel_stack_limit, self.kernel_stack_begin + 4);
        self.clean_memory(self.heap, self.brk);
        free_page(self.get_cr3() as u32);
        kfree(self.dir as u32);
    }

    pub fn fclean(&mut self) {
        self.clean();
        Self::free(self as *const Self as u32);
    }
    
    pub fn free(ptr: u32){
        kfree(ptr);
    }

    pub fn register(ptr: u32) -> Result<(),()>{
        unsafe{
            for i in 0..MAX_PROCS{
                if MY_PROCS[i] == 0 {
                    MY_PROCS[i] = ptr;
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn unregister(ptr: u32){
        unsafe{
            for i in 0..MAX_PROCS{
                if MY_PROCS[i] == ptr {
                    MY_PROCS[i] = 0;
                    break;
                }
            }
        }
    }
}

impl Drop for ProcessControlBlock {
    fn drop(&mut self) {
        self.fclean();
    }
}

impl ProcessControlBlock {
    pub fn change_process(&mut self) {
        unsafe{
            CURRENT_PROC = Some(self as *const Self);
            DIR = self.dir as *mut PageDirectory;
            self.state = ProcStatus::Running;
        }
    }

    pub fn change_context(&self) {
        unsafe{
            let dir = self.get_cr3() - 0xC0000000;
            asm!("mov cr3, {}", in(reg) dir);
        }
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

impl ProcessControlBlock{

    pub fn add_child(&mut self, child: *const ProcessControlBlock) {
        self.children.insert(child as *mut Child);
    }

    pub fn get_child(&self) -> Option<*const ProcessControlBlock>{

        if let Some(head) = self.children.get() {
            return Some(head as *const ProcessControlBlock);
        }
        None
    }

    pub fn get_next_child(&mut self) -> Option<*const ProcessControlBlock>{

        self.children.roll();

        self.get_child()
    }

    pub fn remove_child(&mut self) {
        self.children.remove();
    }

}

struct Child(ProcessControlBlock);

impl Queuable for Child {
    type Ptr = Child;
    fn get_next(&self) -> *const Child{
        self.0.simbling
    }
    fn set_next(&mut self, value: *const Child){
        self.0.simbling = value;
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

pub unsafe fn sys_wait(status: &mut i32) -> Option<u32> {
    let parent = *CURRENT_PROC.as_mut().unwrap() as *mut ProcessControlBlock;

    loop {
        if let Some(head) = (*parent).get_child() {
            let mut curr: Option<*const ProcessControlBlock> = None;
            let mut child = head;
            while curr != Some(head) {
                if (*child).get_state() == ProcStatus::Zombie {
                    *status = (*child).get_exit_code();
                    let pid = (*child).get_pid();
                    (*parent).remove_child();
                    ProcessControlBlock::free(child as u32);
                    return Some(pid);
                }
                curr = (*parent).get_next_child();
                child = *curr.as_ref().unwrap();
            }
        }
        else {
            return None;
        }

        (*parent).set_state(ProcStatus::Sleeping);
    }
}

pub unsafe fn sys_exit(status: i32) {
    let child = *CURRENT_PROC.as_mut().unwrap() as *mut ProcessControlBlock;
    let parent = (*child).get_parent() as *mut ProcessControlBlock;

        (*child).clean();
        
        (*child).set_exit_code(status);
        
        (*parent).set_state(ProcStatus::Runnable);
        
        (*child).set_state(ProcStatus::Zombie);

        let mut regs = (*parent).get_regs();
        crate::timer::switch_task(& mut regs);
}

pub unsafe fn sys_getuid() -> u32 {
        let proc = *CURRENT_PROC.as_mut().unwrap() as *mut ProcessControlBlock;
        (*proc).get_owner()
}

pub unsafe fn sys_kill(pid: u32, sig: Sig) -> Result<(),()>{
    for i in 0..MAX_PROCS{
        let ptr = MY_PROCS[i] as *mut ProcessControlBlock;
        if (*ptr).get_pid() == pid {
            if let Some(signal) = Signal::new(sig){
                (*ptr).recv_sig(signal);
                if (*ptr).get_state() == ProcStatus::Sleeping{
                    (*ptr).set_state(ProcStatus::Runnable);
                }
                return Ok(());
            }
        }
    }
    Err(())
}

pub unsafe fn sys_signal(sig: Sig, handler: u32) -> Option<u32>{
    let current_proc = *CURRENT_PROC.as_mut().unwrap() as *mut ProcessControlBlock;
    let ret = (*current_proc).get_handler(sig);
    (*current_proc).add_handler(sig, handler);
    Some(ret)
}

pub unsafe fn sys_fork() -> Option<u32>{
    let parent_proc = *CURRENT_PROC.as_mut().unwrap() as *mut ProcessControlBlock;
    let start = (*parent_proc).get_text();
    let size = (*parent_proc).get_code_size();
    let parent_pid = (*parent_proc).get_pid();

    if let Some(child_proc) = ProcessControlBlock::clone(&*parent_proc){
        unsafe{
            (*parent_proc).add_child(child_proc);
            (*child_proc).set_parent(parent_proc);
            QUEUES[(*child_proc).get_priority()].insert(child_proc);

            let mut regs = (*parent_proc).get_regs();
            let offset = (*parent_proc).get_kernel_esp() - regs.get_esp();
            regs.set_esp((*child_proc).get_kernel_esp() - offset);
            regs.set_ebp((*child_proc).get_kernel_esp() + 4);
            regs.set_eax(0);

            (*child_proc).set_regs(regs);
            (*child_proc).set_state(ProcStatus::Runnable);
        }
        return Some((*child_proc).get_pid());
    }
    None
}

pub fn exec_fn(start: u32, func: u32, size: u32) -> Result<(),()> {
    if let Some(new_proc) = ProcessControlBlock::new(start, size){
        unsafe{

            QUEUES[(*new_proc).get_priority()].insert(new_proc);
            (*new_proc).change_process();
            (*new_proc).change_context();
            
            switch_to_user_mode((*new_proc).get_user_esp(), func - 0xc0000000);
        }
    }
    else{
        return Err(());
    }
    Ok(())
}

pub unsafe fn load_process(kernel_start: u32, kernel_end: u32) -> Result<(),()> {
    if let Some(my_proc) = ProcessControlBlock::load(kernel_start, kernel_end){
        CURRENT_PROC = Some(my_proc);
        QUEUES[(*my_proc).get_priority()].insert(my_proc);
    }
    else{
        return Err(());
    }
    Ok(())
}