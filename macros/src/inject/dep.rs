use syn::{GenericArgument, PathArguments, Type, TypeParamBound};

/// A single dependency, already stripped of its `Arc<..>` wrapper.
pub struct Dep {
    /// The injected type, e.g. `MyStruct` or `dyn MyTrait`.
    pub ty: Type,
    /// Whether the dependency is a trait object and must go through
    /// `get_trait_instance` rather than `get_instance`.
    pub is_trait: bool,
}

/// Why a constructor parameter cannot be injected.
pub enum DepError {
    /// Not of the shape `Arc<Struct>` or `Arc<dyn Trait>` at all.
    Unsupported,
    /// `Arc<dyn Trait + ...>` with more than one trait bound.
    MultipleBounds,
}

/// Unwraps `Arc<Inner>` and reports whether `Inner` is a trait object.
///
/// Accepts `Arc`, `std::sync::Arc` and `core::sync::Arc` alike, since only the
/// final path segment is inspected.
pub fn classify_dep(ty: &Type) -> Result<Dep, DepError> {
    let Type::Path(type_path) = ty else {
        return Err(DepError::Unsupported);
    };
    let outer = type_path
        .path
        .segments
        .last()
        .ok_or(DepError::Unsupported)?;
    if outer.ident != "Arc" {
        return Err(DepError::Unsupported);
    }
    let PathArguments::AngleBracketed(args) = &outer.arguments else {
        return Err(DepError::Unsupported);
    };
    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return Err(DepError::Unsupported);
    };

    match inner {
        Type::Path(_) => Ok(Dep {
            ty: inner.clone(),
            is_trait: false,
        }),
        Type::TraitObject(obj) => {
            // Exactly one trait bound, auto traits included. Trait bindings are
            // keyed by `TypeId::of::<dyn Trait>()`, and `dyn Trait + Send` is a
            // distinct type with its own `TypeId`, so accepting extra bounds
            // would compile and then miss the binding at runtime. Lifetime
            // bounds don't affect `TypeId`, so they are let through.
            let mut traits = obj.bounds.iter().filter_map(|bound| match bound {
                TypeParamBound::Trait(trait_bound) => Some(&trait_bound.path),
                _ => None,
            });
            let trait_path = traits.next().ok_or(DepError::Unsupported)?;
            if traits.next().is_some() {
                return Err(DepError::MultipleBounds);
            }
            if trait_path.segments.is_empty() {
                return Err(DepError::Unsupported);
            }
            Ok(Dep {
                ty: inner.clone(),
                is_trait: true,
            })
        }
        _ => Err(DepError::Unsupported),
    }
}
