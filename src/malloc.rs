use core::mem::size_of;

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
static mut FREEP: Option<*mut Header> = None;

pub unsafe fn kmalloc(nbytes: usize) -> Option<u32>{
    unsafe{
        let nunits: usize = ((nbytes + size_of::<Header>() - 1) / size_of::<Header>()) + 1;
        
        if FREEP == None{
            BASE.next = Some(&mut BASE as *mut Header);
            FREEP = BASE.next;
        }

        let mut tmp: Option<*mut Header> = FREEP;
        let mut prev: *mut Header = tmp.unwrap();
        let mut curr: *mut Header = (*prev).get_next();
        loop{
            if (*curr).size >= nunits{
                if (*curr).size == nunits{
                    (*prev).next = (*curr).next;
                }
                else{
                    (*curr).size -= nunits;
                    curr.wrapping_add((*curr).size);
                    (*curr).size = nunits;
                }
                FREEP = Some(prev);
                break;
            }
            if curr == *(FREEP.as_ref().unwrap()) {
                let addr: Option<*mut Header> = morecore(nunits);
                if addr == None{
                    return None;
                }
                curr = addr.unwrap();
            }
            prev = curr;
            curr = (*curr).get_next();
        }
        Some(curr.wrapping_add(1) as u32)
    }
}

pub fn morecore(mut nunits: usize) -> Option<*mut Header>{

    if nunits < NALLOC {
        nunits = NALLOC;
    }
    unsafe{

        if let Some(addr) = crate::paging::alloc_page(){
            let req = addr as *mut Header;
            (*req).size = nunits;
            kfree((req.wrapping_add(1)) as u32);
            return Some(*(FREEP.as_mut().unwrap()));
        }
    }
    None
}

pub fn kfree(addr: u32) {
    unsafe{    
        if FREEP == None{
            BASE.next = Some(&mut BASE as *mut Header);
            FREEP = BASE.next;
        }

        let target: *mut Header = (addr as *mut Header).wrapping_sub(1);
        let mut free_list: *mut Header = *(FREEP.as_mut().unwrap());

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
        FREEP = Some(free_list);
    }
}

pub fn init_freelist(addr: u32, length: u32){
    unsafe{
        let target: *mut Header = addr as *mut Header;
        (*target).size = length as usize / size_of::<Header>();
        kfree(target.wrapping_add(1) as u32);
    }
}
