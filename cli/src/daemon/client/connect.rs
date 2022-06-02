/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    process::Command,
    time::Duration,
};

use anyhow::{anyhow, Context};
use buck2_core::fs::paths::AbsPathBuf;
use cli_proto::{daemon_api_client::DaemonApiClient, DaemonProcessInfo};
use events::subscriber::EventSubscriber;
use fs2::FileExt;
use gazebo::prelude::StrExt;
use thiserror::Error;
use tonic::transport::Channel;
use wait_timeout::ChildExt;

use crate::{
    commands::common::subscribers::stdout_stderr_forwarder::StdoutStderrForwarder,
    daemon::{
        client::{events_ctx::EventsCtx, BuckdClient, ClientKind, Replayer, VersionCheckResult},
        client_utils::{get_channel, retrying, ConnectionType, ParseError, SOCKET_ADDR},
    },
    paths::Paths,
};
/// Responsible for starting the daemon when no daemon is running.
/// This struct holds a lock such that only one daemon is ever started per daemon directory.
struct BuckdLifecycle<'a> {
    paths: &'a Paths,
    lock_file: File,
}

impl<'a> BuckdLifecycle<'a> {
    async fn lock_with_timeout(
        paths: &'a Paths,
        timeout: Duration,
    ) -> anyhow::Result<BuckdLifecycle<'a>> {
        let lifecycle_path = paths.daemon_dir()?.join("buckd.lifecycle");
        let file = File::create(lifecycle_path)?;
        retrying(
            Duration::from_millis(5),
            Duration::from_millis(100),
            timeout,
            async || Ok(file.try_lock_exclusive()?),
        )
        .await?;

        Ok(BuckdLifecycle::<'a> {
            paths,
            lock_file: file,
        })
    }

    async fn start_server(&self) -> anyhow::Result<()> {
        let project_dir = self.paths.project_root();

        let mut cmd =
            Command::new(std::env::current_exe().expect("somehow couldn't get current exe"));
        // --isolation-dir is an option on the root `buck` cli, not the subcommand.
        cmd.arg("--isolation-dir");
        cmd.arg(self.paths.isolation.as_str());

        cmd.arg("daemon");

        // It is the responsibility of processes that invoke buck2 to indicate via this environment variable whether or
        // not the child process should log to Scribe. The top-level buck2 CLI is invoked via the wrapper, which does
        // this; the `buck2 daemon` command must also be instructed to log to Scribe if the top-level CLI was itself
        // instructed to log.
        //
        // This environment variable ensures that Scribe logging is enabled upon entry of the buck2 daemon command.
        if !events::sink::scribe::is_enabled() {
            cmd.env("BUCK2_ENABLE_SCRIBE", "0");
        }

        cmd.current_dir(project_dir);

        if cfg!(windows) {
            // On Rustc 1.61, reading piped stdout hangs in this case.
            // Possibly related to https://github.com/rust-lang/rust/issues/45572
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
        } else {
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());
        }

        #[cfg(unix)]
        {
            use std::{ffi::OsString, os::unix::process::CommandExt};

            let mut title = OsString::new();
            title.push("buck2d");
            if let Some(dir) = project_dir.file_name() {
                title.push("[");
                title.push(dir);
                title.push("]");
            }
            cmd.arg0(title);
        }

        let mut child = cmd.spawn()?;

        let status_code = match child.wait_timeout(Duration::from_secs(
            env::var("BUCKD_STARTUP_TIMEOUT").map_or(10, |t| {
                t.parse::<u64>()
                    .unwrap_or_else(|_| panic!("Cannot convert {} to int", t))
            }),
        ))? {
            Some(status) => status.code(),
            None => {
                // child hasn't exited yet. this shouldn't ever happen, if it does, maybe the timeout needs to be changed.
                child.kill()?;
                child.wait()?.code()
            }
        };

        match status_code {
            Some(code) if code != 0 => {
                let mut stdout = Vec::new();
                child
                    .stdout
                    .expect("no stdout from `buck daemon` call")
                    .read_to_end(&mut stdout)
                    .expect("stdout not a string");
                let mut stderr = Vec::new();
                child
                    .stderr
                    .expect("no stderr from `buck daemon` call")
                    .read_to_end(&mut stderr)
                    .expect("stderr not a string");
                Err(BuckdConnectError::BuckDaemonStartupFailed {
                    code,
                    stdout: String::from_utf8_lossy(&stdout).to_string(),
                    stderr: String::from_utf8_lossy(&stderr).to_string(),
                }
                .into())
            }
            _ => Ok(()),
        }
    }
}

