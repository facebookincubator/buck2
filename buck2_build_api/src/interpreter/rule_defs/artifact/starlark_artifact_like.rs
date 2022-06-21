/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt::Display;
use std::hash::Hash;

use starlark::collections::StarlarkHasher;
use starlark::values::Value;
use starlark::values::ValueLike;

use crate::actions::artifact::Artifact;
use crate::actions::artifact::ArtifactPath;
use crate::actions::artifact::OutputArtifact;
use crate::interpreter::rule_defs::artifact::StarlarkArtifact;
use crate::interpreter::rule_defs::artifact::StarlarkDeclaredArtifact;
use crate::interpreter::rule_defs::cmd_args::CommandLineArgLike;

/// The Starlark representation of an `Artifact`
///
/// The following fields are available in Starlark:
/// `.basename`: The base name of this artifact. e.g. for an artifact
///              at `foo/bar`, this is `bar`
/// `.extension`: The file extension of this artifact. e.g. for an artifact at foo/bar.sh,
///               this is `sh`. If no extension is present, an empty string is returned
/// `.is_source`: Whether the artifact represents a source file
/// `.owner`: The `Label` of the rule that originally created this artifact. May also be None in
///           the case of source files, or if the artifact has not be used in an action.
/// `as_output()`: Returns a `StarlarkOutputArtifact` instance, or fails if the artifact is
///                either an `Artifact`, or is a bound `DeclaredArtifact` (You cannot bind twice)
/// `.short_path`: The interesting part of the path, relative to somewhere in the output directory.
///                For an artifact declared as `foo/bar`, this is `foo/bar`.
/// This trait also has some common functionality for `StarlarkValue` that we want shared between
/// `StarlarkArtifact` and `StarlarkDeclaredArtifact`
pub trait StarlarkArtifactLike: Display {
    /// The contained artifact as an `OutputArtifact`, or error if that conversion is impossible
    fn output_artifact(&self) -> anyhow::Result<OutputArtifact>;

    /// Gets the bound artifact, or errors if the artifact is not bound
    fn get_bound(&self) -> anyhow::Result<Artifact>;

    fn equals<'v>(&self, other: Value<'v>) -> anyhow::Result<bool> {
        if let Some(other) = other.downcast_ref::<StarlarkArtifact>() {
            Ok(self.fingerprint() == other.fingerprint())
        } else if let Some(other) = other.downcast_ref::<StarlarkDeclaredArtifact>() {
            Ok(self.fingerprint() == other.fingerprint())
        } else {
            Ok(false)
        }
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> anyhow::Result<()> {
        self.fingerprint().hash(hasher);
        Ok(())
    }

    /// Return an interface for frozen and bound artifacts (`StarlarkArtifact`) to add to a CLI
    ///
    /// Returns None if this artifact isn't the correct type to be added to a CLI object
    fn as_command_line_like(&self) -> &dyn CommandLineArgLike;

    /// It's very important that the Hash/Eq of the StarlarkArtifactLike things doesn't change
    /// during freezing, otherwise Starlark invariants are broken. Use the fingerprint
    /// as the inputs to Hash/Eq to ensure they are consistent
    fn fingerprint(&self) -> ArtifactPath<'_>;
}

pub trait ValueAsArtifactLike<'v> {
    fn as_artifact(&self) -> Option<&'v dyn StarlarkArtifactLike>;
}

impl<'v, V: ValueLike<'v>> ValueAsArtifactLike<'v> for V {
    fn as_artifact(&self) -> Option<&'v dyn StarlarkArtifactLike> {
        self.downcast_ref::<StarlarkArtifact>()
            .map(|o| o as &dyn StarlarkArtifactLike)
            .or_else(|| {
                self.downcast_ref::<StarlarkDeclaredArtifact>()
                    .map(|o| o as &dyn StarlarkArtifactLike)
            })
    }
}
