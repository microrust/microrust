use crate::error::macro_error;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    FnArg, GenericArgument, ItemImpl, Pat, PathArguments, Type, TypeParamBound, parse_macro_input,
};

/// A single constructor dependency, already stripped of its `Arc<..>` wrapper.
struct Dep {
    /// The injected type, e.g. `MyStruct` or `dyn MyTrait`.
    ty: Type,
    /// Whether the dependency is a trait object and must go through
    /// `get_trait_instance` rather than `get_instance`.
    is_trait: bool,
}

/// Unwraps `Arc<Inner>` and reports whether `Inner` is a trait object.
///
/// Accepts `Arc`, `std::sync::Arc` and `core::sync::Arc` alike, since only the
/// final path segment is inspected.
fn classify_dep(ty: &Type) -> Option<Dep> {
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

pub fn inject_singleton_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if args.clone().into_iter().size_hint() != (1, Some(1)) {
        return macro_error("Please provide a constructor method like #[singleton(new)]");
    }
    let constructor_ident = parse_macro_input!(args as Ident);

    let input = parse_macro_input!(input as ItemImpl);
    if input.trait_.is_some() {
        return macro_error("Please use attribute macro #[inject_trait]");
    }

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => return macro_error("Expecting: impl <type>"),
    };

    let constructor_fn = input.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(func) if func.sig.ident == constructor_ident => Some(func),
        _ => None,
    });
    let Some(constructor_fn) = constructor_fn else {
        return macro_error(&format!(
            "constructor method `{constructor_ident}` not found in `impl {struct_ident}`"
        ));
    };

    let mut deps = Vec::new();
    for arg in &constructor_fn.sig.inputs {
        let FnArg::Typed(pat_type) = arg else {
            return macro_error(&format!(
                "`{struct_ident}::{constructor_ident}` takes a `self` parameter; \
                 constructors must be associated functions"
            ));
        };

        let param = match &*pat_type.pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            _ => "<binding>".to_string(),
        };
        let Some(dep) = classify_dep(&pat_type.ty) else {
            return macro_error(&format!(
                "unsupported constructor parameter `{param}: {}` in \
                 `{struct_ident}::{constructor_ident}`; expected `Arc<Struct>` or `Arc<dyn Trait>`",
                quote!(#pat_type.ty)
            ));
        };
        deps.push(dep);
    }

    let ctor_ident = format_ident!("ctor_{}", struct_ident);
    let ctor_deps_ident = format_ident!("ctor_deps_{}", struct_ident);

    // One `let argN` per dependency, resolved through the matching accessor.
    let resolve_args: Vec<TokenStream2> = deps
        .iter()
        .enumerate()
        .map(|(index, dep)| {
            let arg = format_ident!("arg{index}");
            let ty = &dep.ty;
            if dep.is_trait {
                quote! {
                    let #arg: ::std::sync::Arc<#ty> =
                        ::microrust::inject::get_trait_instance::<#ty>();
                }
            } else {
                quote! {
                    let #arg: ::std::sync::Arc<#ty> =
                        ::microrust::inject::get_instance::<#ty>();
                }
            }
        })
        .collect();

    let ctor_args: Vec<Ident> = (0..deps.len())
        .map(|index| format_ident!("arg{index}"))
        .collect();

    // Dependency edges are only recorded when there is something to record.
    let deps_ctor = if deps.is_empty() {
        TokenStream2::new()
    } else {
        let collect_deps: Vec<TokenStream2> = deps
            .iter()
            .map(|dep| {
                let ty = &dep.ty;
                if dep.is_trait {
                    quote! { ::microrust::inject::__collect_trait_deps::<#struct_ident, #ty>(); }
                } else {
                    quote! { ::microrust::inject::__collect_struct_deps::<#struct_ident, #ty>(); }
                }
            })
            .collect();

        quote! {
            ::microrust::ctor_parse!(#[ctor(unsafe)] #[allow(non_snake_case)] fn #ctor_deps_ident() {
                #(#collect_deps)*
            });
        }
    };

    let tokens = quote! {
    #input

    impl ::microrust::inject::__InstanceFactory for #struct_ident {
        fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
            ::microrust::__private::tracing::debug!(
                "__InstanceFactory::__create_instance() called for {}",
                ::std::stringify!(#struct_ident)
            );

            #(#resolve_args)*

            let inst = ::std::sync::Arc::new(#struct_ident::#constructor_ident(#(#ctor_args),*));
            let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
            inst_any
        }
    }

    ::microrust::ctor_parse!(#[ctor(unsafe)] #[allow(non_snake_case)] fn #ctor_ident() {::microrust::inject::__collect_struct::<#struct_ident>();});
    #deps_ctor
    };
    TokenStream::from(tokens)
}
