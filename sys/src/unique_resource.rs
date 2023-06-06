pub trait ResourceDeleter<T>
where
    T: Copy + std::cmp::PartialEq,
{
    fn null() -> T;

    fn is_null(r: T) -> bool {
        r == Self::null()
    }

    fn destroy(&mut self, r: T);
}

pub struct UniqueResource<T, D>
where
    T: Copy + std::cmp::PartialEq,
    D: ResourceDeleter<T> + std::default::Default,
{
    res: T,
    deleter: D,
}

impl<T, D> UniqueResource<T, D>
where
    T: Copy + std::cmp::PartialEq,
    D: ResourceDeleter<T> + std::default::Default,
{
    pub fn new(r: T) -> Option<Self> {
        if D::is_null(r) {
            None
        } else {
            Some(Self {
                res: r,
                deleter: D::default(),
            })
        }
    }

    pub fn new_with_deleter(r: T, d: D) -> Option<Self> {
        if D::is_null(r) {
            None
        } else {
            Some(Self { res: r, deleter: d })
        }
    }

    pub fn is_valid(&self) -> bool {
        !D::is_null(self.res)
    }
}

impl<T, D> std::ops::Deref for UniqueResource<T, D>
where
    T: Copy + std::cmp::PartialEq,
    D: ResourceDeleter<T> + std::default::Default,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if !self.is_valid() {
            panic!("Derefing a null handle!");
        }

        &self.res
    }
}

impl<T, D> std::ops::DerefMut for UniqueResource<T, D>
where
    T: Copy + std::cmp::PartialEq,
    D: ResourceDeleter<T> + std::default::Default,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        if !self.is_valid() {
            panic!("Derefing a null handle!");
        }

        &mut self.res
    }
}

impl<T, D> std::default::Default for UniqueResource<T, D>
where
    T: Copy + std::cmp::PartialEq,
    D: ResourceDeleter<T> + std::default::Default,
{
    fn default() -> Self {
        Self {
            res: D::null(),
            deleter: D::default(),
        }
    }
}

impl<T, D> std::ops::Drop for UniqueResource<T, D>
where
    T: Copy + std::cmp::PartialEq,
    D: ResourceDeleter<T> + std::default::Default,
{
    fn drop(&mut self) {
        if self.is_valid() {
            self.deleter.destroy(self.res);
        }
    }
}

#[macro_export]
macro_rules! gen_unique_resource_type {
    ($tyname:tt, $delname:tt, $resty:ty, $nullval:expr, $dtor:expr) => {
        #[derive(Default)]
        pub struct $delname {}

        impl $crate::ResourceDeleter<$resty> for $delname {
            fn null() -> $resty {
                ($nullval)
            }
            fn destroy(&mut self, h: $resty) {
                ($dtor)(h);
            }
        }

        pub type $tyname = $crate::UniqueResource<$resty, $delname>;
    };
}
