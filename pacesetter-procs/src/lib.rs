use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn test(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let test_name = input.sig.ident.clone();
    let test_arguments = input.sig.inputs;
    let test_block = input.block;
    let inner_test_name = syn::Ident::new(
        format!("inner_{}", test_name).as_str(),
        input.sig.ident.span(),
    );

    let setup = quote! {
        let context = common::setup().await;
    };

    let teardown = quote! {
        teardown(context).await;
    };

    let output = quote!(
        #[::tokio::test]
        async fn #test_name() {
            #setup
            async fn #inner_test_name(#test_arguments) #test_block
            #inner_test_name(&context).await;
            #teardown
        }
    );

    TokenStream::from(output)
}
