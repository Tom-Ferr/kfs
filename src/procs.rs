use crate::malloc::{kmalloc, kfree};
use crate::paging::*;
const MAX_THREAD: usize = 5;

#[derive(Copy, Clone)]
enum ProcStatus{
    Unused,
    Zombie,
    Embryo,
    Sleeping,
    Runnable,
    Running,
}

pub struct PocessControlBlock {
    pid: u32,
    priority: u32,
    dir: *mut PageDirectory,
    state: ProcStatus,
    // next: *const PocessControlBlock,
    // threadList: *const Thread,
    threads: [Thread; MAX_THREAD],
}

impl PocessControlBlock {

    pub fn new(code_init: u32, code_end: u32) -> Option<*const Self> {
        if let Some(addr) = kmalloc(size_of::<PocessControlBlock>()) {
            let ptr = addr as *mut Self;
            unsafe{

                if let Ok(..) = (*ptr).init(code_init, code_end){
                    return Some(ptr);
                }
            }
        }
        None
    }

    fn init(&mut self, code_init: u32, code_end: u32) -> Result<(),()> {
        let stack_size = 0x1000;
        let code_size = code_end - code_init;
        let code_npage = ((code_size + FRAME_SIZE - 1) / FRAME_SIZE) as u32;
        let stack_npage = ((stack_size + FRAME_SIZE - 1) / FRAME_SIZE) as u32;
        let stack_frame = FRAME_SIZE;
        let bytes_array: [u32; 4] = [size_of::<PageDirectory>() as u32, FRAME_SIZE, (stack_npage * (size_of::<PageTable>() as u32)), stack_frame];
        let mut ptr_array: [u32;4] = [0;4];

        for i in 0..4 {
            if let Some( mut addr) = alloc_page(bytes_array[i] as usize) {
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
            
            let mut f = code_init & 0xFFFFF000;
            for i in 0..code_npage
            {
                if let Some(p) = alloc_page(0x1000) {
                    (*dir_ptr).set_page(i as usize, p, 0x5);
                    let tb = p as *mut PageTable;
                    for i in 0..1024
                    {
                        (*tb).set_frame(i as usize, (f - 0xC0000000), 0x5);
                        f += FRAME_SIZE;
                        if f >= code_end{
                            break;
                        }
                    }
                }
                else{
                    return Err(());
                }
            }

            let tb = stack as *mut PageTable;
            (*tb).set_frame(1023, ptr_array[3] - 0xC0000000, 0x7);
            (*dir_ptr).set_page(UserSpace::Kernel as usize - 1, stack, 0x7);

            self.pid = 1;
            self.priority = 1;
            self.dir = dir_ptr;
            self.state = ProcStatus::Runnable;
            let mut th = Thread::new();
            th.parent = self;
            th.initialStack = stack as *const usize;
            self.threads[0] = th;
            
        }
        Ok(())
    }

    pub fn get_dir(&self) -> usize {
        unsafe{(*self.dir).get_directory()}
    }
}

impl Drop for PocessControlBlock {
    fn drop(&mut self) {
        // kfree(self.dir as u32);
        kfree(self as *const Self as u32);
    }
}

#[derive(Copy, Clone)]
struct Thread {
    parent: *const PocessControlBlock,
    initialStack: *const usize,
    stackLimit: *const usize,
    kernelStack: *const usize,
    priority: u32,
    state: ProcStatus,
    frame: TrapFrame,
}

impl Thread {
    fn new() -> Self {
        Self{
            parent: 0x0 as *const PocessControlBlock,
            initialStack: 0x0 as *const usize,
            stackLimit: 0x0 as *const usize,
            kernelStack: 0x0 as *const usize,
            priority: 1,
            state: ProcStatus::Runnable,
            frame: TrapFrame::default(),
        }
    }
}

#[derive(Default, Copy, Clone)]
struct TrapFrame {
    esp: u32,
    ebp: u32,
    eip: u32,
    edi: u32,
    esi: u32,
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
    flags: u32,
}