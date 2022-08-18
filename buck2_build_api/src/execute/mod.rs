/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

pub mod blocking;
pub mod commands;
pub mod materializer;

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use blocking::BlockingExecutor;
use blocking::HasBlockingExecutor;
use blocking::IoRequest;
use buck2_common::dice::data::HasIoProvider;
use buck2_core::directory::unordered_entry_walk;
use buck2_core::directory::DirectoryEntry;
use buck2_core::fs::project::ProjectRelativePath;
use buck2_core::fs::project::ProjectRelativePathBuf;
use buck2_core::fs::project::ProjectRoot;
use buck2_interpreter::dice::HasEvents;
use buck2_node::execute::config::CommandExecutorConfig;
use derivative::Derivative;
use derive_more::Display;
use dice::DiceComputations;
use events::dispatch::EventDispatcher;
use gazebo::prelude::*;
use indexmap::indexmap;
use indexmap::IndexMap;
use indexmap::IndexSet;
use thiserror::Error;

use crate::actions::artifact::ArtifactFs;
use crate::actions::artifact::ArtifactValue;
use crate::actions::artifact::BuildArtifact;
use crate::actions::artifact::ExecutorFs;
use crate::actions::directory::ActionDirectoryMember;
use crate::actions::run::knobs::HasRunActionKnobs;
use crate::actions::run::knobs::RunActionKnobs;
use crate::actions::ActionExecutable;
use crate::actions::ActionExecutionCtx;
use crate::actions::RegisteredAction;
use crate::artifact_groups::ArtifactGroup;
use crate::artifact_groups::ArtifactGroupValues;
use crate::calculation::Calculation;
use crate::execute::commands::dice_data::HasCommandExecutor;
use crate::execute::commands::re::client::ActionDigest;
use crate::execute::commands::ClaimManager;
use crate::execute::commands::CommandExecutionKind;
use crate::execute::commands::CommandExecutionManager;
use crate::execute::commands::CommandExecutionOutput;
use crate::execute::commands::CommandExecutionReport;
use crate::execute::commands::CommandExecutionRequest;
use crate::execute::commands::CommandExecutionResult;
use crate::execute::commands::CommandExecutionStatus;
use crate::execute::commands::CommandExecutionTarget;
use crate::execute::commands::CommandExecutor;
use crate::execute::materializer::HasMaterializer;
use crate::execute::materializer::Materializer;

#[derive(Debug)]
pub enum ExecuteError {
    MissingOutputs {
        wanted: Vec<ProjectRelativePathBuf>,
    },

    MismatchedOutputs {
        wanted: Vec<ProjectRelativePathBuf>,
        got: Vec<ProjectRelativePathBuf>,
    },

    Error {
        error: anyhow::Error,
    },

    CommandExecutionError,
}

impl From<anyhow::Error> for ExecuteError {
    fn from(error: anyhow::Error) -> Self {
        if error.is::<CommandExecutionErrorMarker>() {
            return Self::CommandExecutionError;
        }
        Self::Error { error }
    }
}

#[derive(Error, Debug)]
#[error("Command execution failed. Details are in the command report.")]
pub struct CommandExecutionErrorMarker;

/// This is the result of the action as exposed to other things in the dice computation.
#[derive(Clone, Dupe, Debug, PartialEq, Eq)]
pub struct ActionOutputs(Arc<ActionOutputsData>);

#[derive(Derivative, Debug)]
#[derivative(PartialEq, Eq)]
struct ActionOutputsData {
    outputs: IndexMap<BuildArtifact, ArtifactValue>,
}

#[derive(Copy, Dupe, Clone, Debug, PartialEq, Eq)]
pub struct ActionExecutionTimingData {
    pub wall_time: Duration,
}

impl Default for ActionExecutionTimingData {
    fn default() -> Self {
        Self {
            wall_time: Duration::ZERO,
        }
    }
}

