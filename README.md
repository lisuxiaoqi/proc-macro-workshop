# Rust Latam: procedural macros workshop
Solution to the proc-macro-workshop

All tests pass

branch: train

# Thanks

Thanks dtolnay. Your great work help me go through the every details as a macro beginner.
* https://github.com/dtolnay/proc-macro-workshop

Thanks Robbepop. It really kills me in bitfield tests 4. So lucky to get the ideas from your repo.
* https://github.com/Robbepop/proc-macro-workshop

# Feelings
There is no magic behind the macro. All the works is to parse tokens, generate tokens. Nothing more than that!

To be honest, a little boring..

The key point is to understand the concept of TokenStream and TokenTree. It helps a lot during the proccess with the syn and quote crates, but the proc_macro2 is the real boss.

And keep in mind that macro only can handle tokens, it's impossible to understand the semantics in this peroid, so pls forget to get the real output from variables.
