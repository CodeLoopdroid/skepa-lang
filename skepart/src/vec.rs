use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::RtValue;

#[derive(Debug, Clone, PartialEq)]
pub struct RtVec(Rc<RefCell<Vec<RtValue>>>);

impl RtVec {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn borrow(&self) -> Ref<'_, Vec<RtValue>> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Vec<RtValue>> {
        self.0.borrow_mut()
    }
}

impl Default for RtVec {
    fn default() -> Self {
        Self::new()
    }
}