impl<'a> Drop for BuckdLifecycle<'a> {
    fn drop(&mut self) {
        self.lock_file
            .unlock()
            .expect("Unexpected failure to unlock buckd.pid file.")
    }
}

/// Type-safe indicator that the buckd client must be given the expected subscribers
/// prior to being used outside of startup.
struct BootstrapBuckdClient(BuckdClient);

impl BootstrapBuckdClient {
    pub(crate) fn new(
        client: DaemonApiClient<Channel>,
        info: DaemonProcessInfo,
        daemon_dir: AbsPathBuf,
    ) -> Self {
        // Start with basic output forwarding to catch any output (usually errors or panics) at startup.
        // This subscriber gets replaced with the actual subscribers once the startup stage of the daemon lifecycle is complete.
        let events_ctx = EventsCtx::new(daemon_dir, vec![box StdoutStderrForwarder]);
        Self(BuckdClient {
            info,
            client: ClientKind::Daemon(client),
            events_ctx,
        })
    }

    pub(crate) fn with_subscribers(
        self,
        subscribers: Vec<Box<dyn EventSubscriber>>,
    ) -> BuckdClient {
        let mut client = self.0;
        client.events_ctx.subscribers = subscribers;
        client
    }
}

/// The settings prior to connecting to the Buck daemon.
/// By default, attempts to connect to a daemon with the same version as the client.
/// If the daemon has a different version, it will kill it and restart it with the correct version.
/// This behavior can be overridden by calling the `existing_only` method.
/// If the `existing_only` method is called, then any existing buck daemon (regardless of version) is accepted.
///
/// The default set of subscribers is *not* empty, but rather forwards stdout and stderr, which captures panics, for example.
pub struct BuckdConnectOptions {
    pub(crate) existing_only: bool,
    /// Subscribers manage the way that incoming events from the server are handled.
    /// The client will forward events and stderr/stdout output from the server to each subscriber.
    /// By default, this list is set to a single subscriber that notifies the user of basic output from the server.
    pub(crate) subscribers: Vec<Box<dyn EventSubscriber>>,
}

impl Default for BuckdConnectOptions {
    fn default() -> Self {
        Self {
            existing_only: Default::default(),
            subscribers: vec![box StdoutStderrForwarder],
        }
    }
}

impl BuckdConnectOptions {
    pub fn existing_only() -> Self {
        Self {
            existing_only: true,
            ..Default::default()
        }
    }

    pub async fn connect(self, paths: &Paths) -> anyhow::Result<BuckdClient> {
        let daemon_dir = paths.daemon_dir()?;
        std::fs::create_dir_all(&daemon_dir)
            .with_context(|| format!("when creating daemon dir {}", daemon_dir.display()))?;
        let client = self
            .establish_connection(paths)
            .await
            .context("when establishing connection to buckd")?;

        // after startup is complete, replace the basic readers with our own.
        Ok(client.with_subscribers(self.subscribers))
    }

    pub fn replay(self, replayer: Replayer, paths: &Paths) -> anyhow::Result<BuckdClient> {
        let fake_info = DaemonProcessInfo {
            pid: 0,
            endpoint: "".to_owned(),
            version: "".to_owned(),
        };
        let events_ctx = EventsCtx::new(paths.daemon_dir()?, self.subscribers);

        Ok(BuckdClient {
            client: ClientKind::Replayer(Box::pin(replayer)),
            events_ctx,
            info: fake_info,
        })
    }

