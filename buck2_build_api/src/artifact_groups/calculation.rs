/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::collections::HashSet;

use anyhow::Context as _;
use async_recursion::async_recursion;
use async_trait::async_trait;
use buck2_common::{
    dice::file_ops::HasFileOps,
    file_ops::{FileOps, PathMetadata},
};
use buck2_core::{directory::DirectoryData, result::SharedResult};
use derive_more::Display;
use dice::{DiceComputations, Key};
use futures::stream::FuturesOrdered;
use gazebo::prelude::*;
use smallvec::SmallVec;

use crate::{
    actions::{
        artifact::{Artifact, ArtifactKind, ArtifactValue, BaseArtifactKind},
        build_listener::{HasBuildSignals, TransitiveSetComputationSignal},
        calculation::ActionCalculation,
        directory::{ActionDirectoryEntry, ActionDirectoryMember, ActionSharedDirectory, INTERNER},
    },
    artifact_groups::{ArtifactGroup, ArtifactGroupValues, TransitiveSetProjectionKey},
    deferred::calculation::DeferredCalculation,
    keep_going,
    path::BuckPath,
};

#[async_trait]
pub(crate) trait ArtifactGroupCalculation {
    /// Makes an 'Artifact' available to be accessed
    async fn ensure_artifact_group(
        &self,
        input: &ArtifactGroup,
    ) -> anyhow::Result<ArtifactGroupValues>;
}

#[async_trait]
impl ArtifactGroupCalculation for DiceComputations {
    /// makes the 'Artifact' available to be accessed
    async fn ensure_artifact_group(
        &self,
        input: &ArtifactGroup,
    ) -> anyhow::Result<ArtifactGroupValues> {
        // TODO consider if we need to cache this

        let res = match input {
            ArtifactGroup::Artifact(artifact) => {
                let value = ensure_artifact(self, artifact).await?;
                ArtifactGroupValues::from_artifact(artifact.dupe(), value)
            }
            ArtifactGroup::TransitiveSetProjection(key) => {
                self.compute(&EnsureTransitiveSetProjectionKey(key.dupe()))
                    .await?
            }
        };

        Ok(res)
    }
}

#[async_recursion]
async fn path_artifact_value(
    file_ops: &dyn FileOps,
    path: &BuckPath,
) -> anyhow::Result<ActionDirectoryEntry<ActionSharedDirectory>> {
    let cell_path = path.to_cell_path();
    match file_ops.read_path_metadata(&cell_path).await? {
        PathMetadata::ExternalSymlink(symlink) => Ok(ActionDirectoryEntry::Leaf(
            ActionDirectoryMember::ExternalSymlink(symlink),
        )),
        PathMetadata::File(metadata) => Ok(ActionDirectoryEntry::Leaf(
            ActionDirectoryMember::File(metadata),
        )),
        PathMetadata::Directory => {
            let files = file_ops.read_dir(&cell_path).await?;
            let mut entries = Vec::with_capacity(files.len());
            for x in &*files {
                let path_child = BuckPath::new(
                    path.package().dupe(),
                    path.path().join_unnormalized(&x.file_name),
                );
                let value = path_artifact_value(file_ops, &path_child).await?;
                entries.push((x.file_name.clone(), value));
            }
            let d: DirectoryData<_, _, _> = DirectoryData::new(entries.into_iter().collect());
            Ok(ActionDirectoryEntry::Dir(INTERNER.intern(d)))
        }
    }
}

async fn ensure_base_artifact(
    dice: &DiceComputations,
    artifact: &BaseArtifactKind,
) -> anyhow::Result<ArtifactValue> {
    match artifact {
        BaseArtifactKind::Build(ref built) => {
            let action_result = dice.build_artifact(built).await?;
            if let Some(value) = action_result.get(built) {
                Ok(value.dupe())
            } else {
                panic!(
                    "Building an artifact didn't produce it. Expected `{:?}` but only have `{:?}`",
                    artifact, action_result
                )
            }
        }
        BaseArtifactKind::Source(ref source) => {
            Ok(path_artifact_value(&dice.file_ops(), source.get_path())
                .await?
                .into())
        }
    }
}

