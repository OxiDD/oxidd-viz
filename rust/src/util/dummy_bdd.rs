use itertools::{EitherOrBoth, Itertools};
use oxidd::util::OutOfMemory;
use oxidd::{util::Borrowed, Edge, InnerNode, Manager, ManagerRef};
use oxidd::{BooleanFunction, Function};
use oxidd_manager_index::node::fixed_arity::NodeWithLevel;
use oxidd_rules_bdd::simple::BDDTerminal;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::hash::Hasher;
use std::hash::{DefaultHasher, Hash};
use std::iter::Cloned;
use std::rc::Rc;
use std::slice::Iter;
use std::sync::Arc;

use oxidd_core::util::DropWith;
use oxidd_core::util::{AllocResult, BorrowedEdgeIter};
use oxidd_core::DiagramRules;
use oxidd_core::LevelNo;
use oxidd_core::LevelView;
use oxidd_core::Node;
use oxidd_core::NodeID;
use oxidd_core::ReducedOrNew;
use oxidd_core::WorkerManager;
use oxidd_core::{BroadcastContext, HasLevel};

use crate::util::logging::console;

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Clone, PartialEq, Eq)]
pub struct DummyBDDManagerRef(Rc<RefCell<DummyBDDManager>>);

impl Hash for DummyBDDManagerRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}
impl<'a> From<&'a DummyBDDManager> for DummyBDDManagerRef {
    fn from(value: &'a DummyBDDManager) -> Self {
        DummyBDDManagerRef(Rc::new(RefCell::new(value.clone())))
    }
}
impl ManagerRef for DummyBDDManagerRef {
    type Manager<'id> = DummyBDDManager;

