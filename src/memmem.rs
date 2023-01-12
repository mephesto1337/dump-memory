use std::os::raw::c_char;

extern "C" {
    fn strcasestr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
}

/// Returns the optional index of where needle is found ignore case
pub fn search_no_case(slice: &[u8], needle: &[u8]) -> Option<usize> {
    let ptr: *const u8 =
        unsafe { strcasestr(slice.as_ptr().cast(), needle.as_ptr().cast()) }.cast();

    if ptr.is_null() {
        None
    } else {
        // SAFETY:
        // * `ptr` is issued from `slice`
        // * `ptr` and `slice` are pointers to u8 so offset is a multiple of 1
        let offset: usize = unsafe { ptr.offset_from(slice.as_ptr()) }
            .try_into()
            .expect("Offset should be positive");
        Some(offset)
    }
}
