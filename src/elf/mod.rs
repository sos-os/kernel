use core::mem;
use core::intrinsics;
use core::slice;

pub mod section;
pub mod header;
use self::header::Header;

pub struct Binary<'a> {
    pub header: Header
  , pub binary: &'a [u8]
}

unsafe fn extract_from_slice<'a, T: Sized>( data: &'a [u8]
                                          , offset: usize
                                          , n: usize)
                                          -> &'a [T] {
    assert!( data.len() - offset >= mem::size_of::<T>() * n
           , "Slice too short to contain {} objects of type {}"
           , n
           , intrinsics::type_name::<T>()
           );
    slice::from_raw_parts(data[offset..].as_ptr() as *const T, n)
}
