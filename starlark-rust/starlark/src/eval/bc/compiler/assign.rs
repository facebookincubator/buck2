/*
 * Copyright 2019 The Starlark in Rust Authors.
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Compile assignment lhs.

use gazebo::prelude::*;

use crate::{
    collections::symbol_map::Symbol,
    eval::{
        bc::{
            compiler::expr::write_n_exprs,
            instr_impl::{
                InstrSetArrayIndex, InstrSetObjectField, InstrStoreModuleAndExport, InstrUnpack,
            },
            stack_ptr::{BcSlot, BcSlotIn},
            writer::BcWriter,
        },
        compiler::{scope::Captured, span::IrSpanned, stmt::AssignCompiledValue},
    },
};

impl IrSpanned<AssignCompiledValue> {
    pub(crate) fn write_bc(&self, value: BcSlotIn, bc: &mut BcWriter) {
        let span = self.span;
        match self.node {
            AssignCompiledValue::Dot(ref object, ref field) => {
                object.write_bc_cb(bc, |object, bc| {
                    let symbol = Symbol::new(field.as_str());
                    bc.write_instr::<InstrSetObjectField>(span, (value, object, symbol));
                });
            }
            AssignCompiledValue::ArrayIndirection(ref array, ref index) => {
                write_n_exprs([array, index], bc, |array_index, bc| {
                    bc.write_instr::<InstrSetArrayIndex>(span, (value, array_index));
                });
            }
            AssignCompiledValue::Tuple(ref xs) => {
                // All assignments are to local variables, e. g.
                // ```
                // (x, y, z) = ...
                // ```
                // so we can avoid using intermediate register.
                let all_local = xs
                    .try_map(|x| x.as_local_non_captured().map(|l| l.to_bc_slot()).ok_or(()))
                    .ok();
                if let Some(all_local) = all_local {
                    let args = bc.heap.alloc_any_slice_display_from_debug(&all_local);
                    bc.write_instr::<InstrUnpack>(span, (value, args));
                } else {
                    bc.alloc_slots(xs.len() as u32, |slots, bc| {
                        let args: Vec<BcSlot> = slots.iter().collect();
                        let args = bc.heap.alloc_any_slice_display_from_debug(&args);
                        bc.write_instr::<InstrUnpack>(span, (value, args));

                        for (x, slot) in xs.iter().zip(slots.iter()) {
                            x.write_bc(slot.to_in(), bc);
                        }
                    });
                }
            }
            AssignCompiledValue::Local(slot, Captured::No) => {
                bc.write_store_local(span, value, slot.to_bc_slot());
            }
            AssignCompiledValue::Local(slot, Captured::Yes) => {
                bc.write_store_local_captured(span, value, slot);
            }
            AssignCompiledValue::Module(slot, ref name) => {
                bc.write_instr::<InstrStoreModuleAndExport>(span, (value, slot, name.clone()));
            }
        }
    }
}
