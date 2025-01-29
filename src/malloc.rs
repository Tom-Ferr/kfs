use core::mem::size_of;
use crate::paging::*;

const NALLOC: usize = 1024;

#[repr(C)]
struct Header {
    next: Option<*mut Header>,
    size: usize,
}

impl Header{
    const fn new() -> Self{
        Self{
            next: None,
            size: 0,
        }
    }

    fn get_next(&mut self) -> *mut Header{
        *(self.next.as_mut().unwrap())
    }
}

static mut BASE: Header = Header::new();
static mut PHYLS: Option<*mut Header> = None;
static mut VITLS: Option<*mut Header> = None;

fn check_virtual_space(nbytes: usize) -> Option<u32>{
    let mut nframes = nbytes / FRAME_SIZE as usize;
    let mut ntables = nframes / 1024;
    let mut candidate: Option<u32> = None;
    
    if nbytes % FRAME_SIZE as usize > 0{
        nframes += 1;
    }
    if nbytes % 1024 > 0{
        ntables += 1;
    }

    unsafe{     
        #[allow(static_mut_refs)]
        let dir = DIR.as_mut().unwrap();
        while ntables > dir.get_virtual_allocs(){
            if let Err(..) = dir.new_virtual_page(dir.get_virtual_allocs() + 1){
                return None;
            }
        }
        for offset in 0..=dir.get_virtual_allocs(){
            let tab = &mut *(dir.get_page(UserSpace::Virtual as usize + offset as usize) as *mut PageTable);
            for i in 0..1024 {
                let byte_index: usize = i / 32;
                let bit_index: usize = i % 32;

                let mut cursor = !0;
                let mut limit = 0;
                if nframes < 32{
                    limit = 32 - nframes;
                    cursor = cursor >> limit;
                }
                
                if bit_index > limit as usize{
                    continue;
                }
                
                let target = cursor << bit_index;
                if (tab.bitmap[byte_index] & target) == 0 {
                    if candidate == None{
                        candidate = Some(( (UserSpace::Virtual as usize + offset + dir.get_whoami()) << 22 | i << 10) as u32);
                    }
                    nframes -= 32 - limit;
                    if nframes == 0 {
                        return candidate;
                    }
                    continue ;
                }
                candidate = None;
                nframes = nbytes / FRAME_SIZE as usize;
            }
        }
    }
    None
}

#[allow(dead_code)]
fn vmap(nbytes: usize, vaddr: u32) -> Result<(),()> {

    let mut nframes = nbytes / FRAME_SIZE as usize;
    if nbytes % FRAME_SIZE as usize > 0{
        nframes += 1;
    }
    
    unsafe{
        let mut ptr = vaddr;
        #[allow(static_mut_refs)]
        let dir = DIR.as_mut().unwrap();
        for _ in 0..nframes{
            if let Some(addr) = alloc_page(FRAME_SIZE as usize){
                let tab_index = (ptr >> 12 & 0x3FF) as usize;
                let dir_index = (ptr >> 22) as usize;
                let tab = &mut *(dir.get_page(dir_index - dir.get_whoami()) as *mut PageTable);
                tab.set_frame(tab_index, addr - 0xC0000000);
                ptr += FRAME_SIZE;
            }
            else{
                let mut begin = vaddr;
                while begin != ptr{
                    let tab_index = (begin >> 12 & 0x3FF) as usize;
                    let dir_index = (begin >> 22) as usize;
                    let tab = &mut *(dir.get_page(dir_index - dir.get_whoami()) as *mut PageTable);
                    let frame = tab.get_frame(tab_index);
                    free_page(frame);
                    begin += FRAME_SIZE;
                }
                return Err(());
            }
        }
    }
    Ok(())
}

fn virtual_allocation(nbytes: usize) -> Option<u32>{
    if let Some(vaddr) = check_virtual_space(nbytes){
        if let Ok(..) = vmap(nbytes, vaddr){
            return Some(vaddr);
        }
    }
    None
}

fn morecore(mut nunits: usize, freep: *mut Option<*mut Header>, f: fn(usize) -> Option<u32>) -> Option<*mut Header>{

    if nunits < NALLOC {
        nunits = NALLOC;
    }
    unsafe{

        if let Some(addr) = f(nunits * size_of::<Header>()){
            let req = addr as *mut Header;
            (*req).size = nunits;
            kfree((req.wrapping_add(1)) as u32);
            return Some(*((*freep).as_mut().unwrap()));
        }
    }
    None
}

