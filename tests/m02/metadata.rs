// Reached SkMetaData tool support (not an M02 production API).
// Type/name lookup, same-size update in place, head insertion and removal follow
// tools/SkMetaData.cpp; Vec records replace its packed allocation/list pointers.
use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Type {
    S32,
    Scalar,
    Ptr,
    Bool,
}
struct Record {
    name: &'static str,
    kind: Type,
    bytes: Vec<u8>,
    count: usize,
}
#[derive(Default)]
struct MetaData {
    records: Vec<Record>,
}
impl MetaData {
    fn find(&self, name: &str, t: Type) -> Option<&Record> {
        self.records.iter().find(|r| r.name == name && r.kind == t)
    }
    fn set(&mut self, name: &'static str, t: Type, bytes: Vec<u8>) {
        if let Some(r) = self
            .records
            .iter_mut()
            .find(|r| r.name == name && r.kind == t)
        {
            r.bytes = bytes;
        } else {
            self.records.insert(
                0,
                Record {
                    name,
                    kind: t,
                    bytes,
                    count: 1,
                },
            );
        }
    }
    fn remove(&mut self, name: &str, t: Type) -> bool {
        if let Some(i) = self
            .records
            .iter()
            .position(|r| r.name == name && r.kind == t)
        {
            self.records.remove(i);
            true
        } else {
            false
        }
    }
    fn s32(&self, name: &str) -> Option<i32> {
        self.find(name, Type::S32)
            .map(|r| i32::from_ne_bytes(r.bytes[..4].try_into().unwrap()))
    }
    fn scalar(&self, name: &str) -> Option<f32> {
        self.find(name, Type::Scalar)
            .map(|r| f32::from_ne_bytes(r.bytes[..4].try_into().unwrap()))
    }
    fn boolean(&self, name: &str) -> Option<bool> {
        self.find(name, Type::Bool).map(|r| r.bytes[0] != 0)
    }
}
#[test]
fn MetaData() {
    let mut r = Reporter::default();
    let mut m = MetaData::default();
    check!(r, m.s32("int").is_none());
    check!(r, m.scalar("scalar").is_none());
    check!(r, !m.remove("int", Type::S32));
    check!(r, !m.remove("scalar", Type::Scalar));
    m.set("int", Type::S32, 12345i32.to_ne_bytes().to_vec());
    m.set("scalar", Type::Scalar, 42f32.to_ne_bytes().to_vec());
    let address = std::ptr::addr_of!(m) as usize;
    m.set("ptr", Type::Ptr, address.to_ne_bytes().to_vec());
    m.set("true", Type::Bool, vec![1]);
    m.set("false", Type::Bool, vec![0]);
    m.set("scalar", Type::Scalar, 0.5f32.to_ne_bytes().to_vec());
    check!(r, m.s32("int") == Some(12345));
    check!(r, m.scalar("scalar") == Some(0.5));
    check!(r, m.boolean("true") == Some(true));
    check!(r, m.boolean("false") == Some(false));
    let expected = [
        ("int", Type::S32, 1),
        ("scalar", Type::Scalar, 1),
        ("ptr", Type::Ptr, 1),
        ("true", Type::Bool, 1),
        ("false", Type::Bool, 1),
    ];
    let mut loops = 0;
    for record in &m.records {
        let mut matches = 0;
        for (name, t, count) in expected {
            if name == record.name {
                matches += 1;
                check!(r, t == record.kind);
                check!(r, count == record.count);
            }
        }
        check!(r, matches == 1);
        loops += 1;
    }
    check!(r, loops == expected.len());
    check!(r, m.remove("int", Type::S32));
    check!(r, m.remove("scalar", Type::Scalar));
    check!(r, m.remove("true", Type::Bool));
    check!(r, m.remove("false", Type::Bool));
    check!(r, m.s32("int").is_none());
    check!(r, m.scalar("scalar").is_none());
    check!(r, m.boolean("true").is_none());
    check!(r, m.boolean("false").is_none());
    r.finish();
}