async fn ensure_artifact(
    dice: &DiceComputations,
    artifact: &Artifact,
) -> anyhow::Result<ArtifactValue> {
    match artifact.0.as_ref() {
        ArtifactKind::Base(ref base) => ensure_base_artifact(dice, base).await,
        // TODO (@torozco) implement this
        ArtifactKind::Projected(..) => Err(anyhow::anyhow!("Not implemented yet")),
    }
}

#[derive(Clone, Dupe, Eq, PartialEq, Hash, Display, Debug)]
#[display(fmt = "EnsureTransitiveSetProjectionKey({})", .0)]
struct EnsureTransitiveSetProjectionKey(TransitiveSetProjectionKey);

#[async_trait]
impl Key for EnsureTransitiveSetProjectionKey {
    type Value = SharedResult<ArtifactGroupValues>;

    async fn compute(&self, ctx: &DiceComputations) -> Self::Value {
        let set = ctx
            .compute_deferred_data(&self.0.key)
            .await
            .context("Failed to compute deferred")?;

        let sub_inputs = set
            .as_transitive_set()?
            .get_projection_sub_inputs(self.0.projection)?;

        // Partition our inputs in artifacts and projections.
        let mut artifacts = Vec::new();
        let mut projections = Vec::new();

        for input in sub_inputs {
            match input {
                ArtifactGroup::Artifact(a) => artifacts.push(a),
                ArtifactGroup::TransitiveSetProjection(key) => {
                    projections.push(EnsureTransitiveSetProjectionKey(key))
                }
            };
        }

        // Compute the new inputs. Note that ordering here (and below) is important to ensure
        // stability of the ArtifactGroupValues we produce across executions, so we use
        // FuturesOrdered.

        let values = keep_going::try_join_all(
            artifacts
                .into_iter()
                .map(|a| async move {
                    let value = ensure_artifact(ctx, &a).await?;
                    anyhow::Ok((a, value))
                })
                .collect::<FuturesOrdered<_>>(),
        );

        let children = keep_going::try_join_all(
            projections
                .iter()
                .map(|key| async move { Ok(ctx.compute(key).await?) })
                .collect::<FuturesOrdered<_>>(),
        );

        let (values, children): (SmallVec<[_; 1]>, Vec<_>) =
            keep_going::try_join(values, children).await?;

        if let Some(build_signals) = ctx.per_transaction_data().get_build_signals() {
            let artifacts = values
                .iter()
                .filter_map(|(artifact, _value)| artifact.action_key().duped())
                .collect::<HashSet<_>>();

            let set_deps = projections.into_iter().map(|p| p.0).collect::<HashSet<_>>();

            build_signals.signal(TransitiveSetComputationSignal {
                key: self.0.dupe(),
                artifacts,
                set_deps,
            });
        }

        let artifact_fs = crate::calculation::Calculation::get_artifact_fs(ctx).await;
        let values = ArtifactGroupValues::new(values, children, &artifact_fs)
            .context("Failed to construct ArtifactGroupValues")?;

        Ok(values)
    }

