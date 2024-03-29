use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    Attribute, Block, GenericParam, ImplItem, ItemFn, ItemImpl, ItemTrait, Token, TraitItem,
    Visibility,
};

pub enum Item {
    Fn(ItemFn),
    Impl(ItemImpl),
    Trait(ItemTrait),
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Item::Fn(item) => item.to_tokens(tokens),
            Item::Impl(item) => item.to_tokens(tokens),
            Item::Trait(item) => item.to_tokens(tokens),
        }
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let mut lookahead = input.lookahead1();

        if lookahead.peek(Token![unsafe]) {
            let ahead = input.fork();
            ahead.parse::<Token![unsafe]>()?;
            lookahead = ahead.lookahead1();
        }

        let pub_token = if lookahead.peek(Token![pub]) {
            let ahead = input.fork();
            let token = ahead.parse::<Token![pub]>()?;
            lookahead = ahead.lookahead1();
            Some(token)
        } else {
            None
        };

        if lookahead.peek(Token![impl]) {
            inject_impl(input, attrs)
        } else if lookahead.peek(Token![trait]) {
            inject_trait(input, pub_token, attrs)
        } else {
            inject_fn(input, pub_token, attrs)
        }
    }
}

fn inject_impl(input: ParseStream, attrs: Vec<Attribute>) -> Result<Item> {
    let mut item: ItemImpl = input.parse()?;
    item.attrs = attrs;
    let trait_name = item.trait_.clone();

    for item in item.items.iter_mut() {
        if let ImplItem::Fn(method) = item {
            let name = method.sig.ident.clone();
            let generics = method
                .sig
                .generics
                .params
                .iter()
                .map(|p| match p {
                    GenericParam::Lifetime(lt) => lt.lifetime.ident.clone(),
                    GenericParam::Type(ty) => ty.ident.clone(),
                    GenericParam::Const(c) => c.ident.clone(),
                })
                .collect::<Vec<_>>();
            let generics = if generics.is_empty() {
                quote!()
            } else {
                quote!(::<#(#generics),*>)
            };

            let self_type = if let Some((_, path, _)) = &trait_name {
                quote!(<Self as #path>)
            } else {
                quote!(Self)
            };

            let args = method.sig.inputs.iter().map(|a| {
                if let syn::FnArg::Typed(pat) = a {
                    let pat = &pat.pat;
                    quote!(#pat)
                } else {
                    quote!(self)
                }
            });

            let ret = if let syn::ReturnType::Type(_, ty) = &method.sig.output {
                quote!(#ty)
            } else {
                quote!(())
            };

            let mut stms = syn::parse2::<Block>(quote!({
                {
                    use mockem::CallMock;

                    if #self_type :: #name #generics .mock_exists(core::marker::PhantomData::<#ret>) {
                        return #self_type :: #name #generics .call_mock((#(#args,)*));
                    }
                }
            }))?
            .stmts;

            std::mem::swap(&mut method.block.stmts, &mut stms);

            method.block.stmts.extend(stms);
        }
    }

    Ok(Item::Impl(item))
}

fn inject_trait(
    input: ParseStream,
    pub_token: Option<Token![pub]>,
    attrs: Vec<Attribute>,
) -> Result<Item> {
    let mut item: ItemTrait = input.parse()?;
    item.attrs = attrs;

    if let Some(token) = pub_token {
        item.vis = Visibility::Public(token);
    }

    let trait_name = item.ident.clone();

    for item in item.items.iter_mut() {
        if let TraitItem::Fn(method) = item {
            if let Some(block) = method.default.as_mut() {
                let name = method.sig.ident.clone();
                let generics = method
                    .sig
                    .generics
                    .params
                    .iter()
                    .map(|p| match p {
                        GenericParam::Lifetime(lt) => lt.lifetime.ident.clone(),
                        GenericParam::Type(ty) => ty.ident.clone(),
                        GenericParam::Const(c) => c.ident.clone(),
                    })
                    .collect::<Vec<_>>();

                let generics = if generics.is_empty() {
                    quote!()
                } else {
                    quote!(::<#(#generics),*>)
                };

                let args = method.sig.inputs.iter().map(|a| {
                    if let syn::FnArg::Typed(pat) = a {
                        let pat = &pat.pat;
                        quote!(#pat)
                    } else {
                        quote!(self)
                    }
                });

                let ret = if let syn::ReturnType::Type(_, ty) = &method.sig.output {
                    quote!(#ty)
                } else {
                    quote!(())
                };

                let mut stms = syn::parse2::<Block>(quote!({
                    {
                        use mockem::CallMock;

                        if <Self as #trait_name> :: #name #generics .mock_exists(core::marker::PhantomData::<#ret>) {
                            return <Self as #trait_name> :: #name #generics .call_mock((#(#args,)*));
                        }
                    }
                }))?
                .stmts;

                std::mem::swap(&mut block.stmts, &mut stms);

                block.stmts.extend(stms);
            }
        }
    }

    Ok(Item::Trait(item))
}

fn inject_fn(
    input: ParseStream,
    pub_token: Option<Token![pub]>,
    attrs: Vec<Attribute>,
) -> Result<Item> {
    let mut item: ItemFn = input.parse()?;
    item.attrs = attrs;

    if let Some(token) = pub_token {
        item.vis = Visibility::Public(token);
    }

    let name = item.sig.ident.clone();

    let args = item.sig.inputs.iter().map(|a| {
        if let syn::FnArg::Typed(pat) = a {
            let pat = &pat.pat;
            quote!(#pat)
        } else {
            quote!(self)
        }
    });

    let ret = if let syn::ReturnType::Type(_, ty) = &item.sig.output {
        quote!(#ty)
    } else {
        quote!(())
    };

    let mut stms = syn::parse2::<Block>(quote!({
        {
            use mockem::CallMock;

            if  #name .mock_exists(core::marker::PhantomData::<#ret>) {
                return #name .call_mock((#(#args,)*));
            }
        }
    }))?
    .stmts;

    std::mem::swap(&mut item.block.stmts, &mut stms);

    item.block.stmts.extend(stms);

    Ok(Item::Fn(item))
}
