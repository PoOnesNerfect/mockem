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

                    let mut stms = syn::parse2::<Block>(quote!({
                        {
                            use mockem::MockCall;

                            if let Some(ret) = #self_type :: #name #generics .call_mock() {
                                return ret;
                            }
                        }
                    }))?
                    .stmts;

                    std::mem::swap(&mut method.block.stmts, &mut stms);

                    method.block.stmts.extend(stms);
                }
            }

            Ok(Item::Impl(item))
        } else if lookahead.peek(Token![trait]) {
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

                        let mut stms = syn::parse2::<Block>(quote!({
                            {
                                use mockem::MockCall;

                                if let Some(ret) = <Self as #trait_name> :: #name #generics .call_mock() {
                                    return ret;
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
        } else {
            let mut item: ItemFn = input.parse()?;
            item.attrs = attrs;

            if let Some(token) = pub_token {
                item.vis = Visibility::Public(token);
            }

            let name = item.sig.ident.clone();

            let mut stms = syn::parse2::<Block>(quote!({
                {
                    use mockem::MockCall;

                    if let Some(ret) = #name .call_mock() {
                        return ret;
                    }
                }
            }))?
            .stmts;

            std::mem::swap(&mut item.block.stmts, &mut stms);

            item.block.stmts.extend(stms);

            Ok(Item::Fn(item))
        }
    }
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