    fn equality(x: &Self::Value, y: &Self::Value) -> bool {
        match (x, y) {
            (Ok(x), Ok(y)) => x.shallow_equals(y),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use buck2_common::{
        dice::{
            cells::HasCellResolver, data::testing::SetTestingIoProvider,
            file_ops::testing::FileOpsKey,
        },
        file_ops::{testing::TestFileOps, FileDigest, FileMetadata, TrackedFileDigest},
    };
    use buck2_core::{
        cells::{
            paths::{CellPath, CellRelativePathBuf},
            testing::CellResolverExt,
            CellName, CellResolver,
        },
        fs::project::{ProjectFilesystemTemp, ProjectRelativePathBuf},
        package::{testing::PackageExt, Package, PackageRelativePathBuf},
        result::ToSharedResultExt,
    };
    use dice::{testing::DiceBuilder, UserComputationData};
    use indoc::indoc;
    use maplit::btreemap;
    use starlark::values::OwnedFrozenValue;

    use super::*;
    use crate::{
        actions::artifact::{Artifact, ArtifactValue, SourceArtifact},
        artifact_groups::deferred::DeferredTransitiveSetData,
        context::SetBuildContextData,
        deferred::{calculation::testing::DeferredResolve, AnyValue},
        interpreter::rule_defs::transitive_set::{testing, TransitiveSet},
    };

    fn mock_deferred_tset(dice_builder: DiceBuilder, value: OwnedFrozenValue) -> DiceBuilder {
        let tset = TransitiveSet::from_value(value.value()).unwrap();
        let resolve = DeferredResolve(tset.key().deferred_key().dupe());

        let data: Arc<dyn AnyValue + 'static> = Arc::new(DeferredTransitiveSetData(value));
        dice_builder.mock_and_return(resolve, anyhow::Ok(data).shared_error())
    }

    #[tokio::test]
    async fn test_ensure_artifact_group() -> anyhow::Result<()> {
        let set = testing::new_transitive_set(indoc!(
            r#"
            def project(args, value):
                args.add(value)

            TestSet = transitive_set(args_projections = {
                "project": project
            })

            foo = source_artifact("foo", "foo")
            bar = source_artifact("bar", "bar")

            s1 = make_tset(TestSet, value = foo)
            make_tset(TestSet, value = bar, children = [s1])
            "#
        ))?;

        let heap = set.owner();

        let cell_resolver = CellResolver::of_names_and_paths(&[(
            CellName::unchecked_new("".into()),
            ProjectRelativePathBuf::unchecked_new("cell-path".into()),
        )]);

        let foo = CellPath::new(
            CellName::unchecked_new("".to_owned()),
            CellRelativePathBuf::unchecked_new("foo/foo".to_owned()),
        );

        let foo_artifact = Artifact::from(SourceArtifact::new(BuckPath::new(
            Package::testing_new("", "foo"),
            PackageRelativePathBuf::unchecked_new("foo".to_owned()),
        )));

        let foo_meta = FileMetadata {
            digest: TrackedFileDigest::new(FileDigest::from_bytes("foo".as_bytes())),
            is_executable: true,
        };

        let bar_artifact = Artifact::from(SourceArtifact::new(BuckPath::new(
            Package::testing_new("", "bar"),
            PackageRelativePathBuf::unchecked_new("bar".to_owned()),
        )));

        let bar = CellPath::new(
            CellName::unchecked_new("".to_owned()),
            CellRelativePathBuf::unchecked_new("bar/bar".to_owned()),
        );

        let bar_meta = FileMetadata {
            digest: TrackedFileDigest::new(FileDigest::from_bytes("bar".as_bytes())),
            is_executable: true,
        };

        let files = TestFileOps::new_with_files_metadata(btreemap![
            foo => foo_meta.dupe(),
            bar => bar_meta.dupe(),
        ]);

        let fs = ProjectFilesystemTemp::new()?;

        let mut dice_builder = DiceBuilder::new()
            .mock_and_return(FileOpsKey(), Ok(Arc::new(files)))
            .set_data(|data| data.set_testing_io_provider(&fs));

        // Register all the sets as deferreds.
        dice_builder = mock_deferred_tset(dice_builder, set.to_owned_frozen_value());

        // This is kinda clowny, but we can't upcast the TransitiveSetGen back to a Value so we
        // have to access Values from their parents.
        for set in set.as_ref().iter() {
            for child in set.children.iter() {
                // Safety: We know the entire set came from the same heap.
                let child = unsafe { OwnedFrozenValue::new(heap.dupe(), *child) };
                dice_builder = mock_deferred_tset(dice_builder, child);
            }
        }

        let dice = dice_builder.build(UserComputationData::new());
        dice.set_cell_resolver(cell_resolver);
        dice.set_buck_out_path(None);
        let dice = dice.commit();

        let result = dice
            .ensure_artifact_group(&ArtifactGroup::TransitiveSetProjection(
                TransitiveSetProjectionKey {
                    key: set.key.dupe(),
                    projection: 0,
                },
            ))
            .await?
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(
            &result,
            &[
                (
                    bar_artifact,
                    ArtifactValue::file(FileMetadata {
                        digest: bar_meta.digest,
                        is_executable: bar_meta.is_executable,
                    })
                ),
                (
                    foo_artifact,
                    ArtifactValue::file(FileMetadata {
                        digest: foo_meta.digest,
                        is_executable: foo_meta.is_executable,
                    })
                ),
            ]
        );

        Ok(())
    }
}
