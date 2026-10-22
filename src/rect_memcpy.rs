/// Copies a rectangular subregion of a row-oriented buffer.
///
/// # Safety
///
/// `dst` must be valid for writes of `dst_row_bytes * (row_count - 1) + trim_row_bytes`
/// bytes, and `src` must be valid for reads of
/// `src_row_bytes * (row_count - 1) + trim_row_bytes` bytes. The regions must not
/// overlap.
pub unsafe fn sk_rect_memcpy(
    dst: *mut u8,
    dst_row_bytes: usize,
    src: *const u8,
    src_row_bytes: usize,
    trim_row_bytes: usize,
    row_count: usize,
) {
    debug_assert!(trim_row_bytes <= dst_row_bytes);
    debug_assert!(trim_row_bytes <= src_row_bytes);
    if row_count == 0 {
        return;
    }
    if trim_row_bytes == dst_row_bytes && trim_row_bytes == src_row_bytes {
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, trim_row_bytes * row_count);
        }
        return;
    }

    let mut dst = dst;
    let mut src = src;
    for _ in 0..row_count {
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, trim_row_bytes);
            dst = dst.add(dst_row_bytes);
            src = src.add(src_row_bytes);
        }
    }
}
