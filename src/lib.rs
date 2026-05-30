use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, GenericArgument, ItemFn, PatType, PathArguments, ReturnType, Type, parse_macro_input,
};

#[proc_macro_attribute]
pub fn strategy(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_tokens: proc_macro2::TokenStream = attr.into();

    if !attr_tokens.is_empty() {
        return syn::Error::new_spanned(
            attr_tokens,
            "strategy attribute does not take any arguments",
        )
        .into_compile_error()
        .into();
    }

    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.asyncness.is_some() {
        return syn::Error::new_spanned(&input_fn.sig, "async functions are not supported")
            .into_compile_error()
            .into();
    }

    let mut inputs = input_fn.sig.inputs.iter();
    if let Some(FnArg::Receiver(_)) = inputs.next() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "cli strategy functions must be plain free functions; remove the &self receiver and keep options, arguments, and subcommands arguments",
        )
        .into_compile_error()
        .into();
    }

    let mut inputs = input_fn.sig.inputs.iter();

    let options_pat = match inputs.next() {
        Some(FnArg::Typed(PatType { pat, ty, .. })) => {
            if !matches_vec_of_path(ty.as_ref(), &["Switch"])
                && !matches_vec_of_path(ty.as_ref(), &["cmdkit", "Switch"])
            {
                return syn::Error::new_spanned(
                    ty,
                    "cli strategy functions must accept a Vec<Switch> options argument",
                )
                .into_compile_error()
                .into();
            }

            pat
        }
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "cli strategy functions must accept an options Vec<Switch> argument",
            )
            .into_compile_error()
            .into();
        }
    };

    let arguments_pat = match inputs.next() {
        Some(FnArg::Typed(PatType { pat, ty, .. })) => {
            if !matches_vec_of_path(ty.as_ref(), &["Argument"])
                && !matches_vec_of_path(ty.as_ref(), &["cmdkit", "Argument"])
            {
                return syn::Error::new_spanned(
                    ty,
                    "cli strategy functions must accept a Vec<Argument> arguments argument",
                )
                .into_compile_error()
                .into();
            }

            pat
        }
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "cli strategy functions must accept an arguments Vec<Argument> argument",
            )
            .into_compile_error()
            .into();
        }
    };

    let subcommands_pat = match inputs.next() {
        Some(FnArg::Typed(PatType { pat, ty, .. })) => {
            if inputs.next().is_some() {
                return syn::Error::new_spanned(
                    &input_fn.sig,
                    "cli strategy functions must accept exactly three parsed invocation arguments",
                )
                .into_compile_error()
                .into();
            }

            if !matches_vec_of_path(ty.as_ref(), &["String"])
                && !matches_vec_of_path(ty.as_ref(), &["std", "string", "String"])
                && !matches_vec_of_path(ty.as_ref(), &["alloc", "string", "String"])
            {
                return syn::Error::new_spanned(
                    ty,
                    "cli strategy functions must accept a Vec<String> subcommands argument",
                )
                .into_compile_error()
                .into();
            }

            pat
        }
        _ => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "cli strategy functions must accept a subcommands Vec<String> argument",
            )
            .into_compile_error()
            .into();
        }
    };

    match &input_fn.sig.output {
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(path)
                if path.path.segments.len() == 1
                    && path.path.segments[0].ident == "Result"
                    && matches_result_type(&path.path) => {}
            _ => {
                return syn::Error::new_spanned(
                    ty,
                    "cli strategy functions must return Result<(), cmdkit::StrategyError>",
                )
                .into_compile_error()
                .into();
            }
        },
        ReturnType::Default => {
            return syn::Error::new_spanned(
                &input_fn.sig,
                "cli strategy functions must return Result<(), cmdkit::StrategyError>",
            )
            .into_compile_error()
            .into();
        }
    }

    let fn_ident = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let strategy_ident = format_ident!("{}", to_pascal(&fn_ident.to_string()));
    let factory_ident = format_ident!("{}_strategy", fn_ident);
    let attrs = &input_fn.attrs;
    let body = &input_fn.block;

    let expanded = quote! {
        #(#attrs)*
        #vis struct #strategy_ident;

        impl #strategy_ident {
            #vis fn new() -> Self {
                Self
            }
        }

        impl ::cmdkit::CommandStrategy for #strategy_ident {
            fn execute(
                &self,
                #options_pat: Vec<::cmdkit::Switch>,
                #arguments_pat: Vec<::cmdkit::Argument>,
                #subcommands_pat: Vec<String>,
            ) -> Result<(), ::cmdkit::StrategyError> {
                #body
            }
        }

        #vis fn #factory_ident() -> #strategy_ident {
            #strategy_ident::new()
        }
    };

    expanded.into()
}

fn to_pascal(s: &str) -> String {
    let mut out = String::new();
    for part in s.split('_') {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

fn matches_vec_of_path(ty: &Type, expected_segments: &[&str]) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };

    let Some(last_segment) = path.path.segments.last() else {
        return false;
    };

    if last_segment.ident != "Vec" {
        return false;
    }

    let PathArguments::AngleBracketed(arguments) = &last_segment.arguments else {
        return false;
    };

    let Some(GenericArgument::Type(inner_type)) = arguments.args.first() else {
        return false;
    };

    matches_path_segments(inner_type, expected_segments)
}

fn matches_result_type(path: &syn::Path) -> bool {
    let Some(last_segment) = path.segments.last() else {
        return false;
    };

    let PathArguments::AngleBracketed(arguments) = &last_segment.arguments else {
        return false;
    };

    let mut args = arguments.args.iter();

    matches!(args.next(), Some(GenericArgument::Type(Type::Tuple(tuple))) if tuple.elems.is_empty())
        && matches!(
            args.next(),
            Some(GenericArgument::Type(inner_type)) if matches_path_segments(inner_type, &["StrategyError"])
                || matches_path_segments(inner_type, &["cmdkit", "StrategyError"])
        )
        && args.next().is_none()
}

fn matches_path_segments(ty: &Type, expected_segments: &[&str]) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };

    let actual_segments: Vec<_> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();

    actual_segments == expected_segments
}