    fn with_manager_shared<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(&Self::Manager<'id>) -> T,
    {
        f(&self.0.borrow())
    }

    fn with_manager_exclusive<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(&mut Self::Manager<'id>) -> T,
    {
        f(&mut self.0.borrow_mut())
    }
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DummyBDDFunction(pub DummyBDDEdge);
impl DummyBDDFunction {
    pub fn from(manager_ref: &mut DummyBDDManagerRef, data: &str) -> DummyBDDFunction {
        manager_ref.with_manager_exclusive(|manager| {
            let mut root = Option::None;
            let transition_texts = data.split(",");
            let edges = transition_texts.flat_map(|item| {
                let trans = item.split(">");
                let mut out = Vec::new();
                let mut prev_node = Option::None;
                for node in trans {
                    let node: NodeID = node.trim().parse().unwrap();

                    if let Some(prev) = prev_node {
                        out.push((prev, node.clone()));
                    }
                    prev_node = Some(node);
                }
                out
            });
            for (from, to) in edges.clone() {
                if root == None {
                    root = Some(from.clone());
                }
                manager.add_node(from);
                manager.add_node(to);
            }
            for (from, to) in edges {
                manager.add_edge(from, to, manager_ref.clone());
            }

            DummyBDDFunction(DummyBDDEdge::new(
                Arc::new(root.unwrap()),
                manager_ref.clone(),
            ))
        })
    }
    pub fn from_dddmp(
        manager_ref: &mut DummyBDDManagerRef,
        data: &str,
    ) -> (Vec<(DummyBDDFunction, Vec<String>)>, Vec<String>) {
        manager_ref.with_manager_exclusive(|manager| {
            let mut terminals = HashMap::new();

            let get_text = |from: &str, to: &str| {
                let start = data.find(from).unwrap() + from.len();
                Box::new(&data[start + 1..start + data[start..].find(to).unwrap()])
            };

            let roots_text = get_text(".rootids", "\n");
            let roots = roots_text
                .trim()
                .split(" ")
                .flat_map(|n| n.parse::<usize>())
                .collect_vec();
            let root_names = if data.find(".rootnames").is_some() {
                let roots_names_text = get_text(".rootnames", "\n");
                roots_names_text
                    .trim()
                    .split(" ")
                    .map(|t| t.to_string())
                    .collect_vec()
            } else {
                roots
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("f{i}"))
                    .collect_vec()
            };

            let node_text = get_text(".nodes", ".end");
            let nodes_data = node_text.split("\n").filter_map(|node| {
                let parts = node.trim().split(" ").collect::<Vec<&str>>();
                if parts.len() >= 4 {
                    let id: NodeID = parts[0].parse().unwrap();
                    let level = parts[1];
                    let children = parts[2..].iter().map(|v| v.parse().unwrap()).collect_vec();
                    Some((id, level, children))
                } else {
                    None
                }
            });
            let mut max_level = 0;
            for (_, level, _) in nodes_data.clone() {
                let Ok(level) = level.parse() else { continue };

                if level > max_level {
                    max_level = level;
                }
            }

            for (id, level, children) in nodes_data.clone() {
                let level_num = level.parse();
                manager.add_node_level(
                    id.clone(),
                    if let Ok(level) = level_num {
                        level
                    } else {
                        max_level + 1 // Terminal nodes don't define a level, we have to assign it
                    },
                    if level_num.is_ok() {
                        None
                    } else {
                        Some(level.to_string())
                    },
                );

                if level_num.is_err() {
                    terminals.insert(
                        level.to_string(),
                        DummyBDDEdge::new(Arc::new(id), manager_ref.clone()),
                    );
                }
            }

            for (id, level, children) in nodes_data {
                if manager.has_edges(id) {
                    continue; // This node was already loaded
                }
                if level.parse::<i32>().is_err() {
                    continue;
                }; // Filter out terminals

                let is_terminal = |_: NodeID| false;
                // let is_terminal = |to: NodeID| to == 1 || to == 2;
                // let is_terminal = |to: NodeID| to == 1; // Only filter connections to false

                for child in children {
                    if !is_terminal(child) {
                        manager.add_edge(id.clone(), child, manager_ref.clone());
                    }
                }
            }

            manager.init_terminals(terminals);

            let mut func_map = HashMap::<NodeID, (DummyBDDFunction, Vec<String>)>::new();
            for (root, name) in roots.into_iter().zip(root_names.into_iter()) {
                func_map
                    .entry(root)
                    .or_insert_with(|| {
                        (
                            DummyBDDFunction(DummyBDDEdge::new(
                                Arc::new(root),
                                manager_ref.clone(),
                            )),
                            vec![],
                        )
                    })
                    .1
                    .push(name.to_string());
            }
            let funcs = func_map.values().cloned().collect_vec();

            let var_names_text = if data.find(".suppvarnames").is_some() {
                get_text(".suppvarnames", ".orderedvarnames")
            } else {
                get_text(".permids", ".nroots")
            };
            let var_names = var_names_text
                .trim()
                .split(" ")
                .map(|t| t.to_string())
                .collect_vec();
            (funcs, var_names)
        })
    }
    pub fn from_buddy(
        manager_ref: &mut DummyBDDManagerRef,
        data: &str,
        var_data: Option<&str>,
    ) -> (Vec<(DummyBDDFunction, Vec<String>)>, Vec<String>) {
        manager_ref.with_manager_exclusive(|manager| {
            let mut variables = Vec::new();
            let mut layer_levels = Vec::<usize>::new(); // Specifies per "layer", what level it should have. Variable names and nodes refer to layers, not levels.
            let mut referenced = HashSet::<usize>::new();
            let mut defined = HashSet::<usize>::new();
            let mut root = None;
            let mut max_level = 0;
            for (line, text) in data.split("\n").enumerate() {
                match line {
                    0 => {}
                    1 => {
                        layer_levels = text
                            .trim()
                            .split(" ")
                            .filter_map(|v| v.parse::<usize>().ok())
                            .collect();
                        let mut order = vec![0; layer_levels.len()];
                        for (layer, &index) in layer_levels.iter().enumerate() {
                            order[index] = layer;
                        }

                        variables = match var_data {
                            Some(vars) => {
                                let var_names =
                                    vars.split("\n").map(|v| v.trim().to_string()).collect_vec();
                                order.iter().map(|&i| var_names[i].clone()).collect()
                            }
                            _ => order.iter().map(|v| format!("{}", v)).collect(),
                        };
                    }
                    _ => {
                        let parts = text.trim().split(" ").collect_vec();
                        if parts.len() != 4 {
                            continue;
                        }

                        let Ok(id) = parts[0].parse::<usize>() else {
                            continue;
                        };
                        let Ok(layer) = parts[1].parse::<usize>() else {
                            continue;
                        };
                        let level = layer_levels.get(layer).cloned().unwrap_or(0) as u32;
                        let Ok(false_branch) = parts[2].parse::<usize>() else {
                            continue;
                        };
                        let Ok(true_branch) = parts[3].parse::<usize>() else {
                            continue;
                        };

                        manager.add_node_level(id, level, None);
                        manager.add_edge(id, true_branch, manager_ref.clone());
                        manager.add_edge(id, false_branch, manager_ref.clone());

                        if level > max_level {
                            max_level = level;
                        }
                        defined.insert(id);
                        referenced.insert(false_branch);
                        referenced.insert(true_branch);
                        root = Some(id);
                    }
                }
            }

            let terminals = referenced
                .difference(&defined)
                .sorted()
                .zip_longest(vec!["F", "T"])
                .filter_map(|id_and_name| match id_and_name {
                    EitherOrBoth::Both(&id, name) => {
                        manager.add_node_level(id, max_level + 1, Some(name.to_string()));
                        Some((
                            name.to_string(),
                            DummyBDDEdge::new(Arc::new(id), manager_ref.clone()),
                        ))
                    }
                    EitherOrBoth::Left(&id) => {
                        let name = format!("{}", id);
                        manager.add_node_level(id, max_level + 1, Some(name.clone()));
                        Some((name, DummyBDDEdge::new(Arc::new(id), manager_ref.clone())))
                    }
                    EitherOrBoth::Right(_) => None,
                })
                .collect();
            manager.init_terminals(terminals);

            (
                root.map(|root| {
                    (
                        DummyBDDFunction(DummyBDDEdge::new(Arc::new(root), manager_ref.clone())),
                        vec!["f".to_string()],
                    )
                })
                .into_iter()
                .collect(),
                variables,
            )
        })
    }
}

unsafe impl Function for DummyBDDFunction {
    type Manager<'id> = DummyBDDManager;

    type ManagerRef = DummyBDDManagerRef;
    fn from_edge<'id>(
        manager: &Self::Manager<'id>,
        edge: oxidd_core::function::EdgeOfFunc<'id, Self>,
    ) -> Self {
        DummyBDDFunction(edge)
    }

    fn as_edge<'id>(
        &self,
        manager: &Self::Manager<'id>,
    ) -> &oxidd_core::function::EdgeOfFunc<'id, Self> {
        &self.0
    }

    fn into_edge<'id>(
        self,
        manager: &Self::Manager<'id>,
    ) -> oxidd_core::function::EdgeOfFunc<'id, Self> {
        self.0
    }

    fn manager_ref(&self) -> Self::ManagerRef {
        todo!()
    }

    fn with_manager_shared<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(&Self::Manager<'id>, &oxidd_core::function::EdgeOfFunc<'id, Self>) -> T,
    {
        self.0
             .1
            .with_manager_shared(|manager| f(manager, self.as_edge(manager)))
    }

    fn with_manager_exclusive<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(
            &mut Self::Manager<'id>,
            &oxidd_core::function::EdgeOfFunc<'id, Self>,
        ) -> T,
    {
        self.0
             .1
            .with_manager_exclusive(|manager| f(manager, self.as_edge(manager)))
    }
}

