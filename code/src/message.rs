use crate::queue::*;
use crate::malloc::{kmalloc, kfree};

pub type MessageQueue = Queue<Message>;

pub struct Message{
    text: *const u8,
    lenght: usize,
    next: *const Message
}

#[allow(dead_code)]
impl Message{
    pub fn new(text: &[u8]) -> Option<*const Self>{
        unsafe{

            if let Some(addr) = kmalloc(size_of::<Self>()){
                let slice_ptr = text.as_ptr();
                let ptr = addr as *mut Self;
                (*ptr).lenght = crate::utils::strlen(slice_ptr);
                if let Some(alloc) = kmalloc((*ptr).lenght){
                    (*ptr).text = alloc as *const u8;
                    crate::utils::memcpy(slice_ptr as *mut u8, (*ptr).text as *mut u8, (*ptr).lenght as u32);
                    return Some(ptr);
                }
            }
        }
        None
    }

    pub fn get_text(&self) -> *const u8 {
        self.text
    }

    pub fn get_length(&self) -> usize {
        self.lenght
    }
}

impl Drop for Message{
    fn drop(&mut self){
        kfree(self.text as u32);
        kfree(self as *const Self as u32);
    }
}

impl Queuable for Message {
    type Ptr = Message;
    fn get_next(&self) -> *const Message{
        self.next
    }
    fn set_next(&mut self, value: *const Message){
        self.next = value;
    }
}
