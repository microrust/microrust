use std::sync::{
    Once,
    atomic::{AtomicBool, Ordering},
};

use tracing::debug;

use crate::inject::{
    structs::{__DepGraph, __STRUCT_DEPS, __STRUCT_REGISTRY},
    traits::{__TRAIT_COLLISIONS, __TRAIT_REGISTRY},
};

/// Panics with every detected cycle, used by both [`report_registrations`] and
/// the first-resolution guard so the message is identical either way.
fn __fail_on_cycles() -> Vec<Vec<&'static str>> {
    let cycles = dependency_cycles();
    if cycles.is_empty() {
        return cycles;
    }

    let rendered: Vec<String> = cycles
        .iter()
        .map(|cycle| cycle.join(" -> "))
        .collect();

    panic!(
        "[microrust] found {} dependency cycle(s) in the injection graph: {}",
        rendered.len(),
        rendered.join("; ")
    );
}

/// Panics with every ambiguous trait binding.
///
/// Mirrors the cycle check: the clash is recorded during `ctor` registration and
/// raised here, because nothing observed at registration time would be seen.
fn __fail_on_trait_collisions() {
    // Rendered inside a scope so the guard is gone before the panic: panicking
    // while holding it would poison the mutex, and every later report would
    // fail with `PoisonError` instead of naming the clash.
    let rendered: Vec<String> = {
        let collisions = __TRAIT_COLLISIONS.lock().unwrap();
        collisions
            .iter()
            .map(|collision| {
                format!(
                    "{} is implemented by both {} and {}",
                    collision.trait_name, collision.first_impl, collision.second_impl,
                )
            })
            .collect()
    };
    if rendered.is_empty() {
        return;
    }

    panic!(
        "[microrust] found {} ambiguous trait binding(s): {}",
        rendered.len(),
        rendered.join("; "),
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

fn __find_cycles(graph: &__DepGraph) -> Vec<Vec<&'static str>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Visit {
        New,
        OnPath,
        Done,
    }

    let mut visit = vec![Visit::New; graph.node_count()];
    let mut cycles: Vec<Vec<&'static str>> = Vec::new();

    // Iterative rather than recursive, so a deep dependency chain cannot
    // overflow the stack. Each frame is a node on the current path plus the
    // position of the next dependency to explore from it.
    for root in 0..graph.node_count() {
        if visit[root] != Visit::New {
            continue;
        }
        visit[root] = Visit::OnPath;
        let mut path: Vec<(usize, usize)> = vec![(root, 0)];

        while let Some((node, next)) = path.last_mut() {
            let node = *node;
            let Some(&dep) = graph.deps(node).get(*next) else {
                visit[node] = Visit::Done;
                path.pop();
                continue;
            };
            *next += 1;

            match visit[dep] {
                Visit::New => {
                    visit[dep] = Visit::OnPath;
                    path.push((dep, 0));
                }
                // An edge back onto the current path closes a cycle:
                // `dep ..= node` is the cycle, and the closing hop is `dep`
                // again. The scan keeps going afterwards, so every cyclic
                // region is reported instead of only the first.
                Visit::OnPath => {
                    if let Some(start) = path.iter().position(|(on_path, _)| *on_path == dep) {
                        let mut cycle: Vec<&'static str> = path[start..]
                            .iter()
                            .map(|(on_path, _)| graph.name(*on_path))
                            .collect();
                        cycle.push(graph.name(dep));
                        cycles.push(cycle);
                    }
                }
                // Fully explored with no path back here, or its cycle was
                // already reported when it was on the path.
                Visit::Done => {}
            }
        }
    }

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
/// Panics if the injection graph contains a dependency cycle, or if a trait is
/// implemented by more than one struct. Both are static wiring errors rather
/// than runtime conditions. A cycle would re-enter a `OnceLock` that is still
/// initializing and deadlock rather than fail; an ambiguous trait silently
/// resolves to whichever struct's `ctor` ran first, and `ctor` order across
/// crates is unspecified, so the winner is not even stable between builds. Both
/// are therefore reported up front with the offending types named.
///
/// A cycle is additionally caught on the first `get_instance()` in a process,
/// since that would otherwise deadlock. Ambiguous traits are not, so an
/// application that never calls this function can still resolve one silently.
pub fn report_registrations() {
    let structs = __STRUCT_REGISTRY.lock().unwrap();
    let mut struct_names: Vec<&'static str> = structs.values().map(|entry| entry.name).collect();
    struct_names.sort_unstable();
    let struct_count = struct_names.len();
    drop(structs);
    let edge_count = __STRUCT_DEPS.lock().unwrap().edge_count();

    let traits = __TRAIT_REGISTRY.lock().unwrap();
    let mut trait_names: Vec<&'static str> = traits.values().map(|entry| entry.name).collect();
    trait_names.sort_unstable();
    let trait_count = trait_names.len();
    drop(traits);

    debug!("[microrust] registered {struct_count} struct(s)");
    for name in struct_names {
        debug!("[microrust]   struct    {name}");
    }

    debug!("[microrust] registered {trait_count} trait binding(s)");
    for name in trait_names {
        debug!("[microrust]   trait     {name}");
    }

    debug!("[microrust] {edge_count} dependency edge(s)");

    __fail_on_cycles();
    __fail_on_trait_collisions();
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
        __fail_on_cycles();
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::__find_cycles;
    use crate::inject::structs::{__DepGraph, __DepNode};

    // Distinct types give distinct `TypeId`s; the graph only ever sees those
    // and the names, so marker structs stand in for real singletons.
    struct A;
    struct B;
    struct C;
    struct D;

    fn node<T: 'static>(name: &'static str) -> __DepNode {
        __DepNode {
            type_id: TypeId::of::<T>(),
            name,
        }
    }

    fn graph(edges: &[(__DepNode, __DepNode)]) -> __DepGraph {
        let mut graph = __DepGraph::default();
        for (from, to) in edges {
            graph.add_edge(*from, *to);
        }
        graph
    }

    #[test]
    fn diamond_is_not_a_cycle() {
        // A -> B -> D and A -> C -> D: D is reached twice, but never while it
        // is still on the path.
        let graph = graph(&[
            (node::<A>("A"), node::<B>("B")),
            (node::<A>("A"), node::<C>("C")),
            (node::<B>("B"), node::<D>("D")),
            (node::<C>("C"), node::<D>("D")),
        ]);
        assert!(__find_cycles(&graph).is_empty());
    }

    #[test]
    fn cycle_closing_at_the_root_is_found() {
        let graph = graph(&[
            (node::<A>("A"), node::<B>("B")),
            (node::<B>("B"), node::<C>("C")),
            (node::<C>("C"), node::<A>("A")),
        ]);
        assert_eq!(__find_cycles(&graph), vec![vec!["A", "B", "C", "A"]]);
    }

    #[test]
    fn self_dependency_is_a_two_element_path() {
        let graph = graph(&[(node::<A>("A"), node::<A>("A"))]);
        assert_eq!(__find_cycles(&graph), vec![vec!["A", "A"]]);
    }

    #[test]
    fn repeated_edge_reports_its_cycle_once() {
        let graph = graph(&[
            (node::<A>("A"), node::<B>("B")),
            (node::<A>("A"), node::<B>("B")),
            (node::<B>("B"), node::<A>("A")),
        ]);
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(__find_cycles(&graph), vec![vec!["A", "B", "A"]]);
    }

    #[test]
    fn disjoint_cycles_are_all_reported_in_insertion_order() {
        let graph = graph(&[
            (node::<A>("A"), node::<B>("B")),
            (node::<B>("B"), node::<A>("A")),
            (node::<C>("C"), node::<D>("D")),
            (node::<D>("D"), node::<C>("C")),
        ]);
        assert_eq!(
            __find_cycles(&graph),
            vec![vec!["A", "B", "A"], vec!["C", "D", "C"]]
        );
    }
}
