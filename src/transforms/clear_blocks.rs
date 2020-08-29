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

/// Try to replace each block with `{ unimplemented!() }`, similar to `rustc`'s
/// every body loops printer.
use std::mem;

use quote::quote;
use syn::visit_mut::*;

pub fn clear_blocks<F: FnMut(&syn::File) -> bool>(file: &mut syn::File, mut try_compile: F) {
    let mut visitor = BlockVisitor {
        backup: None,
        cur_index: 0,
        target_index: 1,
        unimplemented: syn::parse2(quote!({ unimplemented!() })).unwrap(),
    };

    loop {
        visitor.cur_index = 0;
        visit_file_mut(&mut visitor, file);

        // no more changes to be made
        if visitor.backup.is_none() {
            break;
        }

        if try_compile(file) {
            // this change works, keep it!
            visitor.backup = None;
        }
    }
}

struct BlockVisitor {
    backup: Option<syn::Block>,
    cur_index: usize,
    target_index: usize,
    unimplemented: syn::Block,
}

impl VisitMut for BlockVisitor {
    fn visit_block_mut(&mut self, i: &mut syn::Block) {
        self.cur_index += 1;

        if self.target_index == self.cur_index {
            if let Some(backup) = self.backup.take() {
                // the change we tried didn't work. revert and try the next
                // possible change
                mem::replace(i, backup);
            } else if *i != self.unimplemented {
                self.backup = Some(mem::replace(i, self.unimplemented.clone()));
                return;
            }

            self.target_index += 1;
        }

        visit_block_mut(self, i)
    }
}
