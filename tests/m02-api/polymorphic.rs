// RefCntTest.cpp: original sk_make_sp helper requires concrete -> abstract ownership.
use rust_skia::*;
trait Abstract: RefCnt {
    fn f(&self);
}
struct Concrete {
    base: RefCntBase,
}
impl RefCnt for Concrete {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
}
impl Abstract for Concrete {
    fn f(&self) {}
}
fn make_foo() -> SkSp<dyn Abstract> {
    rust_skia::sk_make_sp_as::<dyn Abstract, _>(Concrete {
        base: RefCntBase::new(),
    })
}
#[test]
fn sk_make_sp() {
    let _x = make_foo();
}
