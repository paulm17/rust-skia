// Test-local Graph and Node from IncrTopoSortTest.cpp. Containers, sorting,
// strings and intrusive strong ownership remain the production Rust APIs.
use super::*;
use std::cell::{Cell, RefCell};
struct Node {
    base: RefCntBase,
    id: char,
    index: Cell<usize>,
    visited: Cell<bool>,
    deps: RefCell<SkTDArray<*mut Node>>,
    dependents: RefCell<SkTDArray<*mut Node>>,
}
impl RefCnt for Node {
    fn ref_cnt_base(&self) -> &RefCntBase {
        &self.base
    }
}
struct Entry {
    node: SkSp<Node>,
    dest: *mut Node,
}
struct Graph<'a> {
    nodes: SkTArray<SkSp<Node>>,
    stack: SkTArray<Entry>,
    r: &'a mut Reporter,
}
impl<'a> Graph<'a> {
    fn new(n: usize, r: &'a mut Reporter) -> Self {
        let mut nodes = SkTArray::new();
        nodes.reserve(n);
        Self {
            nodes,
            stack: SkTArray::new(),
            r,
        }
    }
    fn validate(&mut self) {
        check!(self.r, self.stack.empty());
        for i in 0..self.nodes.count() {
            let n = &self.nodes[i];
            check!(self.r, n.index.get() == i);
            for d in n.deps.borrow().iter() {
                check!(self.r, unsafe { (**d).index.get() } < n.index.get());
            }
            for d in n.dependents.borrow().iter() {
                check!(self.r, n.index.get() < unsafe { (**d).index.get() });
            }
            check!(self.r, !n.visited.get());
        }
        for e in self.stack.iter() {
            debug_assert!(e.node.visited.get());
        }
    }
    fn add_node(&mut self, id: char) -> *mut Node {
        self.validate();
        let tmp = sk_make_sp(Node {
            base: RefCntBase::new(),
            id,
            index: Cell::new(0),
            visited: Cell::new(false),
            deps: RefCell::new(SkTDArray::new()),
            dependents: RefCell::new(SkTDArray::new()),
        });
        let ptr = tmp.as_ptr() as *mut Node;
        self.nodes.push_back(tmp.clone());
        tmp.index.set(self.nodes.count() - 1);
        self.validate();
        ptr
    }
    fn add_edge(&mut self, d: *mut Node, n: *mut Node) {
        let mut deps = SkTDArray::new();
        deps.push_back(d);
        self.add_edges(&mut deps, n);
    }
    fn add_edges(&mut self, deps: &mut SkTDArray<*mut Node>, n: *mut Node) {
        self.validate();
        let mut i = 0;
        unsafe {
            while i < deps.count() {
                let d = deps[i];
                (*n).deps.borrow_mut().push_back(d);
                (*d).dependents.borrow_mut().push_back(n);
                if (*d).index.get() < (*n).index.get() {
                    deps.remove_shuffle(i);
                } else {
                    i += 1;
                }
            }
            if deps.empty() {
                return;
            }
            sk_tqsort(deps.as_mut_slice(), |a, b| {
                (**b).index.get().cmp(&(**a).index.get())
            });
            let mut lower = self.nodes.count();
            for d in deps.iter() {
                if (**d).index.get() < lower {
                    self.shift(lower);
                }
                if !(*n).visited.get() {
                    self.dfs(n, (**d).index.get());
                }
                lower = lower.min((*n).index.get());
            }
            self.shift(lower);
        }
        self.validate();
    }
    fn dfs(&mut self, n: *mut Node, upper: usize) {
        unsafe {
            (*n).visited.set(true);
            for d in (*n).dependents.borrow().iter() {
                debug_assert!((**d).index.get() != upper);
                if !(**d).visited.get() && (**d).index.get() < upper {
                    self.dfs(*d, upper);
                }
            }
            self.stack.push_back(Entry {
                node: sk_ref_sp(&*n),
                dest: self.nodes[upper].as_ptr() as *mut Node,
            });
        }
    }
    fn move_node(&mut self, node: SkSp<Node>, index: usize) {
        debug_assert!(self.nodes[index].is_null());
        self.nodes[index] = node;
        self.nodes[index].index.set(index);
    }
    fn shift(&mut self, mut index: usize) {
        let mut removed = 0;
        while !self.stack.empty() {
            let node = self.nodes[index].clone();
            let in_stack = self.stack.iter().any(|e| e.node == node);
            if node.visited.get() {
                debug_assert!(in_stack);
                node.visited.set(false);
                self.nodes[index] = SkSp::null();
                removed += 1;
            } else {
                debug_assert!(!in_stack);
                debug_assert!(removed > 0);
                self.move_node(node.clone(), index - removed);
                self.nodes[index] = SkSp::null();
            }
            while !self.stack.empty()
                && std::ptr::eq(node.as_ptr(), self.stack.back().unwrap().dest)
            {
                removed -= 1;
                let e = self.stack.back().unwrap().node.clone();
                self.move_node(e, index - removed);
                self.stack.pop_back();
            }
            index += 1;
        }
    }
    fn actual(&mut self) -> SkString {
        self.validate();
        let mut s = SkString::new();
        for i in 0..self.nodes.count() {
            s.append_unichar(self.nodes[i].id as u32);
            if i < self.nodes.count() - 1 {
                s.append_str(",");
            }
        }
        s
    }
}
fn test_1(reporter: &mut Reporter) {
    let mut g = Graph::new(10, reporter);

    let nodeQ = g.add_node('q');
    let nodeY = g.add_node('y');
    let nodeA = g.add_node('a');
    let nodeZ = g.add_node('z');
    let nodeB = g.add_node('b');
    g.add_node('c');
    let nodeW = g.add_node('w');
    let nodeD = g.add_node('d');
    let nodeX = g.add_node('x');
    let nodeR = g.add_node('r');

    // All these edge are non-invalidating
    g.add_edge(nodeD, nodeR);
    g.add_edge(nodeZ, nodeW);
    g.add_edge(nodeA, nodeB);
    g.add_edge(nodeY, nodeZ);
    g.add_edge(nodeQ, nodeA);

    {
        let kExpectedInitialState = SkString::from_str("q,y,a,z,b,c,w,d,x,r");
        let actualInitialState = g.actual();
        check!(g.r, kExpectedInitialState == actualInitialState);
    }

    // Add the invalidating edge
    g.add_edge(nodeX, nodeY);

    {
        let kExpectedFinalState = SkString::from_str("q,a,b,c,d,x,y,z,w,r");
        let actualFinalState = g.actual();
        check!(g.r, kExpectedFinalState == actualFinalState);
    }
}