/// Metadata associated with the execution of this action.
#[derive(Debug)]
pub struct ActionExecutionMetadata {
    pub execution_kind: ActionExecutionKind,
    pub timing: ActionExecutionTimingData,
}

/// The *way* that a particular action was executed.
#[derive(Debug, Display, Clone)]
pub enum ActionExecutionKind {
    #[display(fmt = "command({})", self.0)]
    Command(CommandExecutionKind),
    /// This action is simple and executed inline within buck2 (e.g. write, symlink_dir)
    #[display(fmt = "simple")]
    Simple,
    /// This action was not executed at all.
    #[display(fmt = "skipped")]
    Skipped,
    /// This action logically executed, but didn't do all the work.
    #[display(fmt = "deferred")]
    Deferred,
}

impl ActionExecutionKind {
    pub fn as_enum(&self) -> buck2_data::ActionExecutionKind {
        match self {
            ActionExecutionKind::Command(command) => command.as_enum(),
            ActionExecutionKind::Simple => buck2_data::ActionExecutionKind::Simple,
            ActionExecutionKind::Skipped => buck2_data::ActionExecutionKind::Skipped,
            ActionExecutionKind::Deferred => buck2_data::ActionExecutionKind::Deferred,
        }
    }
}

impl ActionOutputs {
    pub fn new(outputs: IndexMap<BuildArtifact, ArtifactValue>) -> Self {
        Self(Arc::new(ActionOutputsData { outputs }))
    }

    pub fn from_single(artifact: BuildArtifact, value: ArtifactValue) -> Self {
        Self::new(indexmap! {artifact => value})
    }

    pub fn get(&self, artifact: &BuildArtifact) -> Option<&ArtifactValue> {
        self.0.outputs.get(artifact)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&BuildArtifact, &ArtifactValue)> {
        self.0.outputs.iter()
    }

    pub fn calc_output_bytes(&self) -> u64 {
        let mut output_bytes = 0;
        for output in &self.0.outputs {
            let mut walk = unordered_entry_walk(output.1.entry().as_ref());
            while let Some((_path, entry)) = walk.next() {
                match entry {
                    DirectoryEntry::Leaf(ActionDirectoryMember::File(f)) => {
                        output_bytes += f.digest.size();
                    }
                    _ => {}
                }
            }
        }

        output_bytes
    }
}

/// Executes 'Actions'
#[async_trait]
pub trait ActionExecutor: Send + Sync {
    async fn execute(
        &self,
        inputs: IndexMap<ArtifactGroup, ArtifactGroupValues>,
        action: &RegisteredAction,
    ) -> (
        Result<(ActionOutputs, ActionExecutionMetadata), ExecuteError>,
        Vec<CommandExecutionReport>,
    );
}

#[async_trait]
pub trait HasActionExecutor {
    async fn get_action_executor(
        &self,
        config: &CommandExecutorConfig,
    ) -> anyhow::Result<Arc<dyn ActionExecutor>>;
}

#[async_trait]
impl HasActionExecutor for DiceComputations {
    async fn get_action_executor(
        &self,
        executor_config: &CommandExecutorConfig,
    ) -> anyhow::Result<Arc<dyn ActionExecutor>> {
        let artifact_fs = self.get_artifact_fs().await?;
        let io_provider = self.global_data().get_io_provider();
        let project_fs = io_provider.fs();

        let command_executor =
            self.get_command_executor(&artifact_fs, project_fs, executor_config)?;
        let blocking_executor = self.get_blocking_executor();
        let materializer = self.per_transaction_data().get_materializer();
        let events = self.per_transaction_data().get_dispatcher().dupe();
        let run_action_knobs = self.per_transaction_data().get_run_action_knobs();

        Ok(Arc::new(BuckActionExecutor::new(
            CommandExecutor::new(
                command_executor,
                artifact_fs,
                executor_config.path_separator,
            ),
            blocking_executor,
            materializer,
            events,
            run_action_knobs,
        )))
    }
}