    async fn establish_connection(&self, paths: &Paths) -> anyhow::Result<BootstrapBuckdClient> {
        match self.try_connect_existing(paths).await {
            Ok(mut client) => {
                if self.existing_only || client.0.check_version().await?.is_match() {
                    // either the version matches or we don't care about the version, return the client.
                    return Ok(client);
                }
                // fallthrough to the more complicated startup case.
            }
            Err(e) if self.existing_only => {
                return Err(e.context("No existing connection and not asked to start one"));
            }
            Err(_) => {
                // fallthrough to the startup case
            }
        }

        // At this point, we've either failed to connect to buckd or buckd had the wrong version. At this point,
        // we'll get the lifecycle lock to ensure we don't have races with other processes as we check and change things.

        let lifecycle_lock = BuckdLifecycle::lock_with_timeout(paths, Duration::from_secs(10))
            .await
            .with_context(|| "when locking buckd lifecycle.lock")?;

        // Even if we didn't connect before, it's possible that we just raced with another invocation
        // starting the server, so we try to connect again while holding the lock.
        if let Ok(mut client) = self.try_connect_existing(paths).await {
            if self.existing_only || client.0.check_version().await?.is_match() {
                // either the version matches or we don't care about the version, return the client.
                return Ok(client);
            }
            client
                .0
                .kill("client expected different buck version")
                .await?;
        }
        // Now there's definitely no server that can be connected to
        // TODO(cjhopman): a non-responsive buckd process may be somehow lingering around and we should probably kill it off here.
        lifecycle_lock.start_server().await?;
        // It might take a little bit for the daemon server to start up. We could wait for the buckd.info
        // file to appear, but it's just as easy to just retry the connection itself.
        let mut client = retrying(
            Duration::from_millis(5),
            Duration::from_millis(100),
            Duration::from_secs(env::var("BUCKD_STARTUP_TIMEOUT").map_or(10, |t| {
                t.parse::<u64>()
                    .unwrap_or_else(|_| panic!("Cannot convert {} to int", t))
            })),
            async || {
                self.try_connect_existing(paths)
                    .await
                    .with_context(|| "Failed to start server")
            },
        )
        .await?;

        if self.existing_only {
            return Ok(client);
        }

        match client.0.check_version().await? {
            VersionCheckResult::Match => Ok(client),
            VersionCheckResult::Mismatch { expected, actual } => {
                Err(BuckdConnectError::BuckDaemonVersionWrongAfterStart { expected, actual }.into())
            }
        }
    }

    async fn try_connect_existing(&self, paths: &Paths) -> anyhow::Result<BootstrapBuckdClient> {
        let daemon_dir = paths.daemon_dir()?;
        let location = daemon_dir.join("buckd.info");
        let file = File::open(&location)
            .with_context(|| format!("Trying to open buckd info, `{}`", location.display()))?;
        let reader = BufReader::new(file);
        let info: DaemonProcessInfo = serde_json::from_reader(reader).with_context(|| {
            format!(
                "Parsing daemon info in `{}`. Try deleting that file and running `buck2 kill` before running your command again",
                location.display(),
            )
        })?;
        let (protocol, endpoint) = info.endpoint.split1(":");
        let connection_type = match protocol {
            "uds" => ConnectionType::UDS {
                unix_socket: endpoint.to_owned(),
            },
            "tcp" => ConnectionType::TCP {
                socket: SOCKET_ADDR.to_owned(),
                port: endpoint.to_owned(),
            },
            _ => {
                return Err(anyhow!(ParseError::ParseError(endpoint.to_owned())));
            }
        };

        let client = DaemonApiClient::new(get_channel(connection_type).await?);

        Ok(BootstrapBuckdClient::new(client, info, daemon_dir))
    }
}

#[derive(Debug, Error)]
enum BuckdConnectError {
    #[error(
        "buck daemon startup failed with exit code {code}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    BuckDaemonStartupFailed {
        code: i32,
        stdout: String,
        stderr: String,
    },
    #[error("during buck daemon startup, the started process had the wrong version.")]
    BuckDaemonVersionWrongAfterStart { expected: String, actual: String },
}
