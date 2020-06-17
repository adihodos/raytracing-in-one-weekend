#![allow(dead_code)]

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct GenericHandle<T> {
    handle: u32,
    _nothing: std::marker::PhantomData<T>,
}

impl<T> GenericHandle<T> {
    pub fn handle(&self) -> u32 {
        self.handle
    }
}

impl<T> std::convert::From<u32> for GenericHandle<T> {
    fn from(raw: u32) -> GenericHandle<T> {
        GenericHandle {
            handle: raw,
            _nothing: std::marker::PhantomData,
        }
    }
}