pub struct BuckActionExecutor {
    command_executor: CommandExecutor,
    blocking_executor: Arc<dyn BlockingExecutor>,
    materializer: Arc<dyn Materializer>,
    events: EventDispatcher,
    run_action_knobs: RunActionKnobs,
}

impl BuckActionExecutor {
    pub fn new(
        command_executor: CommandExecutor,
        blocking_executor: Arc<dyn BlockingExecutor>,
        materializer: Arc<dyn Materializer>,
        events: EventDispatcher,
        run_action_knobs: RunActionKnobs,
    ) -> Self {
        Self {
            command_executor,
            blocking_executor,
            materializer,
            events,
            run_action_knobs,
        }
    }
}

struct BuckActionExecutionContext<'a> {
    executor: &'a BuckActionExecutor,
    action: &'a RegisteredAction,
    inputs: IndexMap<ArtifactGroup, ArtifactGroupValues>,
    outputs: &'a IndexSet<BuildArtifact>,
    command_reports: &'a mut Vec<CommandExecutionReport>,
}

#[async_trait]
impl ActionExecutionCtx for BuckActionExecutionContext<'_> {
    fn target(&self) -> CommandExecutionTarget<'_> {
        CommandExecutionTarget {
            owner: self.action.owner(),
            category: self.action.category(),
            identifier: self.action.identifier(),
        }
    }

    fn fs(&self) -> &ArtifactFs {
        self.executor.command_executor.fs()
    }

    fn executor_fs(&self) -> ExecutorFs {
        self.executor.command_executor.executor_fs()
    }

    fn materializer(&self) -> &dyn Materializer {
        self.executor.materializer.as_ref()
    }

    fn events(&self) -> &EventDispatcher {
        &self.executor.events
    }

    fn artifact_values(&self, artifact: &ArtifactGroup) -> &ArtifactGroupValues {
        self.inputs.get(artifact).unwrap_or_else(|| panic!("Internal error: action {:?} tried to grab the artifact {:?} even though it was not an input.", self.action.owner(), artifact))
    }

    fn blocking_executor(&self) -> &dyn BlockingExecutor {
        &*self.executor.blocking_executor
    }

    fn run_action_knobs(&self) -> RunActionKnobs {
        self.executor.run_action_knobs
    }

    async fn exec_cmd(
        &mut self,
        request: &CommandExecutionRequest,
    ) -> anyhow::Result<(
        IndexMap<CommandExecutionOutput, ArtifactValue>,
        ActionExecutionMetadata,
    )> {
        let action = self.target();
        let manager = CommandExecutionManager::new(
            self.executor.command_executor.name(),
            <dyn ClaimManager>::new_simple(),
            self.executor.events.dupe(),
        );
        let CommandExecutionResult {
            outputs,
            report,
            rejected_execution,
        } = self
            .executor
            .command_executor
            .exec_cmd(action, request, manager)
            .await;

        // TODO (@torozco): The execution kind should be made to come via the command reports too.
        let res = match &report.status {
            CommandExecutionStatus::Success { execution_kind } => Ok((
                outputs,
                ActionExecutionMetadata {
                    execution_kind: ActionExecutionKind::Command(execution_kind.clone()),
                    timing: report.timing.into(),
                },
            )),

            _ => Err(CommandExecutionErrorMarker.into()),
        };

        self.command_reports.extend(rejected_execution.into_iter());
        self.command_reports.push(report);

        res
    }

    async fn cleanup_outputs(&mut self) -> anyhow::Result<()> {
        // Delete all outputs before we start, so things will be clean.
        let output_paths = self
            .outputs
            .iter()
            .map(|o| self.fs().resolve_build(o))
            .collect::<Vec<_>>();

        // Invalidate all the output paths this action might provide. Note that this is a bit
        // approximative: we might have previous instances of this action that declared
        // different outputs with a different materialization method that will become invalid
        // now. However, nothing should reference those stale outputs, so while this does not
        // do a good job of cleaning up garbage, it prevents using invalid artifacts.
        self.executor
            .materializer
            .invalidate_many(output_paths.clone())
            .await
            .context("Failed to invalidate output directory")?;

        // Use Eden's clean up API if possible, it is significantly faster on Eden compared with
        // the native method as the API does not load and materialize files or folders
        if let Some(eden_buck_out) = self.executor.materializer.eden_buck_out() {
            eden_buck_out
                .remove_paths_recursive(self.fs().fs(), output_paths)
                .await?;
        } else {
            self.executor
                .blocking_executor
                .execute_io(box CleanOutputPaths {
                    paths: output_paths,
                })
                .await
                .context("Failed to cleanup output directory")?;
        }

        Ok(())
    }
}

