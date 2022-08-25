/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::PathBuf;

use crate::client_ctx::ClientCommandContext;
use crate::exit_result::ExitResult;
use crate::subscribers::event_log::retrieve_nth_recent_log;

/// Show the spans that were open when the log ended
#[derive(Debug, clap::Parser)]
#[clap(group = clap::ArgGroup::with_name("event_log"))]
pub struct WhatUpCommand {
    /// A path to an event-log file to read from. Only works for log files with a single command in them.
    #[clap(long, group = "event_log", value_name = "PATH")]
    path: Option<PathBuf>,

    /// Which recent command to read the event log from.
    #[clap(
        long,
        help = "Replay the Nth most recent command (`--recent 0` is the most recent).",
        group = "event_log",
        value_name = "NUMBER"
    )]
    pub recent: Option<usize>,
}

impl WhatUpCommand {
    pub fn exec(self, _matches: &clap::ArgMatches, ctx: ClientCommandContext) -> ExitResult {
        let Self { path, recent } = self;

        let _log = match path {
            Some(path) => path,
            None => retrieve_nth_recent_log(&ctx, recent.unwrap_or(0))?,
        };

        crate::eprintln!("Command not implemented")?;
        ExitResult::failure()
    }
}
