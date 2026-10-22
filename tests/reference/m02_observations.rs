// Supplemental diagnostics for the original allocator/template failures.
use rust_skia::*;
fn main() {
    let p = BlockAllocator::new(GrowthPolicy::Fixed, 2048);
    println!(
        "BlockAllocator sizeof={} prealloc={} usable={}",
        std::mem::size_of::<BlockAllocator>(),
        p.prealloc_size(),
        p.prealloc_usable_space()
    );
    let p = BlockAllocator::with_prealloc(
        GrowthPolicy::Linear,
        1024,
        1024 - std::mem::size_of::<BlockAllocator>(),
    );
    println!(
        "placement requested=1024 actual={} usable={}",
        p.prealloc_size(),
        p.prealloc_usable_space()
    );
    let p = SkSBlockAllocator::<2048>::new(GrowthPolicy::Fixed);
    println!(
        "inline prealloc={} usable={} required_min={}",
        p.prealloc_size(),
        p.prealloc_usable_space(),
        2048 - std::mem::size_of::<BlockAllocator>()
    );
    let mut a = AutoTMalloc::<i32>::new();
    a.reset(10);
    a.realloc(0);
    println!(
        "AutoTMalloc realloc(0) ptr={:p} count={}",
        a.get(),
        a.count()
    );
}
