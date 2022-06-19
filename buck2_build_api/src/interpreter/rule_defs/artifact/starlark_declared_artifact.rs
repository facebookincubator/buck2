/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt::{self, Debug, Display};

use anyhow::{anyhow, Context};
use buck2_core::{
    fs::paths::ForwardRelativePath,
    provider::{ConfiguredProvidersLabel, ProvidersName},
};
use buck2_interpreter::types::label::Label;
use gazebo::{any::ProvidesStaticType, prelude::*};
use starlark::{
    codemap::FileSpan,
    collections::StarlarkHasher,
    environment::{Methods, MethodsBuilder, MethodsStatic},
    starlark_type,
    values::{
        AllocValue, Freeze, Freezer, Heap, NoSerialize, StarlarkValue, StringValue, Trace,
        UnpackValue, Value, ValueLike,
    },
};

use crate::{
    actions::{
        artifact,
        artifact::{Artifact, ArtifactPath, OutputArtifact},
    },
    artifact_groups::ArtifactGroup,
    deferred::BaseDeferredKey,
    interpreter::rule_defs::{
        artifact::{StarlarkArtifact, StarlarkArtifactLike, StarlarkOutputArtifact},
        cmd_args::{
            CommandLineArgLike, CommandLineArtifactVisitor, CommandLineBuilder,
            WriteToFileMacroVisitor,
        },
    },
};

/// A wrapper for an artifact that has been declared in the user's implementation function.
///
/// The `DeclaredArtifact` may or may not have been bound yet, but must be by the end of the
/// user's implementation function. This turns into a `StarlarkArtifact` when frozen, and
/// /must/ have had `ensure_bound()` called on it successfully before freezing.
#[derive(Debug, ProvidesStaticType, Trace, NoSerialize)]
pub struct StarlarkDeclaredArtifact {
    // FileSpan is not Hash, and the location is not really relevant for equality.
    // We also expect each artifact to be declared at one unique location anyway.
    #[trace(unsafe_ignore)]
    pub(super) declaration_location: Option<FileSpan>,
    #[trace(unsafe_ignore)]
    pub(super) artifact: artifact::DeclaredArtifact,
}

impl Display for StarlarkDeclaredArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<build artifact {}", self.artifact)?;
        if let Some(location) = &self.declaration_location {
            write!(f, " declared at {}", location)?;
        }
        if let Some(label) = self.artifact.owner() {
            write!(f, " bound to {}", label)?;
        }
        write!(f, ">")
    }
}

impl StarlarkDeclaredArtifact {
    pub fn new(
        declaration_location: Option<FileSpan>,
        artifact: artifact::DeclaredArtifact,
    ) -> Self {
        StarlarkDeclaredArtifact {
            declaration_location,
            artifact,
        }
    }
}

impl StarlarkArtifactLike for StarlarkDeclaredArtifact {
    fn output_artifact(&self) -> anyhow::Result<OutputArtifact> {
        Ok(self.artifact.as_output())
    }

    fn get_bound(&self) -> anyhow::Result<Artifact> {
        Ok(self.artifact.dupe().ensure_bound()?.into_artifact())
    }

    fn as_command_line_like(&self) -> &dyn CommandLineArgLike {
        self
    }

    fn fingerprint(&self) -> ArtifactPath<'_> {
        self.artifact.get_path()
    }
}

impl CommandLineArgLike for StarlarkDeclaredArtifact {
    fn add_to_command_line(&self, _cli: &mut dyn CommandLineBuilder) -> anyhow::Result<()> {
        // TODO: proper error message
        Err(anyhow!(
            "proper error here; we should not be adding mutable starlark objects to clis"
        ))
    }

    fn visit_artifacts(&self, visitor: &mut dyn CommandLineArtifactVisitor) -> anyhow::Result<()> {
        visitor.visit_input(
            ArtifactGroup::Artifact(self.artifact.dupe().ensure_bound()?.into_artifact()),
            None,
        );
        Ok(())
    }

    fn contains_arg_attr(&self) -> bool {
        false
    }

