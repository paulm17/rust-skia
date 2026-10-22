// Supplemental compile-negative counterpart to erasure_type_safety.rs.
// A sized target must not hide the actual concrete pointee's Rc/non-Send state.
use rust_skia::*;
struct Target {
    base: RefCntBase,
}
impl RefCnt for Target {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
}
struct Unrelated {
    base: RefCntBase,
    not_send: std::rc::Rc<()>,
}
impl RefCnt for Unrelated {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
}
fn main() {
    let wrong: SkSp<Target> = sk_make_sp_as(Unrelated {
        base: RefCntBase::new(),
        not_send: std::rc::Rc::new(()),
    });
    fn requires_send<T: Send>(_: T) {}
    requires_send(wrong);
}
