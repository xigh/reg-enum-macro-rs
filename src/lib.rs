#![allow(dead_code)]

use std::fmt::Debug;

use proc_macro2::{TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parse_macro_input, LitInt, Ident, Token, Result};

struct RegEnumRange {
    value: LitInt,
    start: LitInt,
    end: LitInt,
}

impl Parse for RegEnumRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let value: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let start: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let end: LitInt = input.parse()?;
        Ok(Self{
            value,
            start,
            end,
        })
    }
}

#[derive(Clone)]
struct RegEnumEntry {
    name: Ident,
    value: LitInt,
    start: Option<LitInt>,
    end: Option<LitInt>,
}

impl Debug for RegEnumEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = match &self.start {
            Some(start) => start.base10_digits(),
            None => "",
        };
        let end = match &self.end {
            Some(end) => end.base10_digits(),
            None => "",
        };
        f.debug_struct("")
            .field("name", &self.name.to_string())
            .field("value", &self.value.base10_digits())
            .field("start", &start)
            .field("end", &end)
            .finish()
    }
}

impl Parse for RegEnumEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let range: RegEnumRange = content.parse()?;

            return Ok(Self{
                name,
                value: range.value,
                start: Some(range.start),
                end: Some(range.end),
            });
        }

        let value: LitInt = input.parse()?;
        Ok(Self{
            name,
            value,
            start: None,
            end: None,
        })
    }
}

struct RegEnum {
    name: Ident,
    ty: Ident,
    entries: Vec<RegEnumEntry>,
}

impl Debug for RegEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegEnum")
            .field("name", &self.name.to_string())
            .field("ty", &self.ty.to_string())
            .field("entries", &self.entries)
            .finish()
    }
}

impl Parse for RegEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;    
        let ty: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        
        // println!("name={}, type={}", name, ty);
        let mut re = RegEnum { name: name, ty: ty, entries: Vec::new() };

        let content;
        braced!(content in input);

        let entries = 
            Punctuated::<RegEnumEntry, syn::Token![,]>::parse_terminated(&content)?;
        for entry in entries.iter() {
            re.entries.push((*entry).clone());
        }

        // println!("{:?}", re);

        Ok(re)
    }
}

#[proc_macro]
pub fn reg_enum(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as RegEnum);

    let name = input.name;
    let ty = input.ty;
    let from_ty = format_ident!("from_{}", ty.to_string());
    let to_ty = format_ident!("to_{}", ty.to_string());

    let mut stream0 = proc_macro2::TokenStream::new();
    let mut stream1 = proc_macro2::TokenStream::new();
    let mut stream2 = proc_macro2::TokenStream::new();
    let mut stream3 = proc_macro2::TokenStream::new();

    for entry in input.entries.iter() {
        let (fname, value) = (entry.name.clone(), entry.value.clone());
        let value: u64 = value.base10_parse().unwrap(); // todo: convert to #repr ???

        match (entry.start.clone(), entry.end.clone()) {
            (Some(start), Some(end)) => {
                let start: u64 = start.base10_parse().unwrap();
                let end: u64 = end.base10_parse().unwrap();
                for i in start..=end {
                    let iden = format_ident!("{}{}", fname, i);
                    let value = value + i; // todo

                    let tokens0 = quote! {
                        #name :: #iden => #value as #ty,
                    };
                    stream0.extend(TokenStream::from(tokens0));
            
                    let tokens1 = quote! {
                        #iden ,
                    };
                    stream1.extend(TokenStream::from(tokens1));
            
                    let value = value as u16; // todo
                    let tokens2 = quote! {
                        #value => #name :: #iden ,
                    };
                    stream2.extend(TokenStream::from(tokens2));
            
                    let fname = format!("{}{}", fname, i);
                    let tokens3 = quote! {
                        #name :: #iden => #fname ,
                    };
                    stream3.extend(TokenStream::from(tokens3));            
                }
            }
            _ => {
                let iden = format_ident!("{}", fname);

                let tokens0 = quote! {
                    #name :: #iden => #value as #ty,
                };
                stream0.extend(TokenStream::from(tokens0));
        
                let tokens1 = quote! {
                    #iden ,
                };
                stream1.extend(TokenStream::from(tokens1));
        
                let value = value as u16; // todo
                let tokens2 = quote! {
                    #value => #name :: #iden ,
                };
                stream2.extend(TokenStream::from(tokens2));
        
                let fname = format!("{}", fname);
                let tokens3 = quote! {
                    #name :: #iden => #fname ,
                };
                stream3.extend(TokenStream::from(tokens3));        
            }
        }
    }

    stream0.extend(quote! {
        T::other(v) => *v,
    });
    let group0 = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, stream0);

    stream1.extend(quote! {
        other(#ty),
    });
    let group1 = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, stream1);

    stream2.extend(quote! {
        _ => T::other(v),
    });
    let group2 = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, stream2);

    stream3.extend(quote! {
        Self::other(x) => {
            return write!(f, "{}", x);
        }
    });
    let group3 = proc_macro2::Group::new(proc_macro2::Delimiter::Brace, stream3);

    let tokens = quote! {
        #[allow(non_camel_case_types)]
        pub enum #name #group1

        impl #name {
            pub fn #from_ty (v: #ty) -> Self {
                match v #group2
            }

            pub fn #to_ty (&self) -> #ty {
                match self #group0
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                let s = match self #group3 ;
                write!(f, "{}", s)
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
