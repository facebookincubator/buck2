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

use crate::{
    eval::{
        bc::{
            bytecode::Bc,
            compiler::if_compiler::{write_if_else, write_if_then},
            instr_impl::{
                InstrBeforeStmt, InstrBreak, InstrContinue, InstrPossibleGc, InstrReturn,
                InstrReturnCheckType, InstrReturnConst,
            },
            writer::BcWriter,
        },
        compiler::{
            expr::{ExprCompiled, MaybeNot},
            span::IrSpanned,
            stmt::{AssignCompiledValue, StmtCompileContext, StmtCompiled, StmtsCompiled},
        },
        runtime::call_stack::FrozenFileSpan,
    },
    values::{FrozenHeap, FrozenValue},
};

pub(crate) fn write_for(
    over: &IrSpanned<ExprCompiled>,
    var: &IrSpanned<AssignCompiledValue>,
    span: FrozenFileSpan,
    bc: &mut BcWriter,
    body: impl FnOnce(&mut BcWriter),
) {
    over.write_bc_cb(bc, |over, bc| {
        if let Some(var) = var.as_local_non_captured() {
            // Typical case: `for x in ...: ...`,
            // compile loop assignment directly to a local variable.
            bc.write_for(over, var.to_bc_slot(), span, body)
        } else {
            // General case, e. g. `for (x, y[0]) in ...: ...`,
            // compile loop assignment to a temporary variable,
            // and reassign it in the loop body.
            bc.alloc_slot(|var_slot, bc| {
                bc.write_for(over, var_slot, span, |bc| {
                    var.write_bc(var_slot, bc);
                    body(bc);
                })
            })
        }
    })
}

impl StmtsCompiled {
    pub(crate) fn write_bc(&self, compiler: &StmtCompileContext, bc: &mut BcWriter) {
        for stmt in self.stmts() {
            stmt.write_bc(compiler, bc);
        }
    }
}

impl IrSpanned<StmtCompiled> {
    fn write_bc(&self, compiler: &StmtCompileContext, bc: &mut BcWriter) {
        if compiler.has_before_stmt {
            match self.node {
                StmtCompiled::PossibleGc => {}
                _ => bc.write_instr::<InstrBeforeStmt>(self.span, self.span),
            }
        }
        self.write_bc_inner(compiler, bc);
    }

    fn write_if_then(
        compiler: &StmtCompileContext,
        bc: &mut BcWriter,
        c: &IrSpanned<ExprCompiled>,
        maybe_not: MaybeNot,
        t: &dyn Fn(&StmtCompileContext, &mut BcWriter),
    ) {
        write_if_then(
            c,
            maybe_not,
            |bc| {
                t(compiler, bc);
            },
            bc,
        );
    }

    fn write_if_else(
        c: &IrSpanned<ExprCompiled>,
        t: &StmtsCompiled,
        f: &StmtsCompiled,
        compiler: &StmtCompileContext,
        bc: &mut BcWriter,
    ) {
        assert!(!t.is_empty() || !f.is_empty());
        if f.is_empty() {
            Self::write_if_then(compiler, bc, c, MaybeNot::Id, &|compiler, bc| {
                t.write_bc(compiler, bc)
            });
        } else if t.is_empty() {
            Self::write_if_then(compiler, bc, c, MaybeNot::Not, &|compiler, bc| {
                f.write_bc(compiler, bc)
            });
        } else {
            write_if_else(
                c,
                |bc| t.write_bc(compiler, bc),
                |bc| f.write_bc(compiler, bc),
                bc,
            );
        }
    }

    #[allow(clippy::collapsible_else_if)]
    fn write_return(
        span: FrozenFileSpan,
        expr: &IrSpanned<ExprCompiled>,
        compiler: &StmtCompileContext,
        bc: &mut BcWriter,
    ) {
        if compiler.has_return_type {
            expr.write_bc_cb(bc, |slot, bc| {
                bc.write_instr::<InstrReturnCheckType>(span, slot);
            });
        } else {
            if let Some(value) = expr.as_value() {
                bc.write_instr::<InstrReturnConst>(span, value);
            } else {
                expr.write_bc_cb(bc, |slot, bc| {
                    bc.write_instr::<InstrReturn>(span, slot);
                });
            }
        }
    }

    fn write_bc_inner(&self, compiler: &StmtCompileContext, bc: &mut BcWriter) {
        let span = self.span;
        match self.node {
            StmtCompiled::PossibleGc => bc.write_instr::<InstrPossibleGc>(span, ()),
            StmtCompiled::Return(ref expr) => Self::write_return(span, expr, compiler, bc),
            StmtCompiled::Expr(ref expr) => {
                expr.write_bc_for_effect(bc);
            }
            StmtCompiled::Assign(ref lhs, ref rhs) => {
                if let Some(local) = lhs.as_local_non_captured() {
                    // Write expression directly to local slot.
                    rhs.write_bc(local.to_bc_slot(), bc);
                } else {
                    rhs.write_bc_cb(bc, |slot, bc| {
                        lhs.write_bc(slot, bc);
                    });
                }
            }
            StmtCompiled::AssignModify(ref lhs, op, ref rhs) => {
                lhs.write_bc(span, op, rhs, bc);
            }
            StmtCompiled::If(box (ref c, ref t, ref f)) => {
                Self::write_if_else(c, t, f, compiler, bc);
            }
            StmtCompiled::For(box (ref assign, ref over, ref body)) => {
                write_for(over, assign, span, bc, |bc| body.write_bc(compiler, bc));
            }
            StmtCompiled::Break => {
                bc.write_instr::<InstrBreak>(span, ());
            }
            StmtCompiled::Continue => {
                bc.write_instr::<InstrContinue>(span, ());
            }
        }
    }
}

impl StmtsCompiled {
    pub(crate) fn as_bc(
        &self,
        compiler: &StmtCompileContext,
        local_count: u32,
        param_count: u32,
        heap: &FrozenHeap,
    ) -> Bc {
        let mut bc = BcWriter::new(compiler.bc_profile, local_count, param_count, heap);
        self.write_bc(compiler, &mut bc);

        // Small optimization: if the last statement is return,
        // we do not need to write another return.
        if !matches!(self.last().map(|s| &s.node), Some(StmtCompiled::Return(..))) {
            let span = self.last().map(|s| s.span.end_span()).unwrap_or_default();
            if compiler.has_return_type {
                bc.alloc_slot(|slot, bc| {
                    bc.write_const(span, FrozenValue::new_none(), slot);
                    bc.write_instr::<InstrReturnCheckType>(span, slot);
                });
            } else {
                bc.write_instr::<InstrReturnConst>(span, FrozenValue::new_none());
            }
        }

        bc.finish()
    }
}
