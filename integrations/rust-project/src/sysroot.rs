/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::anyhow;
use anyhow::Context;
use tracing::instrument;

use crate::buck::relative_to;
use crate::buck::truncate_line_ending;
use crate::buck::utf8_output;
use crate::buck::Buck;
use crate::json_project::Sysroot;

#[derive(Debug)]
pub(crate) enum SysrootConfig {
    Sysroot {
        sysroot: PathBuf,
        sysroot_src: Option<PathBuf>,
    },
    BuckConfig,
    Rustup {
        sysroot_src: Option<PathBuf>,
    },
}

/// Choose sysroot and sysroot_src based on platform.
///
/// `sysroot` is the directory that contains std crates:
/// <https://doc.rust-lang.org/rustc/command-line-arguments.html#--sysroot-override-the-system-root>
/// and also contains libexec helpers such as rust-analyzer-proc-macro-srv.
///
/// `sysroot_src` is the directory that contains the source to std crates:
/// <https://rust-analyzer.github.io/manual.html#non-cargo-based-projects>
#[instrument(ret, fields(project_root = %project_root.display(), relative_paths = ?relative_paths))]
pub(crate) fn resolve_buckconfig_sysroot(
    project_root: &Path,
    relative_paths: bool,
) -> Result<Sysroot, anyhow::Error> {
    let buck = Buck::default();

    if cfg!(target_os = "linux") {
        let base: PathBuf = if relative_paths {
            PathBuf::from("")
        } else {
            project_root.into()
        };

        let sysroot_src = buck.resolve_sysroot_src()?;
        let sysroot_src = if relative_paths {
            sysroot_src
        } else {
            project_root.join(sysroot_src)
        };

        // TODO(diliopoulos): remove hardcoded path to toolchain sysroot and replace with:
        // buck2 run fbcode//third-party-buck/platform010/build/rust:bin/rustc -- --print sysroot
        let sysroot = Sysroot {
            sysroot: base.join("fbcode/third-party-buck/platform010/build/rust/llvm-fb-15"),
            sysroot_src,
        };

        return Ok(sysroot);
    }
    // Spawn both `rustc` and `buck audit config` in parallel without blocking.
    let fbsource_rustc = project_root.join("xplat/rust/toolchain/current/basic/bin/rustc");
    let mut sysroot_cmd = Command::new(fbsource_rustc);
    sysroot_cmd
        .arg("--print=sysroot")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let sysroot_child = sysroot_cmd.spawn()?;

    let sysroot_src = buck.resolve_sysroot_src()?;
    let sysroot_src = if relative_paths {
        sysroot_src
    } else {
        project_root.join(sysroot_src)
    };

    // Now block while we wait for both processes.
    let mut sysroot = utf8_output(sysroot_child.wait_with_output(), &sysroot_cmd)
        .context("error asking rustc for sysroot")?;
    truncate_line_ending(&mut sysroot);

    let mut sysroot: PathBuf = sysroot.into();
    if relative_paths {
        sysroot = relative_to(&sysroot, project_root);
    }

    let sysroot = Sysroot {
        sysroot,
        sysroot_src,
    };

    Ok(sysroot)
}

#[instrument(ret)]
pub(crate) fn resolve_rustup_sysroot(
    sysroot_src_override: Option<PathBuf>,
) -> Result<Sysroot, anyhow::Error> {
    let mut cmd = Command::new("rustc");
    cmd.arg("--print=sysroot")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut output = utf8_output(cmd.output(), &cmd)?;
    truncate_line_ending(&mut output);
    let sysroot = PathBuf::from(output);

    let sysroot = if let Some(sysroot_src) = sysroot_src_override {
        validate(&sysroot_src)
            .context("Invalid --sysroot-src, did not contain the standard library source code")?;
        Sysroot {
            sysroot,
            sysroot_src,
        }
    } else {
        let sysroot_src = Sysroot::sysroot_src_for_sysroot(&sysroot);
        validate(&sysroot_src)
            .context("Rustup toolchain did not have rust-src component installed")?;
        Sysroot {
            sysroot_src,
            sysroot,
        }
    };

    Ok(sysroot)
}

pub(crate) fn resolve_provided_sysroot(
    sysroot: &Path,
    sysroot_src_override: Option<&Path>,
    project_root: &Path,
    relative_paths: bool,
) -> Result<Sysroot, anyhow::Error> {
    let mut sysroot = expand_tilde(sysroot)?.canonicalize().context(format!(
        "--sysroot path could not be canonicalized: {}",
        sysroot.display()
    ))?;
    let mut sysroot_src = if let Some(path) = sysroot_src_override {
        let path = expand_tilde(path)?.canonicalize().context(format!(
            "--sysroot-src path could not be canonicalized: {}",
            path.display()
        ))?;
        validate(&path)
            .context("Invalid --sysroot-src, did not contain the standard library source code")?;
        path
    } else {
        let path = Sysroot::sysroot_src_for_sysroot(&sysroot);
        validate(&path)
            .context("Provided --sysroot did not contain the standard library source code")?;
        path
    };
    if relative_paths {
        sysroot = relative_to(&sysroot, project_root);
        sysroot_src = relative_to(&sysroot_src, project_root);
    }
    Ok(Sysroot {
        sysroot,
        sysroot_src,
    })
}

fn validate(sysroot_src: &Path) -> Result<(), anyhow::Error> {
    let core = sysroot_src.join("core");
    if !sysroot_src.exists() {
        return Err(anyhow!("No such directory {}", sysroot_src.display()));
    }
    if !core.exists() {
        return Err(anyhow!(
            "No `core` directory in {}. Are you sure this is a sysroot src directory?",
            sysroot_src.display()
        ));
    }
    Ok(())
}

fn expand_tilde(path: &Path) -> Result<PathBuf, anyhow::Error> {
    if path.starts_with("~") {
        let path = path.strip_prefix("~")?;
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home = PathBuf::from(home);
        Ok(home.join(path))
    } else {
        Ok(path.to_path_buf())
    }
}