/// Simple dummy edge implementation based on [`Arc`]
///
/// The implementation is very limited but perfectly fine to test e.g. an apply
/// cache.
#[derive(Clone)]
pub struct DummyBDDEdge(Arc<NodeID>, DummyBDDManagerRef);

impl PartialEq for DummyBDDEdge {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DummyBDDEdge {}
impl PartialOrd for DummyBDDEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DummyBDDEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}
impl Hash for DummyBDDEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl Drop for DummyBDDEdge {
    fn drop(&mut self) {
        eprintln!(
            "Edges must not be dropped. Use Manager::drop_edge(). Backtrace:\n{}",
            std::backtrace::Backtrace::capture()
        );
    }
}

impl DummyBDDEdge {
    /// Create a new `DummyEdge`
    pub fn new(to: Arc<NodeID>, mr: DummyBDDManagerRef) -> Self {
        DummyBDDEdge(to, mr.clone())
    }
}

impl Edge for DummyBDDEdge {
    type Tag = ();

    fn borrowed(&self) -> Borrowed<'_, Self> {
        let ptr = Arc::as_ptr(&self.0);
        Borrowed::new(DummyBDDEdge(unsafe { Arc::from_raw(ptr) }, self.1.clone()))
    }
    fn with_tag(&self, _tag: ()) -> Borrowed<'_, Self> {
        let ptr = Arc::as_ptr(&self.0);
        Borrowed::new(DummyBDDEdge(unsafe { Arc::from_raw(ptr) }, self.1.clone()))
    }
    fn with_tag_owned(self, _tag: ()) -> Self {
        self
    }
    fn tag(&self) -> Self::Tag {}

    fn node_id(&self) -> NodeID {
        *self.0
    }
}

