use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn only_controller(_attr: proc_macro::TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let sig = item_fn.sig;
    let block = item_fn.block.stmts;
    quote! {
        #sig {
            if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
                ic_cdk::trap("Not permitted.");
            };
            #(#block);*
        }
    }
    .into()
}

pub fn only_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let sig = item_fn.sig;
    let block = item_fn.block.stmts;
    quote! {
        #sig {
            if ic_cdk::caller() != get_proxy() {
                ic_cdk::trap("Not permitted.");
            }
            #(#block);*
        }
    }
    .into()
}

pub fn metric(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let sig = item_fn.sig;
    let block = item_fn.block.stmts;
    quote! {
        #sig {
            let timestamper = chainsight_cdk::time::TimeStamper;
            let start = timestamper::now_nanosec();
            #(#block);*
            let end = timestamper::now_nanosec();
            chainsight_cdk::metric::metric(stringify!(#sig), TaskDuration::new(start, end));
        }
    }
    .into()
}
