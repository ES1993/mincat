use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, LitStr};

pub fn generate(
    method: &str,
    path: proc_macro::TokenStream,
    func: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut func = parse_macro_input!(func as ItemFn);
    let func_vis = func.vis.clone();
    let func_name = func.sig.ident.clone();
    let new_func_name = format_ident!("__origin__{}__", func.sig.ident);
    let method = format_ident!("{}", method);
    let method = quote!(mincat::http::Method::#method);
    let path = parse_macro_input!(path as LitStr);

    func.sig.ident = new_func_name.clone();

    quote!(
        #[allow(non_snake_case)]
        #func

        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        #func_vis struct #func_name;

        impl #func_name {
            fn middleware<T>(&mut self, middleware: T) -> mincat::http::Route
            where
                T: Into<Box<dyn mincat::middleware::Middleware>>,
            {
                let mut route =  mincat::http::Route::init(#method, #path, #new_func_name);
                route.middleware(middleware)
            }
        }

        impl From<#func_name> for mincat::http::Route {
            fn from(_:#func_name) -> mincat::http::Route {
                mincat::http::Route::init(#method, #path, #new_func_name)
            }
        }
    )
    .into()
}
