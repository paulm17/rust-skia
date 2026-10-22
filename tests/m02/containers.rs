use super::*;
use std::cell::Cell;
use std::rc::Rc;
struct Value {
    value: i32,
    instances: Rc<Cell<i32>>,
}
impl Value {
    fn new(value: i32, c: &Rc<Cell<i32>>) -> Self {
        c.set(c.get() + 1);
        Self {
            value,
            instances: c.clone(),
        }
    }
}
impl Drop for Value {
    fn drop(&mut self) {
        self.instances.set(self.instances.get() - 1);
    }
}
#[test]
fn LRUCacheSequential() {
    let mut r = Reporter::default();
    let c = Rc::new(Cell::new(0));
    {
        let mut cache = SkLRUCache::new(100);
        for i in 1..200 {
            check!(r, cache.find(&i).is_none());
            cache.insert(i, Box::new(Value::new(i * i, &c)));
            check!(r, cache.find(&i).is_some());
            check!(r, cache.find(&i).unwrap().value == i * i);
            if i > 100 {
                check!(r, c.get() == 100);
                check!(r, cache.find(&(i - 100)).is_none());
            } else {
                check!(r, c.get() == i);
            }
            check!(r, cache.count() as i32 == c.get());
        }
    }
    check!(r, c.get() == 0);
    r.finish();
}
#[derive(PartialEq, Eq)]
struct MoveOnlyKey(i32);
#[test]
fn LRUCacheRandom() {
    let mut r = Reporter::default();
    let c = Rc::new(Cell::new(0));
    {
        let seq = [0, 1, 2, 3, 4, 1, 6, 2, 7, 5, 3, 2, 2, 3, 1, 7];
        let expected = [7, 1, 3, 2, 5];
        let mut cache = SkLRUCache::new(5);
        for k in seq {
            let key = MoveOnlyKey(k);
            if cache.find(&key).is_none() {
                cache.insert(key, Box::new(Value::new(k, &c)));
            }
        }
        check!(r, c.get() == 5);
        check!(r, cache.count() == 5);
        for k in expected {
            let key = MoveOnlyKey(k);
            check!(r, cache.find(&key).is_some());
            check!(r, cache.find(&key).unwrap().value == k);
        }
    }
    check!(r, c.get() == 0);
    r.finish();
}
#[test]
fn SkSpanBasicTemplateGuide() {
    let mut r = Reporter::default();
    {
        let mut v = vec![1, 2, 3, 4, 5];
        let mut s = SkSpanMut::new(&mut v);
        check!(r, s.as_slice()[3] == 4);
        s.as_mut_slice()[3] = 100;
        check!(r, s.as_slice()[3] == 100);
    }
    {
        let t = vec![1, 2, 3, 4, 5];
        let s = SkSpan::new(&t);
        check!(r, s[3] == 4);
        check!(r, t[3] == 4); // Rust borrows require ending the immutable view before mutation and recreating it.
        let mut t = t;
        t[3] = 100;
        let s = SkSpan::new(&t);
        check!(r, s[3] == 100);
    }
    {
        let mut v = [1, 2, 3, 4, 5];
        let mut s = SkSpanMut::new(&mut v);
        check!(r, s.as_slice()[3] == 4);
        s.as_mut_slice()[3] = 100;
        check!(r, s.as_slice()[3] == 100);
        let s = SkSpan::new(s.as_slice());
        let s1 = s.subspan(1, 3);
        check!(r, s1.size() == 3);
        check!(r, s1.first() == Some(&2));
        check!(r, s1.last() == Some(&100));
        let s2 = s.subspan_from(2);
        check!(r, s2.size() == 3);
        check!(r, s2.first() == Some(&3));
        check!(r, s2.last() == Some(&5));
    }
    {
        let mut t = [1, 2, 3, 4, 5];
        let s = SkSpan::new(&t);
        check!(r, s[3] == 4);
        check!(r, t[3] == 4);
        t[3] = 100;
        let s = SkSpan::new(&t);
        check!(r, s[3] == 100);
    }
    {
        let v: Vec<i32> = vec![];
        let s = SkSpan::new(&v);
        check!(r, s.size() == 0);
    }
    let a = SkSpan::new(&[1, 2, 3, 4]);
    check!(r, a.size() == 4);
    r.finish();
}
fn span_parameter(s: SkSpan<'_, i32>) -> bool {
    s[0] == 1 && s[1] == 2 && s[2] == 3
}
#[test]
fn SkSpanDeduceParam() {
    let mut r = Reporter::default();
    {
        let v = vec![1, 2, 3];
        check!(r, span_parameter(SkSpan::new(&v)));
        check!(r, span_parameter(SkSpan::new(&v)));
    }
    {
        let v = vec![1, 2, 3];
        check!(r, span_parameter(SkSpan::new(&v)));
        check!(r, span_parameter(SkSpan::new(&v)));
    }
    check!(r, span_parameter(SkSpan::new(&vec![1, 2, 3])));
    {
        let v = [1, 2, 3];
        check!(r, span_parameter(SkSpan::new(&v)));
    }
    {
        span_parameter(SkSpan::new(&[1, 2, 3]));
        check!(r, span_parameter(SkSpan::new(&[1, 2, 3])));
    }
    {
        let v = [1, 2, 3];
        let s = SkSpan::new(&v);
        check!(r, span_parameter(s));
    }
    r.finish();
}
#[test]
fn SkSpanDeduceSize() {
    let mut r = Reporter::default();
    let d = [1, 2, 3, 4, 5];
    {
        let s: usize = d.len();
        let span = SkSpan::new(&d[..s]);
        check!(r, span.size() == d.len());
    }
    {
        let s = d.len() as u32;
        let span = SkSpan::new(&d[..s as usize]);
        check!(r, span.size() == d.len());
    }
    {
        let s: usize = d.len();
        let span = SkSpan::new(&d[..s]);
        check!(r, span.size() == d.len());
    }
    r.finish();
}
#[derive(Clone)]
struct Message {
    x: i32,
    y: f32,
}
#[test]
fn MessageBus() {
    let mut r = Reporter::default();
    let mut bus = SkMessageBus::<Message, u32>::new();
    let i1 = bus.make_inbox(0);
    let i2 = bus.make_inbox(0);
    bus.post(Message { x: 5, y: 4.2 });
    bus.post(Message { x: 6, y: 4.3 });
    let m = i1.poll();
    check!(r, m.len() == 2);
    check!(r, m[0].x == 5);
    check!(r, m[1].x == 6);
    bus.post(Message { x: 1, y: 0.3 });
    let m = i1.poll();
    check!(r, m.len() == 1);
    check!(r, m[0].x == 1);
    let m = i1.poll();
    check!(r, m.len() == 0);
    let m = i2.poll();
    check!(r, m.len() == 3);
    check!(r, m[0].x == 5);
    check!(r, m[1].x == 6);
    check!(r, m[2].x == 1);
    r.finish();
}
struct RefMessage {
    base: RefCntBase,
    x: i32,
    y: f32,
}
impl RefCnt for RefMessage {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
}
fn refmessage(x: i32, y: f32) -> SkSp<RefMessage> {
    sk_make_sp(RefMessage {
        base: RefCntBase::new(),
        x,
        y,
    })
}
#[test]
fn MessageBusSp() {
    let mut r = Reporter::default();
    let mut bus = SkMessageBus::<SkSp<RefMessage>, u32>::new();
    let inbox = bus.make_inbox(0);
    let m1 = refmessage(5, 4.2);
    let m2 = refmessage(6, 4.3);
    bus.post_unique(m1, |_| true);
    bus.post_unique(m2, |_| true);
    let m = inbox.poll();
    check!(r, m.len() == 2);
    check!(r, m[0].unique());
    check!(r, m[1].unique());
    check!(r, m[0].x == 5);
    check!(r, m[1].x == 6);
    let m3 = refmessage(1, 0.3);
    bus.post_unique(m3, |_| true);
    let m = inbox.poll();
    check!(r, m.len() == 1);
    check!(r, m[0].unique());
    check!(r, m[0].x == 1);
    let m = inbox.poll();
    check!(r, m.len() == 0);
    r.finish();
}
#[test]
fn MessageBus_SkShouldPostMessageToBus() {
    let mut r = Reporter::default();
    let mut bus = SkMessageBus::<u32, u32>::new();
    let i1 = bus.make_inbox(1);
    let i2 = bus.make_inbox(2);
    for id in [0, 1, 2, 3] {
        bus.post_filtered(id, |inbox| {
            assert!(*inbox != 0);
            id == 0 || id == *inbox
        });
    }
    let m = i1.poll();
    check!(r, m.len() == 2);
    if m.len() == 2 {
        check!(r, m[0] == 0);
        check!(r, m[1] == 1);
    }
    let m = i2.poll();
    check!(r, m.len() == 2);
    if m.len() == 2 {
        check!(r, m[0] == 0);
        check!(r, m[1] == 2);
    }
    r.finish();
}
