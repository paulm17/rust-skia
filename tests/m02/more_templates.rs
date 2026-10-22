use super::*;
trait Container: Default {
    fn sized(n: usize) -> Self;
    fn reset_n(&mut self, n: usize);
    fn get_ptr(&self) -> *const i32;
    fn data_ptr(&self) -> *const i32;
    fn fill(&mut self, n: usize);
    fn filled(&self, n: usize) -> bool;
}
macro_rules! container {
    ($t:ty) => {
        impl Container for $t {
            fn sized(n: usize) -> Self {
                let mut c = Self::default();
                c.reset(n);
                c
            }
            fn reset_n(&mut self, n: usize) {
                self.reset(n);
            }
            fn get_ptr(&self) -> *const i32 {
                self.get()
            }
            fn data_ptr(&self) -> *const i32 {
                self.data()
            }
            fn fill(&mut self, n: usize) {
                for i in 0..n {
                    self[i] = n as i32;
                }
            }
            fn filled(&self, n: usize) -> bool {
                for i in 0..n {
                    if self[i] != n as i32 {
                        return false;
                    }
                }
                true
            }
        }
    };
}
container!(AutoTArray<i32,0>);
container!(AutoSTArray<i32,10>);
container!(AutoTMalloc<i32>);
container!(AutoSTMalloc<10,i32>);
fn test_container_apis<C: Container>(r: &mut Reporter) {
    check!(r, C::sized(0).get_ptr().is_null());
    check!(r, C::sized(0).data_ptr().is_null());
    for n in [1, 10, 11] {
        check!(r, !C::sized(n).get_ptr().is_null());
        check!(r, !C::sized(n).data_ptr().is_null());
    }
    let mut c = C::default();
    for n in [1, 10, 11] {
        c.reset_n(n);
        c.fill(n);
        check!(r, !c.get_ptr().is_null());
        check!(r, c.get_ptr() == c.data_ptr());
        check!(r, c.filled(n));
        let m = std::mem::take(&mut c);
        check!(r, m.filled(n));
    }
    c.reset_n(0);
    check!(r, c.get_ptr().is_null());
    check!(r, c.data_ptr().is_null());
    let m = c;
    check!(r, m.get_ptr().is_null());
    check!(r, m.data_ptr().is_null());
}
#[test]
fn TemplateContainerAPIs() {
    let mut r = Reporter::default();
    test_container_apis::<AutoTArray<i32, 0>>(&mut r);
    test_container_apis::<AutoSTArray<i32, 10>>(&mut r);
    test_container_apis::<AutoTMalloc<i32>>(&mut r);
    test_container_apis::<AutoSTMalloc<10, i32>>(&mut r);
    r.finish();
}
#[test]
fn TemplateContainerAPIsTrim() {
    let mut r = Reporter::default();
    let mut c = AutoSTArray::<i32, 10>::new();
    c.trim_to(100);
    check!(r, c.size() == 0);
    c.trim_to(1);
    check!(r, c.size() == 0);
    c.trim_to(0);
    check!(r, c.size() == 0);
    c.reset(1);
    check!(r, c.size() == 1);
    c.trim_to(2);
    check!(r, c.size() == 1);
    c.trim_to(1);
    check!(r, c.size() == 1);
    c.trim_to(0);
    check!(r, c.size() == 0);
    r.finish();
}
