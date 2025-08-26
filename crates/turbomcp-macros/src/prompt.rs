//! Production-grade prompt macro implementation with comprehensive argument parsing

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemFn, Lit, Meta, Token, parse::Parse, parse::ParseStream, parse_macro_input,
    punctuated::Punctuated,
};

/// Comprehensive prompt configuration for maximum utility and DX
#[derive(Debug, Default)]
struct PromptConfig {
    name: Option<String>,
    description: String,
    tags: Vec<String>,
}

/// Production-grade attribute parser for comprehensive prompt configuration
struct PromptArgs {
    items: Punctuated<Meta, Token![,]>,
}

impl Parse for PromptArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PromptArgs {
            items: input.parse_terminated(Meta::parse, Token![,])?,
        })
    }
}

/// Generate production-grade prompt implementation with comprehensive argument processing
pub fn generate_prompt_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    // Production-grade argument parsing with comprehensive validation
    let config = match parse_prompt_args(args) {
        Ok(config) => config,
        Err(error) => {
            return syn::Error::new_spanned(&input.sig.ident, error)
                .to_compile_error()
                .into();
        }
    };

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_sig = &input.sig;
    let prompt_name = config.name.unwrap_or_else(|| fn_name.to_string());
    let description = &config.description;

    // Generate comprehensive metadata function
    let metadata_fn_name = syn::Ident::new(
        &format!("__turbomcp_prompt_metadata_{fn_name}"),
        proc_macro2::Span::call_site(),
    );

    // Generate public metadata function name for testing capability
    let public_metadata_fn_name = syn::Ident::new(
        &format!("{fn_name}_metadata"),
        proc_macro2::Span::call_site(),
    );

    // Generate tags as a vector literal
    let tags_tokens = if config.tags.is_empty() {
        quote! { vec![] }
    } else {
        let tag_strings = &config.tags;
        quote! { vec![#(#tag_strings.to_string()),*] }
    };

    // Production-grade implementation with comprehensive metadata support
    let expanded = quote! {
        // Preserve original function with all its attributes
        #fn_vis #fn_sig #fn_block

        // Generate comprehensive metadata function for internal use
        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub fn #metadata_fn_name() -> (&'static str, &'static str, Vec<String>) {
            (
                #prompt_name,
                #description,
                #tags_tokens
            )
        }

        // Generate public metadata function for testing and integration
        /// Get comprehensive metadata for this prompt
        ///
        /// Returns (name, description, tags) tuple providing complete prompt metadata
        /// for testing, documentation, and runtime introspection with maximum utility.
        pub fn #public_metadata_fn_name() -> (&'static str, &'static str, Vec<String>) {
            (
                #prompt_name,
                #description,
                #tags_tokens
            )
        }
    };

    TokenStream::from(expanded)
}

/// Production-grade argument parsing with progressive enhancement: simple to advanced usage
fn parse_prompt_args(args: TokenStream) -> Result<PromptConfig, String> {
    if args.is_empty() {
        return Err("Prompt description is required for proper documentation".to_string());
    }

    let args: proc_macro2::TokenStream = args.into();

    // First, try parsing as a simple string literal: #[prompt("description")]
    if let Ok(lit_str) = syn::parse2::<syn::LitStr>(args.clone()) {
        return Ok(PromptConfig {
            description: lit_str.value(),
            name: None,
            tags: vec![],
        });
    }

    // Next, try parsing as structured arguments: #[prompt(desc = "...", name = "...", tags = [...])]
    let parsed_args = match syn::parse2::<PromptArgs>(args) {
        Ok(args) => args,
        Err(e) => {
            return Err(format!(
                "Invalid prompt macro arguments. Use:\n  #[prompt(\"description\")] for simple usage\n  #[prompt(desc = \"...\", name = \"...\", tags = [...])] for advanced\nError: {}",
                e
            ));
        }
    };

    let mut config = PromptConfig::default();

    // Process each attribute with comprehensive validation
    for meta in &parsed_args.items {
        match meta {
            // Handle path-only syntax (not supported, guide user to clear syntax)
            Meta::Path(_) => {
                return Err(
                    "Use #[prompt(desc = \"description\")] for structured syntax".to_string(),
                );
            }

            // Handle named attributes: #[prompt(name = "...", desc = "...", tags = [...])]
            Meta::NameValue(name_value) => {
                let attr_name = name_value
                    .path
                    .get_ident()
                    .ok_or_else(|| "Invalid attribute name".to_string())?
                    .to_string();

                match attr_name.as_str() {
                    "name" => {
                        if let syn::Expr::Lit(expr_lit) = &name_value.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.name = Some(lit_str.value());
                            } else {
                                return Err("Prompt name must be a string literal".to_string());
                            }
                        } else {
                            return Err("Prompt name must be a string literal".to_string());
                        }
                    }
                    "desc" | "description" => {
                        if let syn::Expr::Lit(expr_lit) = &name_value.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                config.description = lit_str.value();
                            } else {
                                return Err(
                                    "Prompt description must be a string literal".to_string()
                                );
                            }
                        } else {
                            return Err("Prompt description must be a string literal".to_string());
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Unknown prompt attribute: {}. Supported: name, desc, tags",
                            attr_name
                        ));
                    }
                }
            }

            // Handle list attributes like tags = ["tag1", "tag2"]
            Meta::List(meta_list) => {
                let attr_name = meta_list
                    .path
                    .get_ident()
                    .ok_or_else(|| "Invalid attribute name".to_string())?
                    .to_string();

                match attr_name.as_str() {
                    "tags" => {
                        // Parse the token stream inside the brackets
                        let tags_content = meta_list.tokens.clone();
                        let bracketed: syn::ExprArray = syn::parse2(quote! { [#tags_content] })
                            .map_err(|_| {
                                "Tags must be an array of strings like [\"tag1\", \"tag2\"]"
                                    .to_string()
                            })?;

                        for expr in bracketed.elems {
                            if let syn::Expr::Lit(expr_lit) = expr {
                                if let Lit::Str(lit_str) = expr_lit.lit {
                                    config.tags.push(lit_str.value());
                                } else {
                                    return Err("Tag values must be string literals".to_string());
                                }
                            } else {
                                return Err("Tag values must be string literals".to_string());
                            }
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Unknown list attribute: {}. Supported: tags",
                            attr_name
                        ));
                    }
                }
            }
        }
    }

    // Final validation
    if config.description.is_empty() {
        return Err("Prompt description is required. Use #[prompt(desc = \"your description\")] or #[prompt(\"description\")]".to_string());
    }

    Ok(config)
}