// This test adds two invalidating edge sequentially
fn test_2(reporter: &mut Reporter) {
    let mut g = Graph::new(10, reporter);

    let nodeY = g.add_node('y');
    g.add_node('a');
    let nodeW = g.add_node('w');
    g.add_node('b');
    let nodeZ = g.add_node('z');
    let nodeU = g.add_node('u');
    g.add_node('c');
    let nodeX = g.add_node('x');
    g.add_node('d');
    let nodeV = g.add_node('v');

    // All these edge are non-invalidating
    g.add_edge(nodeU, nodeX);
    g.add_edge(nodeW, nodeU);
    g.add_edge(nodeW, nodeZ);
    g.add_edge(nodeY, nodeZ);

    {
        let kExpectedInitialState = SkString::from_str("y,a,w,b,z,u,c,x,d,v");
        let actualInitialState = g.actual();
        check!(g.r, kExpectedInitialState == actualInitialState);
    }

    // Add the first invalidating edge
    g.add_edge(nodeX, nodeY);

    {
        let kExpectedFirstState = SkString::from_str("a,w,b,u,c,x,y,z,d,v");
        let actualFirstState = g.actual();
        check!(g.r, kExpectedFirstState == actualFirstState);
    }

    // Add the second invalidating edge
    g.add_edge(nodeV, nodeW);

    {
        let kExpectedSecondState = SkString::from_str("a,b,c,d,v,w,u,x,y,z");
        let actualSecondState = g.actual();
        check!(g.r, kExpectedSecondState == actualSecondState);
    }
}

fn test_diamond(reporter: &mut Reporter) {
    let mut g = Graph::new(4, reporter);

    let nodeD = g.add_node('d');
    let nodeC = g.add_node('c');
    let nodeB = g.add_node('b');

    {
        let mut dependedOn = SkTDArray::new();
        dependedOn.push_back(nodeB);
        dependedOn.push_back(nodeC);

        g.add_edges(&mut dependedOn, nodeD); // nodes B and C must come before node D
    }

    let nodeA = g.add_node('a');
    g.add_edge(nodeA, nodeB); // node A must come before node B
    g.add_edge(nodeA, nodeC); // node A must come before node C

    let kExpected0 = SkString::from_str("a,c,b,d");
    let kExpected1 = SkString::from_str("a,b,c,d");
    let actual = g.actual();

    check!(g.r, kExpected0 == actual || kExpected1 == actual);
}

fn test_lopsided_binary_tree(reporter: &mut Reporter) {
    let mut g = Graph::new(7, reporter);

    let nodeG = g.add_node('g');
    let nodeF = g.add_node('f');
    let nodeE = g.add_node('e');
    let nodeD = g.add_node('d');
    let nodeC = g.add_node('c');
    let nodeB = g.add_node('b');
    let nodeA = g.add_node('a');

    g.add_edge(nodeE, nodeG);
    g.add_edge(nodeE, nodeF);
    g.add_edge(nodeC, nodeE);
    g.add_edge(nodeC, nodeD);
    g.add_edge(nodeA, nodeC);
    g.add_edge(nodeA, nodeB);

    let kExpected = SkString::from_str("a,b,c,d,e,f,g");
    let actual = g.actual();

    check!(g.r, kExpected == actual);
}

#[test]
fn IncrTopoSort() {
    let mut r = Reporter::default();
    test_1(&mut r);
    test_2(&mut r);
    test_diamond(&mut r);
    test_lopsided_binary_tree(&mut r);
    r.finish();
}