/// Dummy manager that does not actually manage anything. It is only useful to
/// clone and drop edges.
// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Clone, PartialEq, Eq)]
pub struct DummyBDDManager(
    BTreeMap<NodeID, DummyBDDNode>,
    HashMap<String, DummyBDDEdge>,
);
impl DummyBDDManager {
    pub fn new() -> DummyBDDManager {
        DummyBDDManager(BTreeMap::new(), HashMap::new())
    }
    fn init_terminals(&mut self, terminals: HashMap<String, DummyBDDEdge>) {
        self.1.extend(terminals);
    }
}
impl Hash for DummyBDDManager {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Dummy diagram rules
pub struct DummyBDDRules;
impl DiagramRules<DummyBDDEdge, DummyBDDNode, String> for DummyBDDRules {
    // type Cofactors<'a> = Iter<'a, Borrowed<'a, DummyEdge>>;
    type Cofactors<'a>
        = <DummyBDDNode as InnerNode<DummyBDDEdge>>::ChildrenIter<'a>
    where
        DummyBDDNode: 'a,
        DummyBDDEdge: 'a;

    fn reduce<M>(
        _manager: &M,
        level: LevelNo,
        children: impl IntoIterator<Item = DummyBDDEdge>,
    ) -> ReducedOrNew<DummyBDDEdge, DummyBDDNode>
    where
        M: Manager<Edge = DummyBDDEdge, InnerNode = DummyBDDNode>,
    {
        ReducedOrNew::New(DummyBDDNode::new(level, children), ())
    }

    fn cofactors(_tag: (), node: &DummyBDDNode) -> Self::Cofactors<'_> {
        node.children()
    }
}

