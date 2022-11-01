//use crate::bindings;

pub trait LlItem<T: Copy = Self>: Copy {
    type Output;
    fn get_next(&self) -> Option<Self::Output>;
    fn set_next(&mut self, elem: T);
}

impl <T: LlItem + LlItem<Output = T>>Iterator for LinkedList<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = &self.next {
            self.next = n.get_next();
        } else {
            self.next = Some(unsafe{*self.first});
        }
        self.next
    }
}

impl <T: LlItem>LinkedList<T> {
    pub fn new(f: T) -> LinkedList<T> {
        LinkedList { first: &f, next: None }
    }
    /*fn cleanup(self) {
        unsafe{bindings::pbs_statfree(self.first as *mut bindings::batch_status)}
    }*/
}

pub struct LinkedList<T: LlItem> {
    first: *const T,
    next: Option<T>
}

macro_rules! impl_LlItem {
    ([$($t:ty),+]) => {
        $(impl linked_list::LlItem for $t {
            type Output = $t;
            fn get_next(&self) -> Option<Self::Output> {
                if self.next == 0 as *mut Self::Output {
                    return None;
                }
                Some(unsafe{*self.next}) 
            }
            fn set_next(&mut self, mut elem: $t) {
                self.next = &mut elem;
            }
        })*
    }
}
pub(crate) use impl_LlItem;

