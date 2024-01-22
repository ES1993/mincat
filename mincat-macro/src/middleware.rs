use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

pub fn generate(func: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut func = parse_macro_input!(func as ItemFn);
    let func_vis = func.vis.clone();
    let func_name = func.sig.ident.clone();
    let new_func_name = format_ident!("__origin__{}__", func.sig.ident);

    func.sig.ident = new_func_name.clone();

    quote!(
        #[allow(non_snake_case)]
        #func

        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy)]
        #func_vis struct #func_name;

        impl From<#func_name> for Box<dyn mincat::middleware::Middleware> {
            fn from(_:#func_name) -> Box<dyn mincat::middleware::Middleware> {
                use  mincat::middleware::Middleware;
                mincat::middleware::FuncMiddleware::from_fn(#new_func_name).clone_box()
            }
        }
    )
    .into()
}
