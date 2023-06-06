pub struct ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    value: std::mem::ManuallyDrop<T>,
    dropfn: std::mem::ManuallyDrop<F>,
}

impl<T, F> std::ops::Drop for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    fn drop(&mut self) {
        let (value, dropfn) =
            unsafe { (std::ptr::read(&*self.value), std::ptr::read(&*self.dropfn)) };
        dropfn(value);
    }
}

impl<T, F> ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    pub fn new(value: T, dropfn: F) -> Self {
        Self {
            value: std::mem::ManuallyDrop::new(value),
            dropfn: std::mem::ManuallyDrop::new(dropfn),
        }
    }
}

impl<T, F> std::ops::Deref for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, F> std::ops::DerefMut for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
