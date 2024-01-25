use proc_macro::TokenStream;

mod extract_form;
mod generics_param;
mod method_handler;
mod middleware;

#[proc_macro_derive(Form, attributes(multer))]
pub fn derive_form(input: TokenStream) -> TokenStream {
    extract_form::generate(input)
}

#[proc_macro_attribute]
pub fn middleware(_: TokenStream, input: TokenStream) -> TokenStream {
    middleware::generate(input)
}

#[proc_macro]
pub fn repeat_macro_max_generics_param(input: TokenStream) -> TokenStream {
    generics_param::generate(input)
}

macro_rules! generate_route_attribute {
    ($name: ident, $method: expr) => {
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, input: TokenStream) -> TokenStream {
            method_handler::generate($method, args, input)
        }
    };
}

generate_route_attribute!(get, "GET");
generate_route_attribute!(put, "PUT");
generate_route_attribute!(post, "POST");
generate_route_attribute!(delete, "DELETE");
generate_route_attribute!(head, "HEAD");
generate_route_attribute!(patch, "PATCH");
generate_route_attribute!(options, "OPTIONS");