impl DummyBDDManager {
    fn add_node_level(
        &mut self,
        from: NodeID,
        level: LevelNo,
        terminal: Option<String>,
    ) -> &mut DummyBDDNode {
        self.0.entry(from).or_insert_with(|| {
            if terminal.is_some() {
                DummyBDDNode(level, Vec::new(), terminal)
            } else {
                DummyBDDNode::new(level, Vec::new())
            }
        })
    }
    fn add_node(&mut self, from: NodeID) -> &mut DummyBDDNode {
        self.add_node_level(from, from.try_into().unwrap(), None)
    }
    fn add_edge(&mut self, from: NodeID, to: NodeID, mr: DummyBDDManagerRef) {
        let from_children = &mut self.0.get_mut(&from).unwrap().1;
        let edge = DummyBDDEdge::new(Arc::new(to), mr);
        from_children.push(edge);
    }
    fn has_edges(&self, node: NodeID) -> bool {
        let from_children = &self.0.get(&node).unwrap().1;
        from_children.len() > 0
    }
}

unsafe impl Manager for DummyBDDManager {
    type Edge = DummyBDDEdge;
    type EdgeTag = ();
    type InnerNode = DummyBDDNode;
    type Terminal = String;
    type TerminalRef<'a> = &'a String;
    type TerminalIterator<'a>
        = Cloned<std::collections::hash_map::Values<'a, String, DummyBDDEdge>>
    where
        Self: 'a;
    type Rules = DummyBDDRules;
    type NodeSet = HashSet<NodeID>;
    type LevelView<'a>
        = DummyBDDLevelView
    where
        Self: 'a;
    type LevelIterator<'a>
        = std::iter::Empty<DummyBDDLevelView>
    where
        Self: 'a;

    fn get_node(&self, edge: &Self::Edge) -> Node<Self> {
        let to_node = self
            .0
            .get(&*edge.0)
            .expect("Edge should refer to defined node");
        if let Some(terminal) = &to_node.2 {
            Node::Terminal(terminal)
        } else {
            Node::Inner(to_node)
        }
    }

    fn clone_edge(&self, edge: &Self::Edge) -> Self::Edge {
        DummyBDDEdge(edge.0.clone(), edge.1.clone())
    }

    fn drop_edge(&self, edge: Self::Edge) {
        // Move the inner arc out. We need to use `std::ptr::read` since
        // `DummyEdge` implements `Drop` (to print an error).
        let inner = unsafe { std::ptr::read(&edge.0) };
        std::mem::forget(edge);
        drop(inner);
    }

    fn num_inner_nodes(&self) -> usize {
        0
    }

    fn num_levels(&self) -> LevelNo {
        0
    }

    fn add_level(
        &mut self,
        _f: impl FnOnce(LevelNo) -> Self::InnerNode,
    ) -> AllocResult<Self::Edge> {
        unimplemented!()
    }

    fn level(&self, _no: LevelNo) -> Self::LevelView<'_> {
        panic!("out of range")
    }

    fn levels(&self) -> Self::LevelIterator<'_> {
        std::iter::empty()
    }

    fn get_terminal(&self, terminal: Self::Terminal) -> AllocResult<Self::Edge> {
        if let Some(terminal) = self.1.get(&terminal) {
            AllocResult::Ok(terminal.clone())
        } else {
            AllocResult::Err(OutOfMemory)
        }
    }

    fn num_terminals(&self) -> usize {
        self.1.len()
    }

    fn terminals(&self) -> Self::TerminalIterator<'_> {
        self.1.values().into_iter().cloned()
    }

    fn gc(&self) -> usize {
        0
    }

    fn reorder<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        f(self)
    }

    fn reorder_count(&self) -> u64 {
        0
    }
}

/// Dummy level view (not constructible)
pub struct DummyBDDLevelView;

unsafe impl LevelView<DummyBDDEdge, DummyBDDNode> for DummyBDDLevelView {
    type Iterator<'a>
        = std::iter::Empty<&'a DummyBDDEdge>
    where
        Self: 'a,
        DummyBDDEdge: 'a;

    type Taken = Self;

    fn len(&self) -> usize {
        unreachable!()
    }

    fn level_no(&self) -> LevelNo {
        unreachable!()
    }

    fn reserve(&mut self, _additional: usize) {
        unreachable!()
    }