#[async_trait]
impl ActionExecutor for BuckActionExecutor {
    async fn execute(
        &self,
        inputs: IndexMap<ArtifactGroup, ArtifactGroupValues>,
        action: &RegisteredAction,
    ) -> (
        Result<(ActionOutputs, ActionExecutionMetadata), ExecuteError>,
        Vec<CommandExecutionReport>,
    ) {
        let mut command_reports = Vec::new();

        let res = async {
            let outputs = action.outputs()?;

            let mut ctx = BuckActionExecutionContext {
                executor: self,
                action,
                inputs,
                outputs: outputs.as_ref(),
                command_reports: &mut command_reports,
            };

            let (result, metadata) = match action.as_executable() {
                ActionExecutable::Pristine(exe) => {
                    ctx.cleanup_outputs().await?;
                    exe.execute(&mut ctx).await?
                }
                ActionExecutable::Incremental(exe) => {
                    // Let the action perform clean up in this case.
                    exe.execute(&mut ctx).await?
                }
            };

            // Check all the outputs were returned, and no additional outputs
            // TODO (T122966509): Check projections here as well
            if !outputs.iter().eq(result.0.outputs.keys()) {
                let wanted = outputs
                    .iter()
                    .filter(|x| !result.0.outputs.contains_key(*x))
                    .map(|x| self.command_executor.fs().resolve_build(x))
                    .collect();
                let got = result
                    .0
                    .outputs
                    .keys()
                    .filter(|x| !outputs.contains(*x))
                    .map(|x| self.command_executor.fs().resolve_build(x))
                    .collect::<Vec<_>>();
                if got.is_empty() {
                    Err(ExecuteError::MissingOutputs { wanted })
                } else {
                    Err(ExecuteError::MismatchedOutputs { wanted, got })
                }
            } else {
                Ok((result, metadata))
            }
        }
        .await;

        (res, command_reports)
    }
}

/// IoRequest we dispatch to the blocking executor to clear output paths.
pub struct CleanOutputPaths {
    paths: Vec<ProjectRelativePathBuf>,
}

impl CleanOutputPaths {
    pub(crate) fn clean<'a>(
        paths: impl Iterator<Item = &'a ProjectRelativePath>,
        fs: &'a ProjectRoot,
    ) -> anyhow::Result<()> {
        for path in paths {
            cleanup_path(fs, path)
                .with_context(|| format!("Error cleaning up output path `{}`", path))?;
        }
        Ok(())
    }
}

