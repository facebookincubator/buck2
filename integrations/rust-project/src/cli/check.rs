/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;
use std::str::FromStr;

use tracing::info;
use tracing::instrument;

use crate::buck;
use crate::buck::select_mode;
use crate::diagnostics;

pub(crate) struct Check {
    pub(crate) buck: buck::Buck,
    pub(crate) use_clippy: bool,
    pub(crate) target: String,
}

impl Check {
    pub(crate) fn new(mode: Option<String>, use_clippy: bool, target: String) -> Self {
        let mode = select_mode(mode.as_deref());
        let buck = buck::Buck::new(mode);
        Self {
            buck,
            use_clippy,
            target,
        }
    }

    #[instrument(name = "check", skip_all, fields(target = %self.target))]
    pub(crate) fn run(&self) -> Result<(), anyhow::Error> {
        let start = std::time::Instant::now();
        let buck = &self.buck;

        let cell_root = buck.resolve_target_cell_root(&self.target)?;
        let diagnostic_files = buck.check_target(self.use_clippy, &self.target)?;

        let mut diagnostics = vec![];
        for path in diagnostic_files {
            let contents = std::fs::read_to_string(path)?;
            for l in contents.lines() {
                // rustc (and with greater relevance, the underlying build.bxl script) emits diagnostics as newline-delimited JSON.
                // One complicating factor is that the file paths in the diagnostics are relative to each Buck project, which assumes that
                // a user does their work from a project root, such as `fbsource`. this means that the diagnostics for `lib.rs` in
                // `fbcode//common/rust/tracing-scuba:tracing-scuba` will be shown as `fbcode/common/rust/tracing-scuba/src/lib.rs`.
                //
                // this is not ideal. if the user decides to open their editor from the cell root (`fbcode`) or the target's
                // directory (`fbsource/fbcode/common/rust/tracing-scuba`), rust-analyzer will attempt to normalize the file paths
                // in the machine-readable diagnostic message relative to the current working directory. rust-analyzer will then not
                // be able find the resulting path inside its VFS, leading to no diagnostics being shown to the user. To fix this,
                // we rewrite the file paths in the diagnostics to be relative to the buck2 project root, resulting in a fully absolute
                // path.
                if let Ok(mut message) = serde_json::from_str::<diagnostics::Message>(l) {
                    make_message_absolute(&mut message, &cell_root);

                    let span = serde_json::to_value(message)?;
                    // this is done under the assumption that the number of diagnostics inside the vector
                    // is small (e.g., 32 or 64), so a linear seach of a vector will faster than hashing each element.
                    if !diagnostics.contains(&span) {
                        diagnostics.push(span);
                    }
                } else {
                    let value = serde_json::Value::from_str(l)?;
                    diagnostics.push(value)
                }
            }
        }

        for diagnostic in diagnostics {
            let out = serde_json::to_string(&diagnostic)?;
            println!("{}", out);
        }

        let duration = start.elapsed();
        info!(duration_ms = duration.as_millis(), "finished check");

        Ok(())
    }
}

fn make_message_absolute(message: &mut diagnostics::Message, base_dir: &Path) {
    for span in message.spans.iter_mut() {
        make_span_absolute(span, base_dir);
    }

    for message in message.children.iter_mut() {
        make_message_absolute(message, base_dir);
    }
}

fn make_span_absolute(span: &mut diagnostics::Span, base_dir: &Path) {
    span.file_name = base_dir.join(&span.file_name);

    if let Some(expansion) = &mut span.expansion {
        if let Some(def_site_span) = &mut expansion.def_site_span {
            make_span_absolute(def_site_span, base_dir);
        }

        make_span_absolute(&mut expansion.span, base_dir);
    }
}
