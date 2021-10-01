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

/// Try to remove each `#[doc]` attribute (this includes doc comments).
use syn::{visit_mut::*, *};

pub fn remove_doc_attrs<F: FnMut(&File) -> bool>(file: &mut File, mut try_compile: F) {
	let mut visitor = AttrContainerVisitor {
		backup: None,
		cur_index: 0,
		target_index: 1,
	};

	loop {
		visitor.cur_index = 0;

		visitor.visit_attr_container(&mut file.attrs);
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

struct AttrContainerVisitor {
	backup: Option<Vec<Attribute>>,
	cur_index: usize,
	target_index: usize,
}

impl AttrContainerVisitor {
	fn visit_attr_container(&mut self, i: &mut Vec<Attribute>) {
		self.cur_index += 1;

		if self.target_index == self.cur_index {
			if let Some(backup) = self.backup.take() {
				// the change we tried didn't work. revert and try the next
				// possible change
				*i = backup;
			} else if i.iter().any(|attr| attr.path.is_ident("doc")) {
				self.backup = Some(i.clone());
				i.retain(|attr| !attr.path.is_ident("doc"));
				return;
			}

			self.target_index += 1;
		}
	}
}

macro_rules! impl_VisitMut_attrs {
	($(fn $i:ident(&mut self, i: &mut $t:ty))*) => {
		$(
			fn $i(&mut self, i: &mut $t) {
				self.visit_attr_container(&mut i.attrs);
				$i(self, i);
			}
		)*
	}
}

impl VisitMut for AttrContainerVisitor {
	impl_VisitMut_attrs! {
		fn visit_arm_mut(&mut self, i: &mut Arm)
		fn visit_const_param_mut(&mut self, i: &mut ConstParam)
		fn visit_derive_input_mut(&mut self, i: &mut DeriveInput)
		fn visit_expr_array_mut(&mut self, i: &mut ExprArray)
		fn visit_expr_assign_mut(&mut self, i: &mut ExprAssign)
		fn visit_expr_assign_op_mut(&mut self, i: &mut ExprAssignOp)
		fn visit_expr_async_mut(&mut self, i: &mut ExprAsync)
		fn visit_expr_binary_mut(&mut self, i: &mut ExprBinary)
		fn visit_expr_block_mut(&mut self, i: &mut ExprBlock)
		fn visit_expr_box_mut(&mut self, i: &mut ExprBox)
		fn visit_expr_break_mut(&mut self, i: &mut ExprBreak)
		fn visit_expr_call_mut(&mut self, i: &mut ExprCall)
		fn visit_expr_cast_mut(&mut self, i: &mut ExprCast)
		fn visit_expr_closure_mut(&mut self, i: &mut ExprClosure)
		fn visit_expr_continue_mut(&mut self, i: &mut ExprContinue)
		fn visit_expr_field_mut(&mut self, i: &mut ExprField)
		fn visit_expr_for_loop_mut(&mut self, i: &mut ExprForLoop)
		fn visit_expr_group_mut(&mut self, i: &mut ExprGroup)
		fn visit_expr_if_mut(&mut self, i: &mut ExprIf)
		fn visit_expr_index_mut(&mut self, i: &mut ExprIndex)
		fn visit_expr_let_mut(&mut self, i: &mut ExprLet)
		fn visit_expr_lit_mut(&mut self, i: &mut ExprLit)
		fn visit_expr_loop_mut(&mut self, i: &mut ExprLoop)
		fn visit_expr_macro_mut(&mut self, i: &mut ExprMacro)
		fn visit_expr_match_mut(&mut self, i: &mut ExprMatch)
		fn visit_expr_method_call_mut(&mut self, i: &mut ExprMethodCall)
		fn visit_expr_paren_mut(&mut self, i: &mut ExprParen)
		fn visit_expr_path_mut(&mut self, i: &mut ExprPath)
		fn visit_expr_range_mut(&mut self, i: &mut ExprRange)
		fn visit_expr_reference_mut(&mut self, i: &mut ExprReference)
		fn visit_expr_repeat_mut(&mut self, i: &mut ExprRepeat)
		fn visit_expr_return_mut(&mut self, i: &mut ExprReturn)
		fn visit_expr_struct_mut(&mut self, i: &mut ExprStruct)
		fn visit_expr_try_mut(&mut self, i: &mut ExprTry)
		fn visit_expr_try_block_mut(&mut self, i: &mut ExprTryBlock)
		fn visit_expr_tuple_mut(&mut self, i: &mut ExprTuple)
		fn visit_expr_type_mut(&mut self, i: &mut ExprType)
		fn visit_expr_unary_mut(&mut self, i: &mut ExprUnary)
		fn visit_expr_unsafe_mut(&mut self, i: &mut ExprUnsafe)
		fn visit_expr_while_mut(&mut self, i: &mut ExprWhile)
		fn visit_expr_yield_mut(&mut self, i: &mut ExprYield)
		fn visit_field_mut(&mut self, i: &mut Field)
		fn visit_field_pat_mut(&mut self, i: &mut FieldPat)
		fn visit_field_value_mut(&mut self, i: &mut FieldValue)
		fn visit_file_mut(&mut self, i: &mut File)
		fn visit_foreign_item_fn_mut(&mut self, i: &mut ForeignItemFn)
		fn visit_foreign_item_macro_mut(&mut self, i: &mut ForeignItemMacro)
		fn visit_foreign_item_static_mut(&mut self, i: &mut ForeignItemStatic)
		fn visit_foreign_item_type_mut(&mut self, i: &mut ForeignItemType)
		fn visit_impl_item_const_mut(&mut self, i: &mut ImplItemConst)
		fn visit_impl_item_macro_mut(&mut self, i: &mut ImplItemMacro)
		fn visit_impl_item_method_mut(&mut self, i: &mut ImplItemMethod)
		fn visit_impl_item_type_mut(&mut self, i: &mut ImplItemType)
		fn visit_item_const_mut(&mut self, i: &mut ItemConst)
		fn visit_item_enum_mut(&mut self, i: &mut ItemEnum)
		fn visit_item_extern_crate_mut(&mut self, i: &mut ItemExternCrate)
		fn visit_item_fn_mut(&mut self, i: &mut ItemFn)
		fn visit_item_foreign_mod_mut(&mut self, i: &mut ItemForeignMod)
		fn visit_item_impl_mut(&mut self, i: &mut ItemImpl)
		fn visit_item_macro_mut(&mut self, i: &mut ItemMacro)
		fn visit_item_macro2_mut(&mut self, i: &mut ItemMacro2)
		fn visit_item_mod_mut(&mut self, i: &mut ItemMod)
		fn visit_item_static_mut(&mut self, i: &mut ItemStatic)
		fn visit_item_struct_mut(&mut self, i: &mut ItemStruct)
		fn visit_item_trait_mut(&mut self, i: &mut ItemTrait)
		fn visit_item_trait_alias_mut(&mut self, i: &mut ItemTraitAlias)
		fn visit_item_type_mut(&mut self, i: &mut ItemType)
		fn visit_item_union_mut(&mut self, i: &mut ItemUnion)
		fn visit_item_use_mut(&mut self, i: &mut ItemUse)
		fn visit_lifetime_def_mut(&mut self, i: &mut LifetimeDef)
		fn visit_local_mut(&mut self, i: &mut Local)
		fn visit_trait_item_const_mut(&mut self, i: &mut TraitItemConst)
		fn visit_trait_item_macro_mut(&mut self, i: &mut TraitItemMacro)
		fn visit_trait_item_method_mut(&mut self, i: &mut TraitItemMethod)
		fn visit_trait_item_type_mut(&mut self, i: &mut TraitItemType)
		fn visit_type_param_mut(&mut self, i: &mut TypeParam)
		fn visit_variant_mut(&mut self, i: &mut Variant)
	}
}
