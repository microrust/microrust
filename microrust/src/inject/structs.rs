use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use tracing::debug;

#[doc(hidden)]
pub type __AnyArc = Arc<dyn Any + Send + Sync>;
#[doc(hidden)]
pub type __FactoryFn = fn() -> __AnyArc;

/// A registered struct factory plus the name to report it under.
///
/// The name is captured at registration time because by the time anything is
/// logged the concrete type is long gone behind a `TypeId`.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct __FactoryEntry {
    pub name: &'static str,
    pub factory: __FactoryFn,
}

#[doc(hidden)]
pub type __FactoryMap = HashMap<TypeId, __FactoryEntry>;

/// A node in the struct dependency graph.
///
/// Carries `type_id` for identity and `name` for diagnostics; the graph is only
/// ever traversed for reporting and cycle detection, both of which need to name
/// the types involved.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct __DepNode {
    pub type_id: TypeId,
    pub name: &'static str,
}

pub(crate) static __STRUCT_REGISTRY: LazyLock<Mutex<__FactoryMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The dependency graph: one node per type, edges as adjacency lists.
///
/// Nodes live in a `Vec` in insertion order, with a `TypeId` index beside it.
/// The index makes adding an edge O(1); the `Vec` keeps traversal order
/// deterministic, so cycle reports list paths in the same order on every run,
/// which iterating a `HashMap` would not.
#[derive(Default)]
pub(crate) struct __DepGraph {
    index: HashMap<TypeId, usize>,
    nodes: Vec<__DepGraphNode>,
}

struct __DepGraphNode {
    name: &'static str,
    deps: Vec<usize>,
}

impl __DepGraph {
    /// Adds a `from -> to` edge, creating either node on first sight.
    ///
    /// A repeated edge (a constructor taking the same dependency twice) is
    /// dropped: it adds no reachability, and a parallel edge into a cycle
    /// would have that cycle reported twice.
    pub(crate) fn add_edge(&mut self, from: __DepNode, to: __DepNode) {
        let from = self.node(from);
        let to = self.node(to);
        let deps = &mut self.nodes[from].deps;
        if !deps.contains(&to) {
            deps.push(to);
        }
    }

    fn node(&mut self, node: __DepNode) -> usize {
        *self.index.entry(node.type_id).or_insert_with(|| {
            self.nodes.push(__DepGraphNode {
                name: node.name,
                deps: Vec::new(),
            });
            self.nodes.len() - 1
        })
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.nodes.iter().map(|node| node.deps.len()).sum()
    }

    pub(crate) fn name(&self, node: usize) -> &'static str {
        self.nodes[node].name
    }

    pub(crate) fn deps(&self, node: usize) -> &[usize] {
        &self.nodes[node].deps
    }
}

pub(crate) static __STRUCT_DEPS: LazyLock<Mutex<__DepGraph>> =
    LazyLock::new(|| Mutex::new(__DepGraph::default()));

#[doc(hidden)]
pub trait __InstanceFactory: Send + Sync {
    fn __create_instance() -> __AnyArc
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
pub fn __collect_struct<T: __InstanceFactory + Send + Sync + 'static>() {
    debug!("__collect_struct() called for {}", type_name::<T>());

    let type_id_struct = TypeId::of::<T>();
    debug!("Struct TypeName: {:#}", type_name::<T>());
    debug!("Struct TypeId: {:?}", type_id_struct);

    if __contains_function_pointer(&type_id_struct) {
        debug!("struct already collected");
        return;
    }

    __insert_function_pointer(
        type_id_struct,
        __FactoryEntry {
            name: type_name::<T>(),
            factory: T::__create_instance,
        },
    );
}

#[doc(hidden)]
pub fn __collect_struct_deps<
    T: __InstanceFactory + Send + Sync + 'static,
    U: __InstanceFactory + Send + Sync + 'static,
>() {
    debug!("__collect_struct_deps() called: {} -> {}", type_name::<T>(), type_name::<U>());

    __add_dep(
        __DepNode {
            type_id: TypeId::of::<T>(),
            name: type_name::<T>(),
        },
        __DepNode {
            type_id: TypeId::of::<U>(),
            name: type_name::<U>(),
        },
    );
}

/// Records a dependency edge onto a trait object.
///
/// Separate from [`__collect_struct_deps`] because a trait object has no
/// `__InstanceFactory` impl; the edge targets `TypeId::of::<dyn Trait>()`
/// directly rather than the concrete type that happens to implement it.
#[doc(hidden)]
pub fn __collect_trait_deps<T: Send + Sync + 'static, U: Send + Sync + ?Sized + 'static>() {
    debug!("__collect_trait_deps() called: {} -> {}", type_name::<T>(), type_name::<U>());

    __add_dep(
        __DepNode {
            type_id: TypeId::of::<T>(),
            name: type_name::<T>(),
        },
        __DepNode {
            type_id: TypeId::of::<U>(),
            name: type_name::<U>(),
        },
    );
}

/// Records that trait `T` resolves to struct `S`.
///
/// Without this edge a trait object is a dead end in the graph: `A -> dyn T`
/// would be recorded, but not that resolving `dyn T` constructs `S`, so a cycle
/// closing through `S` back to `A` would go unreported and deadlock instead.
pub(crate) fn __collect_trait_binding<T: ?Sized + 'static, S: 'static>() {
    debug!("__collect_trait_binding() called: {} -> {}", type_name::<T>(), type_name::<S>());

    __add_dep(
        __DepNode {
            type_id: TypeId::of::<T>(),
            name: type_name::<T>(),
        },
        __DepNode {
            type_id: TypeId::of::<S>(),
            name: type_name::<S>(),
        },
    );
}

fn __add_dep(from: __DepNode, to: __DepNode) {
    __STRUCT_DEPS.lock().unwrap().add_edge(from, to);
}

#[doc(hidden)]
pub fn __factory_entry(type_id: TypeId) -> __FactoryEntry {
    debug!("__factory_entry() called");

    // Copied out before unwrapping so a miss panics without the guard held,
    // which would otherwise poison the registry for every later resolution.
    let entry = __STRUCT_REGISTRY.lock().unwrap().get(&type_id).copied();
    entry.expect("no singleton is registered for this type")
}

#[doc(hidden)]
pub fn __contains_function_pointer(type_id: &TypeId) -> bool {
    let map = __STRUCT_REGISTRY.lock().unwrap();
    map.contains_key(type_id)
}

#[doc(hidden)]
pub fn __insert_function_pointer(type_id: TypeId, entry: __FactoryEntry) {
    let mut map = __STRUCT_REGISTRY.lock().unwrap();
    map.insert(type_id, entry);
}