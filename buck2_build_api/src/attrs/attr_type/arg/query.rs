/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt::Display;
use std::fmt::{self};

use buck2_core::target::ConfiguredTargetLabel;
use gazebo::prelude::*;
use starlark::values::FrozenRef;

use crate::artifact_groups::ArtifactGroup;
use crate::attrs::attr_type::arg::value::add_output_to_arg;
use crate::attrs::attr_type::arg::ArgBuilder;
use crate::attrs::attr_type::arg::QueryExpansion;
use crate::attrs::attr_type::attr_literal::AttrConfig;
use crate::attrs::attr_type::attr_literal::ConfiguredAttrTraversal;
use crate::attrs::attr_type::query::QueryAttrBase;
use crate::attrs::AttrConfigurationContext;
use crate::attrs::AttrResolutionContext;
use crate::attrs::CoercedAttr;
use crate::attrs::CoercedAttrTraversal;
use crate::attrs::ConfiguredAttr;
use crate::interpreter::rule_defs::artifact::StarlarkArtifact;
use crate::interpreter::rule_defs::artifact::StarlarkArtifactLike;
use crate::interpreter::rule_defs::cmd_args::CommandLineArtifactVisitor;

#[derive(Debug, PartialEq)]
pub enum ResolvedQueryMacro {
    Outputs(Vec<Vec<FrozenRef<'static, StarlarkArtifact>>>),
    Targets(Vec<ConfiguredTargetLabel>),
    TargetsAndOutputs(
        String,
        Vec<(
            ConfiguredTargetLabel,
            Vec<FrozenRef<'static, StarlarkArtifact>>,
        )>,
    ),
}

impl Display for ResolvedQueryMacro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(ndmitchell): Include the information in the format output
        match self {
            ResolvedQueryMacro::Outputs(_) => {
                write!(f, "$(query_outputs ...)")
            }
            ResolvedQueryMacro::Targets(_) => write!(f, "$(query_targets ...)"),

            ResolvedQueryMacro::TargetsAndOutputs(_, _) => {
                write!(f, "$(query_targets_and_outputs ...)")
            }
        }
    }
}

impl ResolvedQueryMacro {
    pub fn add_to_arg(&self, builder: &mut dyn ArgBuilder) -> anyhow::Result<()> {
        match self {
            Self::Outputs(list) => {
                let mut first = true;
                for target_outputs in list.iter() {
                    for output in target_outputs.iter() {
                        if !first {
                            builder.push_str(" ");
                        }
                        first = false;
                        add_output_to_arg(builder, output)?;
                    }
                }
            }
            Self::TargetsAndOutputs(separator, list) => {
                let mut first = true;
                for (target, target_outputs) in list.iter() {
                    for output in target_outputs.iter() {
                        if !first {
                            builder.push_str(separator);
                        }
                        first = false;
                        builder.push_str(&target.unconfigured().to_string());
                        builder.push_str(separator);
                        add_output_to_arg(builder, output)?;
                    }
                }
            }
            Self::Targets(list) => {
                // This is defined to add the plain (unconfigured) labels.
                for (i, target) in list.iter().enumerate() {
                    if i != 0 {
                        builder.push_str(" ");
                    }
                    builder.push_str(&target.unconfigured().to_string());
                }
            }
        }
        Ok(())
    }

    pub fn visit_artifacts(
        &self,
        visitor: &mut dyn CommandLineArtifactVisitor,
    ) -> anyhow::Result<()> {
        match self {
            Self::Outputs(list) => {
                for target_outputs in list.iter() {
                    for artifact in target_outputs.iter() {
                        visitor.visit_input(ArtifactGroup::Artifact(artifact.get_bound()?), None);
                    }
                }
            }
            Self::TargetsAndOutputs(_, list) => {
                for (_, target_outputs) in list.iter() {
                    for artifact in target_outputs.iter() {
                        visitor.visit_input(ArtifactGroup::Artifact(artifact.get_bound()?), None);
                    }
                }
            }
            Self::Targets(..) => {
                // no inputs
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct QueryMacroBase<C: AttrConfig> {
    expansion_type: QueryExpansion,
    query: QueryAttrBase<C>,
}

impl QueryMacroBase<CoercedAttr> {
    pub fn new(expansion_type: QueryExpansion, query: QueryAttrBase<CoercedAttr>) -> Self {
        Self {
            expansion_type,
            query,
        }
    }

    pub(crate) fn traverse<'a>(
        &'a self,
        traversal: &mut dyn CoercedAttrTraversal<'a>,
    ) -> anyhow::Result<()> {
        self.query.traverse(traversal)
    }

    pub(crate) fn configure(
        &self,
        ctx: &dyn AttrConfigurationContext,
    ) -> anyhow::Result<QueryMacroBase<ConfiguredAttr>> {
        Ok(QueryMacroBase {
            expansion_type: self.expansion_type.clone(),
            query: self.query.configure(ctx)?,
        })
    }
}

impl QueryMacroBase<ConfiguredAttr> {
    pub fn traverse<'a>(
        &'a self,
        traversal: &mut dyn ConfiguredAttrTraversal<'a>,
    ) -> anyhow::Result<()> {
        self.query.traverse(traversal)
    }

    pub(crate) fn resolve(
        &self,
        ctx: &dyn AttrResolutionContext,
    ) -> anyhow::Result<ResolvedQueryMacro> {
        let query_result = self.query.resolve(ctx)?;

        match &self.expansion_type {
            QueryExpansion::Output => Ok(ResolvedQueryMacro::Outputs(query_result.map(
                |(_, providers)| {
                    providers
                        .provider_collection()
                        .default_info()
                        .default_outputs()
                },
            ))),
            QueryExpansion::Target => Ok(ResolvedQueryMacro::Targets(
                query_result.map(|(target, _)| target.dupe()),
            )),
            QueryExpansion::TargetAndOutput(separator) => {
                Ok(ResolvedQueryMacro::TargetsAndOutputs(
                    match separator {
                        Some(separator) => separator.to_owned(),
                        None => " ".to_owned(),
                    },
                    query_result.map(|(target, providers)| {
                        (
                            target.dupe(),
                            providers
                                .provider_collection()
                                .default_info()
                                .default_outputs(),
                        )
                    }),
                ))
            }
        }
    }
}

impl<C: AttrConfig> Display for QueryMacroBase<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.expansion_type, self.query.query())?;
        Ok(())
    }
}