    fn visit_write_to_file_macros(
        &self,
        _visitor: &mut dyn WriteToFileMacroVisitor,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Freeze for StarlarkDeclaredArtifact {
    type Frozen = StarlarkArtifact;
    fn freeze(self, _freezer: &Freezer) -> anyhow::Result<Self::Frozen> {
        let artifact = self
            .artifact
            .ensure_bound()
            .context("Artifact must be bound by the end of the rule")?
            .into_artifact();
        Ok(StarlarkArtifact { artifact })
    }
}

impl<'v> AllocValue<'v> for StarlarkDeclaredArtifact {
    fn alloc_value(self, heap: &'v Heap) -> Value<'v> {
        heap.alloc_complex(self)
    }
}

impl<'v> StarlarkValue<'v> for StarlarkDeclaredArtifact {
    starlark_type!("artifact");

    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(artifact_methods)
    }

    fn equals(&self, other: Value<'v>) -> anyhow::Result<bool> {
        StarlarkArtifactLike::equals(self, other)
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> anyhow::Result<()> {
        StarlarkArtifactLike::write_hash(self, hasher)
    }
}

impl<'v> UnpackValue<'v> for &'v StarlarkDeclaredArtifact {
    fn expected() -> String {
        StarlarkDeclaredArtifact::get_type_value_static()
            .as_str()
            .to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        value.downcast_ref::<StarlarkDeclaredArtifact>()
    }
}

/// A single input or output for an action
#[starlark_module]
fn artifact_methods(builder: &mut MethodsBuilder) {
    /// The base name of this artifact. e.g. for an artifact at `foo/bar`, this is `bar`
    #[starlark(attribute)]
    fn basename<'v>(
        this: &StarlarkDeclaredArtifact,
        heap: &Heap,
    ) -> anyhow::Result<StringValue<'v>> {
        this.artifact
            .get_path()
            .with_filename(|filename| Ok(heap.alloc_str(filename?.as_str())))
    }

    /// The file extension of this artifact. e.g. for an artifact at foo/bar.sh,
    /// this is `.sh`. If no extension is present, `""` is returned.
    #[starlark(attribute)]
    fn extension<'v>(
        this: &StarlarkDeclaredArtifact,
        heap: &Heap,
    ) -> anyhow::Result<StringValue<'v>> {
        this.artifact.get_path().with_filename(|filename| {
            Ok(match filename?.extension() {
                None => heap.alloc_str(""),
                Some(x) => heap.alloc_str_concat(".", x),
            })
        })
    }

    /// Whether the artifact represents a source file
    #[starlark(attribute)]
    fn is_source(this: &StarlarkDeclaredArtifact) -> anyhow::Result<bool> {
        Ok(false)
    }

    /// The `Label` of the rule that originally created this artifact. May also be None in
    /// the case of source files, or if the artifact has not be used in an action, or if the
    /// action was not created by a rule.
    #[starlark(attribute)]
    fn owner<'v>(
        this: &StarlarkDeclaredArtifact,
        heap: &Heap,
    ) -> anyhow::Result<Option<Label<'v>>> {
        match this.artifact.owner() {
            None => Ok(None),
            Some(x) => Ok(match x {
                BaseDeferredKey::TargetLabel(t) => Some(Label::new(
                    heap,
                    ConfiguredProvidersLabel::new(t, ProvidersName::Default),
                )),
                BaseDeferredKey::BxlLabel(_) => None,
            }),
        }
    }

    /// Returns a `StarlarkOutputArtifact` instance, or fails if the artifact is
    /// either an `Artifact`, or is a bound `DeclaredArtifact` (You cannot bind twice)
    fn as_output(this: &StarlarkDeclaredArtifact) -> anyhow::Result<StarlarkOutputArtifact> {
        Ok(StarlarkOutputArtifact::new(this.output_artifact()?))
    }

    /// The interesting part of the path, relative to somewhere in the output directory.
    /// For an artifact declared as `foo/bar`, this is `foo/bar`.
    #[starlark(attribute)]
    fn short_path<'v>(
        this: &StarlarkDeclaredArtifact,
        heap: &Heap,
    ) -> anyhow::Result<StringValue<'v>> {
        this.artifact
            .get_path()
            .with_short_path(|short_path| Ok(heap.alloc_str(short_path.as_str())))
    }

    fn project<'v>(
        this: &'v StarlarkDeclaredArtifact,
        path: &str,
    ) -> anyhow::Result<StarlarkDeclaredArtifact> {
        let path = ForwardRelativePath::new(path)?;
        // Not sure if this.declaration_location is or the project() call is more appropriate here.
        Ok(StarlarkDeclaredArtifact::new(
            this.declaration_location.dupe(),
            this.artifact.project(path),
        ))
    }
}