    fn get(&self, _node: &DummyBDDNode) -> Option<&DummyBDDEdge> {
        unreachable!()
    }

    fn insert(&mut self, _edge: DummyBDDEdge) -> bool {
        unreachable!()
    }

    fn get_or_insert(&mut self, _node: DummyBDDNode) -> AllocResult<DummyBDDEdge> {
        unreachable!()
    }

    unsafe fn gc(&mut self) {
        unreachable!()
    }

    unsafe fn remove(&mut self, _node: &DummyBDDNode) -> bool {
        unreachable!()
    }

    unsafe fn swap(&mut self, _other: &mut Self) {
        unreachable!()
    }

    fn iter(&self) -> Self::Iterator<'_> {
        unreachable!()
    }

    fn take(&mut self) -> Self::Taken {
        unreachable!()
    }
}

/// Dummy node
#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct DummyBDDNode(LevelNo, Vec<DummyBDDEdge>, Option<String>);

impl DropWith<DummyBDDEdge> for DummyBDDNode {
    fn drop_with(self, _drop_edge: impl Fn(DummyBDDEdge)) {
        unimplemented!()
    }
}

unsafe impl HasLevel for DummyBDDNode {
    fn level(&self) -> LevelNo {
        self.0
    }

    unsafe fn set_level(&self, _level: LevelNo) {
        unimplemented!()
    }
}

impl InnerNode<DummyBDDEdge> for DummyBDDNode {
    const ARITY: usize = 0;

    // type ChildrenIter<'a> = std::iter::Empty<Borrowed<'a, DummyEdge>>
    // where
    //     Self: 'a;
    type ChildrenIter<'a>
        = BorrowedEdgeIter<'a, DummyBDDEdge, Iter<'a, DummyBDDEdge>>
    where
        Self: 'a;

    fn new(level: LevelNo, children: impl IntoIterator<Item = DummyBDDEdge>) -> Self {
        DummyBDDNode(level, children.into_iter().collect(), None)
    }

    fn check_level(&self, _check: impl FnOnce(LevelNo) -> bool) -> bool {
        true
    }

    fn children(&self) -> Self::ChildrenIter<'_> {
        BorrowedEdgeIter::from(self.1.iter())
    }

    fn child(&self, _n: usize) -> Borrowed<DummyBDDEdge> {
        unimplemented!()
    }

    unsafe fn set_child(&self, _n: usize, _child: DummyBDDEdge) -> DummyBDDEdge {
        unimplemented!()
    }

    fn ref_count(&self) -> usize {
        unimplemented!()
    }
}

/// Assert that the reference counts of edges match
///
/// # Example
///
/// ```
/// # use oxidd_core::{Edge, Manager};
/// # use oxidd_test_utils::assert_ref_counts;
/// # use oxidd_test_utils::edge::{DummyEdge, DummyManager};
/// let e1 = DummyEdge::new();
/// let e2 = DummyManager.clone_edge(&e1);
/// let e3 = DummyEdge::new();
/// assert_ref_counts!(e1, e2 = 2; e3 = 1);
/// # DummyManager.drop_edge(e1);
/// # DummyManager.drop_edge(e2);
/// # DummyManager.drop_edge(e3);
/// ```
#[macro_export]
macro_rules! assert_ref_counts {
    ($edge:ident = $count:literal) => {
        assert_eq!($edge.ref_count(), $count);
    };
    ($edge:ident, $($edges:ident),+ = $count:literal) => {
        assert_ref_counts!($edge = $count);
        assert_ref_counts!($($edges),+ = $count);
    };
    // spell-checker:ignore edgess
    ($($edges:ident),+ = $count:literal; $($($edgess:ident),+ = $counts:literal);+) => {
        assert_ref_counts!($($edges),+ = $count);
        assert_ref_counts!($($($edgess),+ = $counts);+);
    };
}
