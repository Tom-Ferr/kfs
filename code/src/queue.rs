use crate::malloc::kfree;

pub trait Queuable{
    type Ptr;
    fn get_next(&self) -> *const Self::Ptr;
    fn set_next(&mut self, value: *const Self::Ptr);
}

#[derive(Copy, Clone)]
pub struct Queue<T: Queuable>{
    head: Option<*const T>,
    tail: Option<*const T>,
}

impl<T: Queuable<Ptr=T>> Queue<T>{
    pub const fn new() -> Self{
        Self{
            head: None,
            tail: None,
        }
    }

    pub fn insert(&mut self, new: *mut T){
        if self.head.is_none(){
            self.head = Some(new);
            self.tail = self.head;
            unsafe{(*new).set_next(new)};
        }
        else{
            unsafe{
                let tail = *self.tail.as_mut().unwrap() as *mut T;
                let head = *self.head.as_ref().unwrap();
                
                (*new).set_next(head);
                (*tail).set_next(new);
                self.tail = Some(new);
            }
        }   
    }

    pub fn remove(&mut self){
        if !self.head.is_none(){
            unsafe{
                let head = *self.head.as_ref().unwrap();
                let tail = *self.tail.as_mut().unwrap() as *mut T;
                if head == tail{
                    self.head = None;
                    self.tail = None;
                }
                else{
                    (*tail).set_next((*head).get_next());
                    self.head = Some((*head).get_next());
                }
            }
        }
    }

    pub fn roll(&mut self){
        if !self.head.is_none(){
            let head = *self.head.as_mut().unwrap() as *mut T;
            self.remove();
            self.insert(head);
        }
    }

    pub fn get(&self) -> Option<*const T>{
        self.head
    }

    pub fn clean(&mut self){
        while let Some(head) = self.get(){
            self.remove();
            kfree(head as u32);
        }
    }
}