fn cleanup_path(fs: &ProjectRoot, mut path: &ProjectRelativePath) -> anyhow::Result<()> {
    fs.remove_path_recursive(path)?;

    // Be aware of T85589819 - the parent directory might already exist, but as a _file_.  It might
    // be even worse, it might be 2 parents up, which will cause create_dir to fail when we try to
    // execute. So, we walk up the tree until we either find a dir we're happy with, or a file we
    // can delete. It's safe to delete this file because we know it doesn't overlap with a current
    // output, or that check would have failed, so it must be a stale file.
    loop {
        path = match path.parent() {
            Some(path) => path,
            None => {
                return Err(anyhow::anyhow!(
                    "Internal Error: reached root before finding a directory that exists!"
                ));
            }
        };

        match fs.resolve(path).symlink_metadata() {
            Ok(m) => {
                if m.is_dir() {
                    // It's a dir, no need to go further, and no need to delete.
                } else {
                    // There was a file , so it's safe to delete and then we can exit because we'll
                    // be able to create a dir here.
                    fs.remove_path_recursive(path)?;
                }
                return Ok(());
            }
            Err(e) => {
                #[cfg(unix)]
                {
                    use std::io;

                    // If we get ENOENT that guarantees there is no file on the path. If there was
                    // one, we would get ENOTDIR. TODO (T123279320) This probably works on Windows,
                    // but it wasn't tested there.
                    let is_enoent = e.kind() == io::ErrorKind::NotFound;

                    if is_enoent {
                        return Ok(());
                    }
                }

                #[cfg(not(unix))]
                {
                    // On non-Unix we don't have the optimization above. Recursing all the way up
                    // until we find the first dir (or file to delete) is fine. There will
                    // eventually be *a* directory (at buck-out, then another one at the empty
                    // directory, which is our cwd, and should exist by now).
                    let _e = e;
                }

                // Continue going up. Eventually we should reach the output directory, which should
                // exist.
            }
        }
    }
}

impl IoRequest for CleanOutputPaths {
    fn execute(self: Box<Self>, project_fs: &ProjectRoot) -> anyhow::Result<()> {
        Self::clean(self.paths.iter().map(AsRef::as_ref), project_fs)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::fs;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use buck2_core::buck_path::BuckPath;
    use buck2_core::category::Category;
    use buck2_core::cells::cell_root_path::CellRootPathBuf;
    use buck2_core::cells::paths::CellRelativePath;
    use buck2_core::cells::testing::CellResolverExt;
    use buck2_core::cells::CellName;
    use buck2_core::cells::CellResolver;
    use buck2_core::configuration::Configuration;
    use buck2_core::fs::paths::ForwardRelativePathBuf;
    use buck2_core::fs::project::ProjectRelativePath;
    use buck2_core::fs::project::ProjectRelativePathBuf;
    use buck2_core::fs::project::ProjectRootTemp;
    use buck2_core::package::package_relative_path::PackageRelativePathBuf;
    use buck2_core::package::Package;
    use buck2_core::target::testing::ConfiguredTargetLabelExt;
    use buck2_core::target::ConfiguredTargetLabel;
    use buck2_core::target::TargetName;
    use buck2_node::execute::config::CommandExecutorConfig;
    use buck2_node::execute::config::PathSeparatorKind;
    use events::dispatch::EventDispatcher;
    use gazebo::prelude::*;
    use indexmap::indexset;
    use indexmap::IndexSet;
    use once_cell::sync::Lazy;

    use crate::actions::artifact::testing::BuildArtifactTestingExt;
    use crate::actions::artifact::Artifact;
    use crate::actions::artifact::ArtifactFs;
    use crate::actions::artifact::ArtifactValue;
    use crate::actions::artifact::BuildArtifact;
    use crate::actions::artifact::SourceArtifact;
    use crate::actions::Action;
    use crate::actions::ActionExecutable;
    use crate::actions::ActionExecutionCtx;
    use crate::actions::PristineActionExecutable;
    use crate::actions::RegisteredAction;
    use crate::artifact_groups::ArtifactGroup;
    use crate::artifact_groups::ArtifactGroupValues;
    use crate::deferred::testing::DeferredDataExt;
    use crate::deferred::testing::DeferredIdExt;
    use crate::deferred::BaseDeferredKey;
    use crate::deferred::DeferredData;
    use crate::deferred::DeferredId;
    use crate::deferred::DeferredKey;
    use crate::execute::blocking::testing::DummyBlockingExecutor;
    use crate::execute::cleanup_path;
    use crate::execute::commands::dry_run::DryRunExecutor;
    use crate::execute::commands::CommandExecutionInput;
    use crate::execute::commands::CommandExecutionRequest;
    use crate::execute::commands::CommandExecutor;
    use crate::execute::materializer::nodisk::NoDiskMaterializer;
    use crate::execute::ActionExecutionKind;
    use crate::execute::ActionExecutionMetadata;
    use crate::execute::ActionExecutionTimingData;
    use crate::execute::ActionExecutor;
    use crate::execute::ActionOutputs;
    use crate::execute::BuckActionExecutor;
    use crate::path::BuckOutPathResolver;
    use crate::path::BuckPathResolver;

    #[tokio::test]
    async fn can_execute_some_action() {
        let cells = CellResolver::of_names_and_paths(&[(
            CellName::unchecked_new("cell".into()),
            CellRootPathBuf::new(ProjectRelativePathBuf::unchecked_new("cell_path".into())),
        )]);

        let temp_fs = ProjectRootTemp::new().unwrap();

        let project_fs = temp_fs.path().clone();
        let artifact_fs = ArtifactFs::new(
            BuckPathResolver::new(cells),
            BuckOutPathResolver::new(ProjectRelativePathBuf::unchecked_new(
                "cell/buck-out/v2".into(),
            )),
            project_fs.clone(),
        );

        let tracker = Arc::new(Mutex::new(Vec::new()));

        let executor = BuckActionExecutor::new(
            CommandExecutor::new(
                Arc::new(DryRunExecutor::new(tracker, None)),
                artifact_fs,
                PathSeparatorKind::Unix,
            ),
            Arc::new(DummyBlockingExecutor { fs: project_fs }),
            Arc::new(NoDiskMaterializer),
            EventDispatcher::null(),
            Default::default(),
        );

        #[derive(Debug)]
        struct TestingAction {
            inputs: IndexSet<ArtifactGroup>,
            outputs: IndexSet<BuildArtifact>,
            ran: AtomicBool,
        }

        #[async_trait]
        impl Action for TestingAction {
            fn kind(&self) -> buck2_data::ActionKind {
                buck2_data::ActionKind::NotSet
            }

            fn inputs(&self) -> anyhow::Result<Cow<'_, IndexSet<ArtifactGroup>>> {
                Ok(Cow::Borrowed(&self.inputs))
            }

            fn outputs(&self) -> anyhow::Result<Cow<'_, IndexSet<BuildArtifact>>> {
                Ok(Cow::Borrowed(&self.outputs))
            }

            fn as_executable(&self) -> ActionExecutable<'_> {
                ActionExecutable::Pristine(self)
            }

