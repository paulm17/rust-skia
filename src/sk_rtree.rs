//! Port of `src/core/SkRTree.h` and `src/core/SkRTree.cpp`.
//!
//! The reference stores `Node*` subtrees in a `std::vector<Node>` and a `Branch` union of
//! `Node*`/`int`. The port keeps the union as [`BranchData`] and refers to subtrees by index into
//! `nodes`, which stays stable across `Vec` growth. Bulk-load grouping and traversal order are
//! unchanged, so `search` reports the same indices in the same order.

use crate::rect::Rect;
use crate::sk_bbh_factory::SkBBoxHierarchy;

/// `SkRTree::kMinChildren`.
pub const K_MIN_CHILDREN: usize = 6;
/// `SkRTree::kMaxChildren`.
pub const K_MAX_CHILDREN: usize = 11;

#[derive(Clone, Copy, Debug)]
enum BranchData {
    /// `Branch::fSubtree`.
    Subtree(usize),
    /// `Branch::fOpIndex`.
    OpIndex(i32),
}

/// `SkRTree::Branch`.
#[derive(Clone, Copy, Debug)]
struct Branch {
    data: BranchData,
    bounds: Rect,
}

impl Branch {
    fn empty() -> Branch {
        Branch {
            data: BranchData::OpIndex(-1),
            bounds: Rect::make_empty(),
        }
    }

    fn subtree(index: usize, bounds: Rect) -> Branch {
        Branch {
            data: BranchData::Subtree(index),
            bounds,
        }
    }
}

/// `SkRTree::Node`.
#[derive(Clone, Copy, Debug)]
struct Node {
    num_children: u16,
    level: u16,
    children: [Branch; K_MAX_CHILDREN],
}

impl Node {
    fn new(level: u16) -> Node {
        Node {
            num_children: 0,
            level,
            children: [Branch::empty(); K_MAX_CHILDREN],
        }
    }
}

/// An R-Tree implementation; see `src/core/SkRTree.h` for the algorithm description.
#[derive(Clone, Debug)]
pub struct SkRTree {
    /// This is the count of data elements (rather than total nodes in the tree).
    f_count: i32,
    f_root: Branch,
    f_nodes: Vec<Node>,
}

impl Default for SkRTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SkRTree {
    pub fn new() -> SkRTree {
        SkRTree {
            f_count: 0,
            f_root: Branch::empty(),
            f_nodes: Vec::new(),
        }
    }

    /// `SkRTree::insert`.
    pub fn insert_rects(&mut self, bounds_array: &[Rect]) {
        debug_assert_eq!(0, self.f_count);

        let mut branches: Vec<Branch> = Vec::with_capacity(bounds_array.len());
        for (i, bounds) in bounds_array.iter().enumerate() {
            if bounds.is_empty() {
                continue;
            }
            branches.push(Branch {
                data: BranchData::OpIndex(i as i32),
                bounds: *bounds,
            });
        }

        self.f_count = branches.len() as i32;
        if self.f_count != 0 {
            if 1 == self.f_count {
                self.f_nodes.reserve(1);
                let n = self.allocate_node_at_level(0);
                let node = &mut self.f_nodes[n];
                node.num_children = 1;
                node.children[0] = branches[0];
                self.f_root = Branch::subtree(n, branches[0].bounds);
            } else {
                let count = Self::count_nodes(self.f_count as usize);
                self.f_nodes.reserve(count);
                self.f_root = self.bulk_load(&mut branches, 0);
            }
        }
    }

    fn allocate_node_at_level(&mut self, level: u16) -> usize {
        self.f_nodes.push(Node::new(level));
        self.f_nodes.len() - 1
    }

    /// `SkRTree::CountNodes`.
    fn count_nodes(branches: usize) -> usize {
        if branches == 1 {
            return 1;
        }
        let mut remainder = branches % K_MAX_CHILDREN;
        if remainder > 0 {
            if remainder >= K_MIN_CHILDREN {
                remainder = 0;
            } else {
                remainder = K_MIN_CHILDREN - remainder;
            }
        }
        let mut current_branch = 0usize;
        let mut nodes = 0usize;
        while current_branch < branches {
            let mut increment_by = K_MAX_CHILDREN;
            if remainder != 0 {
                if remainder <= K_MAX_CHILDREN - K_MIN_CHILDREN {
                    increment_by -= remainder;
                    remainder = 0;
                } else {
                    increment_by = K_MIN_CHILDREN;
                    remainder -= K_MAX_CHILDREN - K_MIN_CHILDREN;
                }
            }
            nodes += 1;
            current_branch += 1;
            let mut k = 1;
            while k < increment_by && current_branch < branches {
                current_branch += 1;
                k += 1;
            }
        }
        nodes + Self::count_nodes(nodes)
    }

