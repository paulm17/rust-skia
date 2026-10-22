// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/SortTest.cpp:38 at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
unsafe extern "C" fn compare_int(a: *const libc::c_void, b: *const libc::c_void) -> libc::c_int {
    unsafe { *a.cast::<i32>() - *b.cast::<i32>() }
}
fn rand_array(rand: &mut SkRandom, array: &mut [i32], n: usize) {
    for j in 0..n {
        array[j] = rand.next_s() & 0xff;
    }
}
fn check_sort(r: &mut Reporter, label: &str, array: &[i32], reference: &[i32], n: usize) {
    for j in 0..n {
        if array[j] != reference[j] {
            r.record(
                false,
                &format!("{label}Sort [{n}] failed {} {}", array[j], reference[j]),
                file!(),
                line!(),
            );
        }
    }
}
#[test]
fn Sort() {
    let mut r = Reporter::default();
    let mut random_array = [0i32; 500];
    let mut rand = SkRandom::new();
    for _ in 0..10000 {
        let count = rand.next_range_u(1, random_array.len() as u32) as usize;
        rand_array(&mut rand, &mut random_array, count);
        let mut sorted_array = random_array;
        unsafe {
            libc::qsort(
                sorted_array.as_mut_ptr().cast(),
                count,
                std::mem::size_of::<i32>(),
                Some(compare_int),
            );
        }
        let mut working_array = random_array;
        sk_theapsort(&mut working_array[..count], |a, b| a < b);
        check_sort(&mut r, "Heap", &working_array, &sorted_array, count);
        working_array = random_array;
        sk_tqsort(&mut working_array[..count], |a, b| a.cmp(b));
        check_sort(&mut r, "Quick", &working_array, &sorted_array, count);
    }
    r.finish();
}
