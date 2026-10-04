use syn::{GenericArgument, PathArguments, Type, TypeParamBound};

/// A single dependency, already stripped of its `Arc<..>` wrapper.
pub struct Dep {
    /// The injected type, e.g. `MyStruct` or `dyn MyTrait`.
    pub ty: Type,
    /// Whether the dependency is a trait object and must go through
    /// `get_trait_instance` rather than `get_instance`.
    pub is_trait: bool,
}

/// Unwraps `Arc<Inner>` and reports whether `Inner` is a trait object.
///
/// Accepts `Arc`, `std::sync::Arc` and `core::sync::Arc` alike, since only the
/// final path segment is inspected.
pub fn classify_dep(ty: &Type) -> Option<Dep> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let outer = type_path.path.segments.last()?;
    if outer.ident != "Arc" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &outer.arguments else {
        return None;
    };
    let GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };

    match inner {
        Type::Path(_) => Some(Dep {
            ty: inner.clone(),
            is_trait: false,
        }),
        Type::TraitObject(obj) => {
            // Require exactly one non-auto-trait bound so the emitted turbofish
            // is always a well-formed `dyn Trait`.
            let mut traits = obj.bounds.iter().filter_map(|bound| match bound {
                TypeParamBound::Trait(trait_bound) => Some(&trait_bound.path),
                _ => None,
            });
            let trait_path = traits.next()?;
            if traits.next().is_some() || trait_path.segments.is_empty() {
                return None;
            }
            Some(Dep {
                ty: inner.clone(),
                is_trait: true,
            })
        }
        _ => None,
    }
}
