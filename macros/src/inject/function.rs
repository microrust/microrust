use crate::error::macro_error;
use crate::inject::dep::classify_dep;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    FnArg, ItemFn, Pat, PatType, ReturnType, parse_macro_input,
    punctuated::Punctuated, token::Comma,
};

/// Builds the ident triple shared by `inject_fn`, `get_function!` and
/// `set_fake_function!`, so all three agree on the generated names.
fn fn_idents(name: &Ident) -> (Ident, Ident, Ident) {
    (
        format_ident!("__MicrorustFn_{name}"),
        format_ident!("__microrust_boxed_{name}"),
        format_ident!("__microrust_ctor_fn_{name}"),
    )
}

/// Registry key: module-qualified so same-named functions in different modules
/// cannot collide.
fn fn_key(name: &Ident) -> TokenStream2 {
    quote! {
        ::std::concat!(::std::module_path!(), "::", ::std::stringify!(#name))
    }
}

/// Parameters without a trailing comma, for use inside `fn(..)` types.
fn params_tokens(inputs: &Punctuated<FnArg, Comma>) -> TokenStream2 {
    let args = inputs.iter();
    quote! { #(#args),* }
}

/// The boxed future type an `async fn`'s callable must return.
///
/// `impl Future` in return position cannot leave its defining module, so the
/// shim boxes it to make the signature nameable.
fn boxed_future_sig(output: &ReturnType) -> TokenStream2 {
    match output {
        ReturnType::Default => quote! { ::std::convert::Infallible },
        ReturnType::Type(_, ty) => quote! {
            ::std::pin::Pin<::std::boxed::Box<dyn ::core::future::Future<Output = #ty> + Send>>
        },
    }
}

pub fn inject_fn_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);
    let fn_name = input.sig.ident.clone();

    if input.sig.constness.is_some() || input.sig.abi.is_some() {
        return macro_error("`#[inject_fn]` supports only non-const, ABI-less functions");
    }
    if input.sig.variadic.is_some() {
        return macro_error("`#[inject_fn]` does not support variadic functions");
    }
    if !input.sig.generics.params.is_empty() {
        return macro_error("`#[inject_fn]` does not support generic functions");
    }

    let is_async = input.sig.asyncness.is_some();
    let mut bindings: Vec<TokenStream2> = Vec::new();
    let mut injected_args: Vec<Ident> = Vec::new();
    let mut kept: Punctuated<FnArg, Comma> = Punctuated::new();

    for arg in std::mem::take(&mut input.sig.inputs) {
        match arg {
            // A method's `self` cannot appear in a plain `fn(..)` pointer type.
            FnArg::Receiver(_) => {
                return macro_error(&format!(
                    "`#[inject_fn]` cannot register methods: `{fn_name}` takes `self`, \
                     which is not expressible as a function pointer parameter"
                ));
            }
            FnArg::Typed(pat_type) => {
                if !is_inject_param(&pat_type) {
                    kept.push(FnArg::Typed(pat_type));
                    continue;
                }

                let Pat::Ident(pat_ident) = &*pat_type.pat else {
                    return macro_error(&format!(
                        "`#[inject]` on `{fn_name}` requires a plain identifier binding"
                    ));
                };
                let binding = pat_ident.ident.clone();
                // `quote!` has no field-access interpolation, so bind both the
                // declared type and the unwrapped dependency type to locals.
                let declared_ty = &pat_type.ty;
                let Some(dep) = classify_dep(declared_ty) else {
                    return macro_error(&format!(
                        "unsupported `#[inject]` parameter `{binding}: {}` on `{fn_name}`; \
                         expected `Arc<Struct>` or `Arc<dyn Trait>`",
                        quote!(#declared_ty)
                    ));
                };

                let dep_ty = &dep.ty;
                let accessor = if dep.is_trait {
                    quote!(::microrust::inject::get_trait_instance::<#dep_ty>())
                } else {
                    quote!(::microrust::inject::get_instance::<#dep_ty>())
                };
                bindings.push(quote! {
                    let #binding: #declared_ty = #accessor;
                });
                injected_args.push(binding);
            }
        }
    }
    input.sig.inputs = kept;

    let original_stmts = &input.block.stmts;
    let new_body = quote!({
        #(#bindings)*
        #(#original_stmts)*
    });
    // `parse_quote!` panics on malformed input, which surfaces to the user as an
    // opaque "custom attribute panicked". Parse fallibly and report instead.
    let new_block = match syn::parse2::<syn::Block>(new_body) {
        Ok(block) => block,
        Err(err) => return macro_error(&format!("could not rewrite `{fn_name}`: {err}")),
    };
    *input.block = new_block;

    let (alias_ident, boxed_ident, ctor_ident) = fn_idents(&fn_name);
    let kept_sig = params_tokens(&input.sig.inputs);
    let key = fn_key(&fn_name);

    // The shim forwards only the caller-supplied parameters: the reduced
    // function resolves its own dependencies from the body.
    let kept_args: Vec<Ident> = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => match &*pat_type.pat {
                Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                _ => None,
            },
            FnArg::Receiver(_) => None,
        })
        .collect();

    // A sync fn is already a plain `fn` pointer; an async fn needs a shim
    // because its item type is not `fn(..) -> Pin<Box<..>>`.
    let (shim, alias_ty, callable) = if is_async {
        let boxed_sig = boxed_future_sig(&input.sig.output);
        (
            quote! {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                fn #boxed_ident(#kept_sig) -> #boxed_sig {
                    ::std::boxed::Box::pin(#fn_name(#(#kept_args),*))
                }
            },
            quote! { fn(#kept_sig) -> #boxed_sig },
            quote! { #boxed_ident as #alias_ident },
        )
    } else {
        let return_sig = &input.sig.output;
        (
            quote! {},
            quote! { fn(#kept_sig) #return_sig },
            quote! { #fn_name as #alias_ident },
        )
    };

    let tokens = quote! {
    #input

    #shim

    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub type #alias_ident = #alias_ty;

    ::microrust::ctor_parse!(#[ctor(unsafe)] #[allow(non_snake_case)] fn #ctor_ident() {
        ::microrust::inject::__register_function(#key, #callable);
    });
    };

    TokenStream::from(tokens)
}

fn is_inject_param(pat_type: &PatType) -> bool {
    pat_type
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("inject"))
}

/// Parses `name, path` for the two `set_fake_*` macros.
fn parse_name_and_path(
    input: syn::parse::ParseStream,
) -> syn::Result<(Ident, syn::Path)> {
    let name = input.parse::<Ident>()?;
    input.parse::<syn::Token![,]>()?;
    let path = input.parse::<syn::Path>()?;
    Ok((name, path))
}

/// `get_function!(name)` -> `Result<fn pointer, InjectError>`
pub fn get_function_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let name = parse_macro_input!(input as Ident);
    let (alias_ident, _, _) = fn_idents(&name);
    let key = fn_key(&name);

    let tokens = quote! {
        ::microrust::inject::get_function_ptr::<#alias_ident>(#key)
    };
    TokenStream::from(tokens)
}

/// `set_fake_function!(name, path)` for a signature-compatible callable.
pub fn set_fake_function_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (name, mock) = parse_macro_input!(input with parse_name_and_path);
    let (alias_ident, _, _) = fn_idents(&name);
    let key = fn_key(&name);

    let tokens = quote! {
        ::microrust::inject::set_function_ptr::<#alias_ident>(#key, #mock)
    };
    TokenStream::from(tokens)
}

/// `set_fake_async_function!(name, path)` for an `async fn` mock.
///
/// The mock's item type is not `fn(..) -> Pin<Box<..>>`, so the expansion boxes
/// it; that boxing is what lets it coerce to the registered alias.
pub fn set_fake_async_function_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (name, mock) = parse_macro_input!(input with parse_name_and_path);
    let (alias_ident, _, _) = fn_idents(&name);
    let key = fn_key(&name);

    let tokens = quote! {
        ::microrust::inject::set_function_ptr::<#alias_ident>(
            #key,
            |__microrust_arg| ::std::boxed::Box::pin(#mock(__microrust_arg)),
        )
    };
    TokenStream::from(tokens)
}
