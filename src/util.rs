//! useful structures

// Ref: once_cell

use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, Pointee},
};

/// A thread-safe fat pointer cell which can be written to only once.
pub struct OnceFatBox<T: ?Sized> {
    thin_ptr: UnsafeCell<*mut ()>,
    lock: UnsafeCell<u8>,
    meta: MaybeUninit<<T as Pointee>::Metadata>,
    _marker: PhantomData<Option<Box<T>>>,
}

impl<T: ?Sized> Default for OnceFatBox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Drop for OnceFatBox<T> {
    fn drop(&mut self) {
        let data_address = *self.thin_ptr.get_mut();
        if !data_address.is_null() {
            let metadata = unsafe { self.meta.assume_init() };
            let fat_ptr: *mut T = ptr::from_raw_parts_mut(data_address, metadata);
            drop(unsafe { Box::from_raw(fat_ptr) })
        }
    }
}

impl<T: ?Sized + Debug> Debug for OnceFatBox<T>
where
    <T as Pointee>::Metadata: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnceFatBox")
            .field("address", &self.thin_ptr)
            .field("meta", &self.meta)
            .finish()
    }
}

impl<T: ?Sized> OnceFatBox<T> {
    /// Creates a new empty cell.
    pub const fn new() -> OnceFatBox<T> {
        OnceFatBox {
            thin_ptr: UnsafeCell::new(ptr::null_mut()),
            lock: UnsafeCell::new(0),
            meta: MaybeUninit::uninit(), // value meaning ignored when thin_ptr is null
            _marker: PhantomData,
        }
    }

    /// Gets a reference to the underlying value.
    pub fn get(&self) -> Option<&T> {
        let data_address = self.thin_ptr.get();
        if data_address.is_null() {
            return None;
        }
        let metadata = unsafe { self.meta.assume_init() };
        let fat_ptr: *const T = ptr::from_raw_parts(unsafe { *data_address }, metadata);
        Some(unsafe { &*fat_ptr })
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell was empty and `Err(value)` if it was full.
    pub fn set(&self, value: Box<T>) -> Result<(), Box<T>> {
        let fat_ptr = Box::into_raw(value);
        let data_address = fat_ptr as *mut ();
        // compare-exchange using amo operations
        let exchange = unsafe {
            let mut ans = Err(());
            asm!(
                "1: lw  {tmp}, ({lock})", // check if lock is held
                "bnez   {tmp}, 1b", // retry if held
                "amoswap.w.aq {tmp}, {one}, ({lock})", // attempt to acquire lock
                "bnez   {tmp}, 1b", // retry if held
                lock = in(reg) self.lock.get(),
                tmp = out(reg) _,
                one = const 1,
            );
            // critical section begin
            if self.thin_ptr.get() == ptr::null_mut() {
                *self.thin_ptr.get() = data_address;
                *(self.meta.as_ptr() as *mut _) = ptr::metadata(fat_ptr);
                ans = Ok(())
            }
            // critical section end
            asm!(
                "amoswap.w.rl x0, x0, ({lock})", // release lock by storing 0
                lock = in(reg) self.lock.get(),
            );
            ans
        };
        if let Err(_) = exchange {
            let value = unsafe { Box::from_raw(fat_ptr) };
            return Err(value);
        }
        Ok(())
    }
}

unsafe impl<T: Sync + Send + ?Sized> Sync for OnceFatBox<T> {}