            fn category(&self) -> &Category {
                static TEST_CATEGORY: Lazy<Category> =
                    Lazy::new(|| Category::try_from("testing").unwrap());

                &TEST_CATEGORY
            }
        }

        #[async_trait]
        impl PristineActionExecutable for TestingAction {
            async fn execute(
                &self,
                ctx: &mut dyn ActionExecutionCtx,
            ) -> anyhow::Result<(ActionOutputs, ActionExecutionMetadata)> {
                self.ran.store(true, Ordering::SeqCst);

                let req = CommandExecutionRequest::new(
                    vec!["foo".to_owned(), "bar".to_owned(), "cmd".to_owned()],
                    self.inputs
                        .iter()
                        .map(|x| {
                            CommandExecutionInput::Artifact(ArtifactGroupValues::from_artifact(
                                x.unpack_artifact().unwrap().dupe(),
                                ArtifactValue::empty_file(),
                            ))
                        })
                        .collect(),
                    self.outputs.clone(),
                    HashMap::new(),
                );

                // on fake executor, this does nothing
                let res = ctx.exec_cmd(&req).await;

                // Must write out the things we promised to do
                for x in &self.outputs {
                    ctx.fs().write_file(x, "", false)?
                }

                res?;
                let outputs = self
                    .outputs
                    .iter()
                    .map(|o| (o.dupe(), ArtifactValue::empty_file()))
                    .collect();
                Ok((
                    ActionOutputs::new(outputs),
                    ActionExecutionMetadata {
                        execution_kind: ActionExecutionKind::Simple,
                        timing: ActionExecutionTimingData::default(),
                    },
                ))
            }
        }

        let pkg = Package::new(
            &CellName::unchecked_new("cell".into()),
            CellRelativePath::unchecked_new("pkg"),
        );

        let inputs = indexset![ArtifactGroup::Artifact(Artifact::from(
            SourceArtifact::new(BuckPath::new(
                pkg.dupe(),
                PackageRelativePathBuf::unchecked_new("source".into()),
            ))
        ))];
        let label = ConfiguredTargetLabel::testing_new(
            pkg,
            TargetName::unchecked_new("foo"),
            Configuration::testing_new(),
        );
        let outputs = indexset![BuildArtifact::testing_new(
            label.dupe(),
            ForwardRelativePathBuf::unchecked_new("output".into()),
            DeferredId::testing_new(0),
        )];

        let action = RegisteredAction::new(
            DeferredData::testing_new(DeferredKey::Base(
                BaseDeferredKey::TargetLabel(label.dupe()),
                DeferredId::testing_new(0),
            )),
            box TestingAction {
                inputs,
                outputs: outputs.clone(),
                ran: Default::default(),
            },
            CommandExecutorConfig::testing_local(),
        );
        let res = executor
            .execute(Default::default(), &action)
            .await
            .0
            .unwrap();
        let outputs = outputs
            .iter()
            .map(|o| (o.dupe(), ArtifactValue::empty_file()))
            .collect();
        assert_eq!(res.0, ActionOutputs::new(outputs));
    }

    #[test]
    fn test_cleanup_path_missing() -> anyhow::Result<()> {
        let fs = ProjectRootTemp::new()?;
        let fs = fs.path();
        fs::create_dir_all(fs.resolve(ProjectRelativePath::unchecked_new("foo/bar/qux")))?;
        cleanup_path(fs, ProjectRelativePath::unchecked_new("foo/bar/qux/xx"))?;
        assert!(
            fs.resolve(ProjectRelativePath::unchecked_new("foo/bar/qux"))
                .exists()
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_path_present() -> anyhow::Result<()> {
        let fs = ProjectRootTemp::new()?;
        let fs = fs.path();
        fs::create_dir_all(fs.resolve(ProjectRelativePath::unchecked_new("foo/bar/qux")))?;
        cleanup_path(fs, ProjectRelativePath::unchecked_new("foo/bar/qux"))?;
        assert!(
            !fs.resolve(ProjectRelativePath::unchecked_new("foo/bar/qux"))
                .exists()
        );
        assert!(
            fs.resolve(ProjectRelativePath::unchecked_new("foo/bar"))
                .exists()
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_path_overlap() -> anyhow::Result<()> {
        let fs = ProjectRootTemp::new()?;
        let fs = fs.path();
        fs.write_file(ProjectRelativePath::unchecked_new("foo/bar"), "xx", false)?;
        cleanup_path(fs, ProjectRelativePath::unchecked_new("foo/bar/qux"))?;
        assert!(
            !fs.resolve(ProjectRelativePath::unchecked_new("foo/bar"))
                .exists()
        );
        assert!(
            fs.resolve(ProjectRelativePath::unchecked_new("foo"))
                .exists()
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_path_overlap_deep() -> anyhow::Result<()> {
        let fs = ProjectRootTemp::new()?;
        let fs = fs.path();
        fs.write_file(ProjectRelativePath::unchecked_new("foo/bar"), "xx", false)?;
        cleanup_path(
            fs,
            ProjectRelativePath::unchecked_new("foo/bar/qux/1/2/3/4"),
        )?;
        assert!(
            !fs.resolve(ProjectRelativePath::unchecked_new("foo/bar"))
                .exists()
        );
        assert!(
            fs.resolve(ProjectRelativePath::unchecked_new("foo"))
                .exists()
        );
        Ok(())
    }
}
