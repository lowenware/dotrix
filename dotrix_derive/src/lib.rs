//! There macros are used to reduce boiler plate code
//!

use darling::FromDeriveInput;
use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(FromDeriveInput, Default)]
#[darling(attributes(reloadable), default)]
struct ReloadableOpts {
    field: Option<String>,
}

#[proc_macro_derive(Reloadable, attributes(reloadable))]
pub fn derive_reloadable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts: ReloadableOpts = ReloadableOpts::from_derive_input(&input).expect("Wrong options");
    let field_name = match opts.field {
        Some(x) => Ident::new(&x, Span::call_site()),
        None => Ident::new("reload_state", Span::call_site()),
    };
    let DeriveInput { ident, .. } = input;

    let output = quote! {
        impl dotrix_core::reloadable::Reloadable for #ident {
            fn get_reload_state(&self) -> &dotrix_core::reloadable::ReloadState {
                &self.#field_name
            }
        }

        impl dotrix_core::reloadable::ReloadableMut for #ident {
            fn get_reload_state_mut(&mut self) -> &mut dotrix_core::reloadable::ReloadState {
                &mut self.#field_name
            }
        }

        impl dotrix_core::reloadable::Reloadable for &mut #ident {
            fn get_reload_state(&self) -> &dotrix_core::reloadable::ReloadState {
                &self.#field_name
            }
        }

        impl dotrix_core::reloadable::ReloadableMut for &mut #ident {
            fn get_reload_state_mut(&mut self) -> &mut dotrix_core::reloadable::ReloadState {
                &mut self.#field_name
            }
        }

        impl dotrix_core::reloadable::Reloadable for & #ident {
            fn get_reload_state(&self) -> &dotrix_core::reloadable::ReloadState {
                &self.#field_name
            }
        }
    };
    output.into()
}

#[derive(FromDeriveInput, Default)]
#[darling(attributes(buffer_provider), default)]
struct BufferProviderOpts {
    field: Option<String>,
}
#[proc_macro_derive(BufferProvider, attributes(buffer_provider))]
pub fn derive_buffer_provider(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts: BufferProviderOpts =
        BufferProviderOpts::from_derive_input(&input).expect("Wrong options");
    let field_name = match opts.field {
        Some(x) => Ident::new(&x, Span::call_site()),
        None => Ident::new("buffer", Span::call_site()),
    };
    let DeriveInput { ident, .. } = input;

    let output = quote! {
        impl dotrix_core::providers::BufferProvider for #ident {
            fn get_buffer(&self) -> &dotrix_core::renderer::Buffer {
                &self.#field_name
            }
        }

        impl dotrix_core::providers::BufferProvider for & #ident {
            fn get_buffer(&self) -> &dotrix_core::renderer::Buffer {
                &self.#field_name
            }
        }

        impl dotrix_core::providers::BufferProvider for &mut #ident {
            fn get_buffer(&self) -> &dotrix_core::renderer::Buffer {
                &self.#field_name
            }
        }
    };
    output.into()
}

#[derive(FromDeriveInput, Default)]
#[darling(attributes(texture_provider), default)]
struct TextureProviderOpts {
    field: Option<String>,
}
#[proc_macro_derive(TextureProvider, attributes(texture_provider))]
pub fn derive_texture_provider(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let opts: TextureProviderOpts =
        TextureProviderOpts::from_derive_input(&input).expect("Wrong options");
    let field_name = match opts.field {
        Some(x) => Ident::new(&x, Span::call_site()),
        None => Ident::new("texture", Span::call_site()),
    };
    let DeriveInput { ident, .. } = input;

    let output = quote! {
        impl dotrix_core::providers::TextureProvider for #ident {
            fn get_texture(&self) -> &dotrix_core::renderer::Texture {
                &self.#field_name
            }
        }

        impl dotrix_core::providers::TextureProvider for & #ident {
            fn get_texture(&self) -> &dotrix_core::renderer::Texture {
                &self.#field_name
            }
        }

        impl dotrix_core::providers::TextureProvider for &mut #ident {
            fn get_texture(&self) -> &dotrix_core::renderer::Texture {
                &self.#field_name
            }
        }
    };
    output.into()
}