unsafe fn malloc_routine(nbytes: usize, freep: *mut Option<*mut Header>, f: fn(usize) -> Option<u32>) -> Option<u32>{
    unsafe{
        let nunits: usize = ((nbytes + size_of::<Header>() - 1) / size_of::<Header>()) + 1;
        
        if *freep == None{
            BASE.next = Some(&raw mut BASE as *mut Header);
            *freep = BASE.next;
        }

        let tmp: Option<*mut Header> = *freep;
        let mut prev: *mut Header = tmp.unwrap();
        let mut curr: *mut Header = (*prev).get_next();
        loop{
            if (*curr).size >= nunits{
                if (*curr).size == nunits{
                    (*prev).next = (*curr).next;
                }
                else{
                    (*curr).size -= nunits;
                    curr = curr.wrapping_add((*curr).size);
                    (*curr).size = nunits;
                }
                *freep = Some(prev);
                return Some(curr.wrapping_add(1) as u32);
            }
            if curr == *((*freep).as_ref().unwrap()) {
                let addr: Option<*mut Header> = morecore(nunits, freep, f);
                if addr == None{
                    return None;
                }
                curr = addr.unwrap();
            }
            prev = curr;
            curr = (*curr).get_next();
        }
    }
}

unsafe fn free_routine(addr: u32, freep: *mut Option<*mut Header>) {
    unsafe{    
        if *freep == None{
            BASE.next = Some(&raw mut BASE as *mut Header);
            *freep = BASE.next;
        }

        let target: *mut Header = (addr as *mut Header).wrapping_sub(1);
        let mut free_list: *mut Header = *((*freep).as_mut().unwrap());

        while !(target > free_list && target < (*free_list).get_next()){
            if free_list >= (*free_list).get_next() && (target > free_list || target < (*free_list).get_next()){
                break;
            }
            free_list = (*free_list).get_next();
        }
        if target.wrapping_add((*target).size) == (*free_list).get_next(){
            (*target).size += (*(*free_list).get_next()).size;
            (*target).next = (*(*free_list).get_next()).next;
        }
        else{
            (*target).next = (*free_list).next;
        }
        if free_list.wrapping_add((*free_list).size) == target{
            (*free_list).size += (*target).size;
            (*free_list).next = (*target).next;
        }
        else{
            (*free_list).next = Some(target);
        }
        *freep = Some(free_list);
    }
}

#[allow(dead_code)]
pub fn kmalloc(nbytes: usize) -> Option<u32>{
    if nbytes > 127 * 1024{
        return None;
    }
    unsafe{
        malloc_routine(nbytes, &raw mut PHYLS,alloc_page)
    }
}

#[allow(dead_code)]
pub fn vmalloc(nbytes: usize) -> Option<u32>{
    if nbytes > 0xA00000{
        return None;
    }
    unsafe{
        malloc_routine(nbytes, &raw mut VITLS, virtual_allocation)
    }
}

#[allow(dead_code)]
pub fn kfree(addr: u32){
    unsafe{
        free_routine(addr, &raw mut PHYLS);
    }
}

#[allow(dead_code)]
pub fn vfree(addr: u32){
    unsafe{
        free_routine(addr, &raw mut VITLS);
    }
}

#[allow(dead_code)]
pub fn ksize(addr: u32) -> usize{
    let target: *mut Header = (addr as *mut Header).wrapping_sub(1);
    unsafe {((*target).size * size_of::<Header>()) - size_of::<Header>()}
}

#[allow(dead_code)]
pub fn vsize(addr: u32) -> usize{
    let target: *mut Header = (addr as *mut Header).wrapping_sub(1);
    unsafe {((*target).size * size_of::<Header>()) - size_of::<Header>()}
}

pub fn init_freelist(kernel_end: u32){
    unsafe{
        block_pages(kernel_end, (NALLOC * size_of::<Header>()) as u32);
        let target = (kernel_end + 0xC0000000) as *mut Header;
        (*target).size = NALLOC;
        kfree(target.wrapping_add(1) as u32);
    }
}
