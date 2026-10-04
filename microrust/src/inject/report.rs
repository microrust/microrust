use std::sync::{
    Once,
    atomic::{AtomicBool, Ordering},
};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{DfsEvent, IntoNodeIdentifiers, depth_first_search},
};
use tracing::debug;

use crate::inject::{
    function::__FUNCTION_REGISTRY,
    structs::{__DepNode, __STRUCT_DEPS, __STRUCT_REGISTRY},
    traits::__TRAIT_REGISTRY,
};

/// Panics with every detected cycle, used by both [`report_registrations`] and
/// the first-resolution guard so the message is identical either way.
fn __fail_on_cycles(context: &str) -> Vec<Vec<&'static str>> {
    let cycles = dependency_cycles();
    if cycles.is_empty() {
        return cycles;
    }

    let rendered: Vec<String> = cycles
        .iter()
        .map(|cycle| cycle.join(" -> "))
        .collect();

    panic!(
        "[microrust] {} dependency cycle(s) detected in the injection graph: {}",
        context,
        rendered.join("; ")
    );
}

/// Detects dependency cycles among registered types.
///
/// Returns one witness path per cyclic region, e.g. `["A", "B", "A"]`. This is
/// a witness set rather than an exhaustive enumeration of all simple cycles,
/// which is enough to point at the offending wiring.
///
/// Self-dependencies are reported as a two-element path (`A -> A`).
///
/// Traverses in `O(|V| + |E|)`, so it is cheap to call from a hot path but is
/// only called once per process by the crate itself.
pub fn dependency_cycles() -> Vec<Vec<&'static str>> {
    let graph = __STRUCT_DEPS.lock().unwrap();
    __find_cycles(&graph)
}

fn __find_cycles(graph: &DiGraph<__DepNode, ()>) -> Vec<Vec<&'static str>> {
    let mut cycles: Vec<Vec<&'static str>> = Vec::new();
    let mut path: Vec<NodeIndex> = Vec::new();

    // Returning `Ok(())` always lets the traversal continue past the first back
    // edge, so every cycle is reported instead of only the earliest one.
    let _: Result<(), ()> = depth_first_search(graph, graph.node_identifiers(), |event| {
        match event {
            // The stack tracks `Discover`/`Finish` rather than `TreeEdge`
            // because the traversal root is never the target of a `TreeEdge`.
            // Pushing only tree edges would leave the root off the stack and
            // silently miss the most common case, a cycle closing at the root.
            DfsEvent::Discover(node, _) => path.push(node),
            // `to` is an ancestor of `from` on the current DFS stack, so
            // `to ..= from` is the cycle, and the closing hop is `to` again.
            DfsEvent::BackEdge(_, to) => {
                if let Some(start) = path.iter().position(|node| *node == to) {
                    let mut cycle: Vec<&'static str> =
                        path[start..].iter().map(|node| graph[*node].name).collect();
                    cycle.push(graph[to].name);
                    cycles.push(cycle);
                }
            }
            DfsEvent::Finish(_, _) => {
                path.pop();
            }
            _ => {}
        }
        Ok(())
    });

    cycles
}

/// Emits what was registered, once a `tracing` subscriber is listening.
///
/// Registries are populated from `#[ctor]` functions, which run in `.init_array`
/// before `main`. At that point no subscriber exists: `tracing`'s global max
/// level starts at `OFF` and the global dispatch starts as `NO_SUBSCRIBER`, so
/// any event emitted from a `ctor` is discarded before its arguments are even
/// formatted. Registration therefore stays silent and the diagnostics are
/// emitted here instead.
///
/// Call this after installing the subscriber:
///
/// ```no_run
/// microrust::inject::report_registrations();
/// ```
///
/// # Panics
///
/// Panics if the injection graph contains a dependency cycle. A cycle is a
/// static wiring error, not a runtime condition: resolving it would re-enter a
/// `OnceLock` that is still initializing and deadlock rather than fail, so it
/// is reported up front with the offending types named.
pub fn report_registrations() {
    let structs = __STRUCT_REGISTRY.lock().unwrap();
    let mut struct_names: Vec<&'static str> = structs.values().map(|entry| entry.name).collect();
    struct_names.sort_unstable();
    let struct_count = struct_names.len();
    let edge_count = __STRUCT_DEPS.lock().unwrap().edge_count();
    drop(structs);

    let traits = __TRAIT_REGISTRY.lock().unwrap();
    let mut trait_names: Vec<&'static str> = traits.values().map(|entry| entry.name).collect();
    trait_names.sort_unstable();
    let trait_count = trait_names.len();
    drop(traits);

    let functions = __FUNCTION_REGISTRY.lock().unwrap();
    let mut function_names: Vec<(String, &'static str)> = functions
        .iter()
        .map(|(key, entry)| (key.clone(), entry.sig_name))
        .collect();
    function_names.sort_unstable();
    let function_count = function_names.len();
    drop(functions);

    debug!("[microrust] registered {struct_count} struct(s)");
    for name in struct_names {
        debug!("[microrust]   struct    {name}");
    }

    debug!("[microrust] registered {trait_count} trait binding(s)");
    for name in trait_names {
        debug!("[microrust]   trait     {name}");
    }

    debug!("[microrust] registered {function_count} injectable function(s)");
    for (key, sig_name) in function_names {
        debug!("[microrust]   fn        {key} :: {sig_name}");
    }

    debug!("[microrust] {edge_count} dependency edge(s)");

    __fail_on_cycles("found");
}

/// First-resolution guard against an undeclared cycle.
///
/// [`report_registrations`] is opt-in, so an application that never calls it
/// would otherwise discover a cycle the slow way: a `OnceLock` re-entrancy
/// deadlock on the first `get_instance` that reaches back into a type still
/// being constructed. One graph traversal per process is cheap insurance.
pub(crate) fn __assert_no_cycles() {
    static ONCE: Once = Once::new();
    static CYCLIC: AtomicBool = AtomicBool::new(false);

    // The `Once` body deliberately does not panic: a panic there would poison
    // the cell, and every later caller would abort with "Once instance has
    // previously been poisoned" instead of the cycle report. Recording the
    // verdict and raising it out here keeps the message identical for all
    // callers regardless of which one gets there first.
    ONCE.call_once(|| {
        CYCLIC.store(!dependency_cycles().is_empty(), Ordering::Relaxed);
    });

    if CYCLIC.load(Ordering::Relaxed) {
        __fail_on_cycles("found");
    }
}