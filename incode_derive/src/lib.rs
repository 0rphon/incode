extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Instructions)]
pub fn instruction_set(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_instruction_set(&ast)
}

fn impl_instruction_set(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl InstructionSet for #name {
            fn new() -> Box<Self> {Box::new(Self {len: 0,instructions: Vec::new()})}
            fn len(&self) -> usize {self.len}
            fn len_mut(&mut self) -> &mut usize {&mut self.len}
            fn instructions(self) -> Vec<Instruction> {self.instructions}
            fn instructions_ref(&self) -> &Vec<Instruction> {&self.instructions}
            fn instructions_mut(&mut self) -> &mut Vec<Instruction> {&mut self.instructions}
        }
    };
    gen.into()
}