/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::io::Write;
use std::sync::Arc;

use async_trait::async_trait;
use buck2_build_api::interpreter::context::fbcode_prelude;
use buck2_core::cells::*;
use buck2_interpreter::common::StarlarkModulePath;
use buck2_interpreter::dice::HasCalculationDelegate;
use buck2_interpreter::dice::HasGlobalInterpreterState;
use buck2_interpreter::interpreter::InterpreterConfigForCell;
use cli_proto::ClientContext;

use crate::commands::audit::AuditSubcommand;
use crate::commands::common::CommonConfigOptions;
use crate::commands::common::CommonConsoleOptions;
use crate::commands::common::CommonEventLogOptions;
use crate::daemon::server::ServerCommandContext;

#[derive(Debug, clap::Parser, serde::Serialize, serde::Deserialize)]
#[clap(
    name = "audit-prelude",
    about = "print the interpreter prelude to stdout"
)]
pub(crate) struct AuditPreludeCommand {}

#[async_trait]
impl AuditSubcommand for AuditPreludeCommand {
    async fn server_execute(
        &self,
        mut server_ctx: ServerCommandContext,
        _client_ctx: ClientContext,
    ) -> anyhow::Result<()> {
        let ctx = server_ctx.dice_ctx().await?;
        let mut stdout = server_ctx.stdout()?;
        // Print out all the Prelude-like stuff that is loaded into each module
        let global_interpreter_state = ctx.get_global_interpreter_state().await?;
        let interpreter_calculation = ctx
            .get_interpreter_calculator(fbcode_prelude().cell(), fbcode_prelude().build_file_cell())
            .await?;
        // Slightly odd that to get the build_file_global_env out of global_interpreter_state
        // we first have to wrap it in an InterpreterConfig with a fake CellAliasResolver
        let aliases = hashmap![
            CellAlias::new("".to_owned()) =>
            CellName::unchecked_new("".to_owned()),
        ];
        let interpreter_config = InterpreterConfigForCell::new(
            CellAliasResolver::new(Arc::new(aliases))?,
            global_interpreter_state,
        )?;
        writeln!(
            stdout,
            "{}",
            interpreter_config.build_file_global_env().describe()
        )?;
        writeln!(
            stdout,
            "{}",
            interpreter_calculation
                .eval_module(StarlarkModulePath::LoadFile(&fbcode_prelude()))
                .await?
                .env()
                .describe()
        )?;

        Ok(())
    }

    fn config_opts(&self) -> Option<&CommonConfigOptions> {
        None
    }

    fn console_opts(&self) -> Option<&CommonConsoleOptions> {
        None
    }

    fn event_log_opts(&self) -> Option<&CommonEventLogOptions> {
        None
    }
}
