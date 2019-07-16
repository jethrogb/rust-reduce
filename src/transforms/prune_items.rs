// Copyright (c) Jethro G. Beekman
//
// This file is part of rust-reduce.
//
// rust-reduce is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-reduce is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-reduce.  If not, see <https://www.gnu.org/licenses/>.
use syn::punctuated::Punctuated;

/// Try to remove each item.

pub fn prune_items<F: FnMut(&syn::File) -> bool>(file: &mut syn::File, mut try_compile: F) {
    let mut level = 0;
    let mut index = 0;
    loop {
        let backup = file.clone();
        if !file.items.prune(level, &mut { index }) {
            if index == 0 {
                break;
            }
            level += 1;
            index = 0;
            continue;
        }
        if !try_compile(&file) {
            *file = backup;
            index += 1;
        } else {
            // try delete next, which will be at same index now that we've
            // deleted something
        }
    }
}

trait Prune {
    fn prune(&mut self, level: usize, index: &mut usize) -> bool;
}

impl Prune for Vec<syn::Item> {
    fn prune(&mut self, level: usize, index: &mut usize) -> bool {
        if level == 0 {
            if *index < self.len() {
                self.remove(*index);
                true
            } else {
                *index -= self.len();
                false
            }
        } else {
            for item in self {
                if match item {
                    syn::Item::Mod(syn::ItemMod {
                        content: Some((_, items)),
                        ..
                    }) => items.prune(level - 1, index),
                    syn::Item::Struct(item @ syn::ItemStruct{ .. }) => item.prune(level - 1, index),
                    syn::Item::Impl(syn::ItemImpl { items, .. }) => items.prune(level - 1, index),
                    syn::Item::Enum(item @ syn::ItemEnum{ .. }) => item.prune(level - 1, index),
                    _ => false,
                } {
                    return true;
                }
            }
            false
        }
    }
}

impl Prune for Vec<syn::ImplItem> {
    fn prune(&mut self, level: usize, index: &mut usize) -> bool {
        if level < 5 {
            if *index < self.len() {
                self.remove(*index);
                true
            } else {
                *index -= self.len();
                false
            }
        } else {
            false
        }
    }
}

impl Prune for syn::ItemEnum{
    fn prune(&mut self, level: usize, index: &mut usize) -> bool {
        if level < 5 {
            if *index < self.variants.len() {
                //  dbg!(my_enum.variants.clone());
                let mut smaller : Punctuated<syn::Variant, syn::Token![,]> = Punctuated::new();
                for (i, pair) in self.variants.pairs().enumerate() {
                    if i != *index {
                        smaller.push((*pair.value()).clone());
                    }
                }
                self.variants = smaller;
                true
            }
            else {
                *index -= self.variants.len();
                false
            }
        } else { false }
    }
}

impl Prune for syn::ItemStruct {
    fn prune(&mut self, level: usize, index: &mut usize) -> bool {
        if level < 5 {
            if let syn::Fields::Named(ref mut named) = self.fields {
                if *index < named.named.len() {
                    let mut smaller : Punctuated<syn::Field, syn::Token![,]> = Punctuated::new();
                    for (i, pair) in named.named.pairs().enumerate() {
                        if i != *index {
                            smaller.push((*pair.value()).clone());
                        }
                    }
                    named.named = smaller;
                    true
                }
                else {
                    *index -= named.named.len();
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}