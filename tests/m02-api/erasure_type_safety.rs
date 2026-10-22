// Supplemental compile-negative contract: a non-implementer containing Rc is
// neither an Abstract nor Send. A safe factory must not manufacture this type.
use rust_skia::*;
trait Abstract: RefCnt + Send + Sync {}
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
    let wrong: SkSp<dyn Abstract> = sk_make_sp_as(Unrelated {
        base: RefCntBase::new(),
        not_send: std::rc::Rc::new(()),
    });
    fn requires_send<T: Send>(_: T) {}
    requires_send(wrong);
}
