//! Port of `src/utils/SkClipStackUtils.h` and `src/utils/SkClipStackUtils.cpp`.
//!
//! `SkClipStack_AsPath` resolves the stack with `SkPathOps`' `Op(path, operand, op)`. The path
//! boolean engine is owned by **M23**, so this port handles the cases that need no boolean op and
//! reports `None` for the rest instead of returning a stale or fabricated path:
//!
//! - a `kReplace` element becomes the accumulated path (the reference assigns `operand`);
//! - the first non-replace element is combined with the reference's initial
//!   `kInverseEvenOdd` empty path, for which `Op` is exactly:
//!   `kIntersect` → the operand, `kDifference` → the operand with its inverse flag toggled.
//!
//! Any later non-replace element requires the M23 boolean engine.

use crate::sk_clip_op::SkClipOp;
use crate::sk_clip_stack::{DeviceSpaceType, Iter, IterStart, SkClipStack};
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_types::SkPathFillType;

/// `SkClipStack_AsPath`.
///
/// Returns `None` when an element needs `SkPathOps`' boolean engine (M23).
pub fn sk_clip_stack_as_path(cs: &SkClipStack) -> Option<SkPath> {
    // Build the initial path explicitly: the reference's `SkPath path;` has winding fill, then
    // `setFillType(kInverseEvenOdd)`, which describes the infinite plane.
    let mut builder = SkPathBuilder::new();
    let mut path = builder.detach(None);
    path.set_fill_type(SkPathFillType::InverseEvenOdd);

    // `Op` has not been applied yet; the first non-replace element combines with the initial
    // inverse-even-odd empty path, whose result is algebraic.
    let mut combined = false;

    let mut iter = Iter::new(cs, IterStart::Bottom);
    while let Some(element) = iter.next() {
        if element.get_device_space_type() == DeviceSpaceType::Shader {
            // TODO: Handle DeviceSpaceType::kShader somehow; it can't be turned into an SkPath
            // but perhaps the pdf backend can apply shaders in another way.
            continue;
        }
        let operand = if element.get_device_space_type() != DeviceSpaceType::Empty {
            element.as_device_space_path()
        } else {
            SkPath::default()
        };

        let element_op = element.get_op();
        if element.is_replace_op() {
            path = operand;
            combined = true;
            // TODO: Once expanding clip ops are removed, we can switch the iterator to be top
            // to bottom, which allows us to break here on encountering a replace op.
        } else if !combined {
            path = match element_op {
                SkClipOp::Intersect => operand,
                SkClipOp::Difference => {
                    let mut operand = operand;
                    operand.toggle_inverse_fill_type();
                    operand
                }
            };
            combined = true;
        } else {
            // `Op(path, operand, (SkPathOp)element_op)` needs the M23 path boolean engine.
            let _ = (path, operand, element_op);
            return None;
        }
    }
    Some(path)
}

/// Alternative spelling of [`sk_clip_stack_as_path`] matching the reference's camel-case helper
/// name in documentation.
pub fn sk_clip_stack_utils_as_path(cs: &SkClipStack) -> Option<SkPath> {
    sk_clip_stack_as_path(cs)
}