    /// `SkRTree::bulkLoad`. Consumes the input array.
    fn bulk_load(&mut self, branches: &mut Vec<Branch>, level: u16) -> Branch {
        if branches.len() == 1 {
            // Only one branch. It will be the root.
            return branches[0];
        }

        // We might sort our branches here, but we expect Blink gives us a reasonable x,y order.
        // Skipping a call to sort (in Y) here resulted in a 17% win for recording with negligible
        // difference in playback speed.
        let mut remainder = branches.len() % K_MAX_CHILDREN;
        let mut new_branches = 0usize;

        if remainder > 0 {
            // If the remainder isn't enough to fill a node, we'll add fewer nodes to other branches.
            if remainder >= K_MIN_CHILDREN {
                remainder = 0;
            } else {
                remainder = K_MIN_CHILDREN - remainder;
            }
        }

        let mut current_branch = 0usize;
        while current_branch < branches.len() {
            let mut increment_by = K_MAX_CHILDREN;
            if remainder != 0 {
                // if need be, omit some nodes to make up for remainder
                if remainder <= K_MAX_CHILDREN - K_MIN_CHILDREN {
                    increment_by -= remainder;
                    remainder = 0;
                } else {
                    increment_by = K_MIN_CHILDREN;
                    remainder -= K_MAX_CHILDREN - K_MIN_CHILDREN;
                }
            }
            let n = self.allocate_node_at_level(level);
            {
                let node = &mut self.f_nodes[n];
                node.num_children = 1;
                node.children[0] = branches[current_branch];
            }
            let mut b = Branch {
                data: BranchData::Subtree(n),
                bounds: branches[current_branch].bounds,
            };
            current_branch += 1;
            let mut k = 1;
            while k < increment_by && current_branch < branches.len() {
                b.bounds.join(&branches[current_branch].bounds);
                {
                    let node = &mut self.f_nodes[n];
                    node.children[k] = branches[current_branch];
                    node.num_children += 1;
                }
                current_branch += 1;
                k += 1;
            }
            branches[new_branches] = b;
            new_branches += 1;
        }
        branches.truncate(new_branches);
        self.bulk_load(branches, level + 1)
    }

    fn search_node(&self, node_index: usize, query: &Rect, results: &mut Vec<i32>) {
        let node = &self.f_nodes[node_index];
        for i in 0..node.num_children as usize {
            let branch = &node.children[i];
            if Rect::intersects_rects(&branch.bounds, query) {
                if 0 == node.level {
                    if let BranchData::OpIndex(index) = branch.data {
                        results.push(index);
                    }
                } else if let BranchData::Subtree(child) = branch.data {
                    self.search_node(child, query, results);
                }
            }
        }
    }

    /// `SkRTree::getDepth`.
    pub fn get_depth(&self) -> i32 {
        if self.f_count != 0 {
            if let BranchData::Subtree(root) = self.f_root.data {
                return self.f_nodes[root].level as i32 + 1;
            }
        }
        0
    }

    /// `SkRTree::getCount`. Insertion count (not overall node count, which may be greater).
    pub fn get_count(&self) -> i32 {
        self.f_count
    }

    /// The root bounds, for development diagnostics.
    pub fn get_bounds(&self) -> Rect {
        self.f_root.bounds
    }

    /// Number of allocated nodes; only public for development diagnostics.
    pub fn node_count(&self) -> usize {
        self.f_nodes.len()
    }
}

impl SkBBoxHierarchy for SkRTree {
    fn insert(&mut self, rects: &[Rect]) {
        self.insert_rects(rects);
    }

    fn search(&self, query: &Rect, results: &mut Vec<i32>) {
        if self.f_count > 0 && Rect::intersects_rects(&self.f_root.bounds, query) {
            if let BranchData::Subtree(root) = self.f_root.data {
                self.search_node(root, query, results);
            }
        }
    }

    fn bytes_used(&self) -> usize {
        // The reference reports the size of the object plus the node capacity.
        core::mem::size_of::<SkRTree>() + self.f_nodes.capacity() * core::mem::size_of::<Node>()
    }
}
