pub trait LlItem<T: Copy = Self>: Copy {
    type Output;
    fn get_next(self) -> Option<Self::Output>;
}

impl <T: LlItem + LlItem<Output = T>>Iterator for LinkedList<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = &self.next {
            self.next = n.get_next();
        }else{
            self.next = Some(self.first);
        }
        self.next
    }
}

impl <T: LlItem>LinkedList<T> {
    pub fn new(f: T) -> LinkedList<T> {
        LinkedList { first: f, next: None }
    }
    //TODO impl drop
}

pub struct LinkedList<T: LlItem> {
    first: T,
    next: Option<T>
}

macro_rules! impl_LlItem {
    ([$($t:ty),+]) => {
        $(impl linked_list::LlItem for $t {
            type Output = $t;
            fn get_next(self) -> Option<Self::Output> {
                if self.next == 0 as *mut Self::Output {
                    return None;
                }
                Some(unsafe{*self.next}) 
            }
        })*
    }
}
pub(crate) use impl_LlItem;

