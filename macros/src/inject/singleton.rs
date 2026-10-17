use crate::error::macro_error;
use crate::inject::dep::{DepError, classify_dep};
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{FnArg, ItemImpl, Pat, parse_macro_input};

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
    // Registration runs in a `ctor`, which needs one concrete type to take a
    // `TypeId` of. A generic impl has none, and letting it through surfaces as
    // unrelated errors about unbound type parameters in the generated code.
    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return macro_error(
            "#[singleton] cannot be applied to a generic impl; \
             implement it for a concrete type, e.g. `impl Wrapper<u32>`",
        );
    }

    // Keep the whole `self_ty` path for codegen: emitting only the final
    // segment loses the module qualification and breaks when the type is not
    // in scope under its bare name.
    let syn::Type::Path(self_path) = &*input.self_ty else {
        return macro_error("Expecting: impl <type>");
    };
    let Some(last_segment) = self_path.path.segments.last() else {
        return macro_error("Expecting: impl <type>");
    };
    // Used only to name the type in diagnostics.
    let struct_ident = last_segment.ident.clone();

    let constructor_fn = input.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(func) if func.sig.ident == constructor_ident => Some(func),
        _ => None,
    });
    let Some(constructor_fn) = constructor_fn else {
        return macro_error(&format!(
            "constructor method `{constructor_ident}` not found in `impl {struct_ident}`"
        ));
    };
    // Caught by the `Arc<Self>` annotation below as well, but as a type
    // mismatch against an opaque future; naming the actual mistake is clearer.
    if constructor_fn.sig.asyncness.is_some() {
        return macro_error(&format!(
            "`{struct_ident}::{constructor_ident}` is `async`; singleton constructors \
             must be synchronous, since resolution happens outside any runtime"
        ));
    }

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
        let ty = &pat_type.ty;
        let dep = match classify_dep(ty) {
            Ok(dep) => dep,
            Err(DepError::Unsupported) => {
                return macro_error(&format!(
                    "unsupported constructor parameter `{param}: {}` in \
                     `{struct_ident}::{constructor_ident}`; expected `Arc<Struct>` or `Arc<dyn Trait>`",
                    quote!(#ty)
                ));
            }
            Err(DepError::MultipleBounds) => {
                return macro_error(&format!(
                    "unsupported constructor parameter `{param}: {}` in \
                     `{struct_ident}::{constructor_ident}`; a trait dependency must name \
                     exactly one trait, as `Arc<dyn Trait>`. Declare `Send + Sync` as \
                     supertraits of the trait instead of adding them here",
                    quote!(#ty)
                ));
            }
        };
        deps.push(dep);
    }

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

    let collect_deps: Vec<TokenStream2> = deps
        .iter()
        .map(|dep| {
            let ty = &dep.ty;
            if dep.is_trait {
                quote! { ::microrust::inject::__collect_trait_deps::<#self_path, #ty>(); }
            } else {
                quote! { ::microrust::inject::__collect_struct_deps::<#self_path, #ty>(); }
            }
        })
        .collect();

    let tokens = quote! {
    #input

    impl ::microrust::inject::__InstanceFactory for #self_path {
        fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
            ::microrust::__private::tracing::debug!(
                "__InstanceFactory::__create_instance() called for {}",
                ::std::stringify!(#struct_ident)
            );

            #(#resolve_args)*

            // The annotation is load-bearing: without it, a constructor that
            // returns some other type still coerces to `Arc<dyn Any>` and only
            // fails at the `downcast` in `get_instance`, long after compile time.
            // `<..>` makes the path valid in expression position for a type
            // with generic arguments: `Wrapper<u32>::new` would parse as a
            // chain of comparisons.
            let inst: ::std::sync::Arc<#self_path> =
                ::std::sync::Arc::new(<#self_path>::#constructor_ident(#(#ctor_args),*));
            let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
            inst_any
        }
    }

    // The anonymous `const` gives the `ctor` fn a scope of its own, so its name
    // never enters the caller's module. Deriving the name from the type instead
    // collided for same-named types in different modules.
    const _: () = {
        ::microrust::ctor_parse!(#[ctor(unsafe)] fn __microrust_register() {
            ::microrust::inject::__collect_struct::<#self_path>();
            #(#collect_deps)*
        });
    };
    };
    TokenStream::from(tokens)
}
