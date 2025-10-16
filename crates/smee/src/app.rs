// Copyright 2025 Laurent Pireyn
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::fs;
use std::fs::File;
use std::fs::Permissions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Stdio;
use std::str::FromStr;

use clap::Parser;
use eyre::OptionExt;
use eyre::Report;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use is_executable::IsExecutable;
use log::LevelFilter;

use crate::cli::ActivateArgs;
use crate::cli::ColorPolicy;
use crate::cli::DeactivateArgs;
use crate::cli::HookArgs;
use crate::cli::HookScopeArgs;
use crate::cli::SmeeCommand;
use crate::cli::SmeeSubcommand;
use crate::git::Scope;
use crate::git::ValueType;
use crate::git::git_config_add;
use crate::git::git_config_get;
use crate::git::git_config_get_all;
use crate::git::git_config_remove;
use crate::git::git_hooks_path;
use crate::git::git_work_tree;
use crate::logger;

/// Smee application.
#[derive(Debug)]
pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub fn run(self) {
        // Let Clap report errors and exit if necessary
        let command = SmeeCommand::parse();
        // Determine max log level
        let max_level = if command.quiet {
            // The `--quiet` option was used
            LevelFilter::Warn
        } else if command.verbose {
            // The `--verbose` option was used
            LevelFilter::Debug
        } else {
            const KEY_MAX_LOG_LEVEL: &str = "smee.maxLogLevel";
            if let Ok(opt_value) = git_config_get(KEY_MAX_LOG_LEVEL, ValueType::String)
                && let Some(value) = opt_value
            {
                // The Git configuration entry is present
                // NOTE: We fallback on the default level if the value cannot be parsed,
                // partly to be nice and partly because we cannot log an error yet
                LevelFilter::from_str(&value).unwrap_or(LevelFilter::Info)
            } else {
                // Default level
                LevelFilter::Info
            }
        };
        // Initialize logger
        logger::init(max_level);
        // Apply color policy
        match command.color {
            ColorPolicy::Auto => {
                owo_colors::unset_override();
            }
            ColorPolicy::Always => {
                owo_colors::set_override(true);
            }
            ColorPolicy::Never => {
                owo_colors::set_override(false);
            }
        }
        // Run subcommand
        if let Err(err) = match command.subcommand {
            SmeeSubcommand::Install => self.run_install(),
            SmeeSubcommand::Uninstall => self.run_uninstall(),
            SmeeSubcommand::Activate(args) => self.run_activate(&args),
            SmeeSubcommand::Deactivate(args) => self.run_deactivate(&args),
            SmeeSubcommand::Hook(args) => self.run_hook(&args),
        } {
            // Report error and exit with 1
            report_error(&err);
            process::exit(1);
        }
    }

    fn run_install(self) -> Result<()> {
        // Determine hooks path
        let hooks_path = git_hooks_path()?;
        fs::create_dir_all(&hooks_path)
            .wrap_err_with(|| format!("cannot create hooks path {}", hooks_path.display()))?;
        // Install hooks
        let total_count = HOOKS.len();
        let mut created_count = 0usize;
        for hook in HOOKS {
            let file = hooks_path.join(hook);
            if file.exists() {
                if is_smee_hook(&file)? {
                    log::debug!("hook {} already installed", hook);
                    continue;
                }
                // Save existing hook
                let bak_file = hooks_path.join(format!("{hook}.{EXT_BAK}"));
                if bak_file.exists() {
                    bail!("saved hook {} already exists", bak_file.display());
                }
                fs::rename(&file, &bak_file).wrap_err_with(|| {
                    format!("cannot save {} as {}", file.display(), bak_file.display())
                })?;
                log::debug!("hook {} saved as {}", file.display(), bak_file.display());
            }
            // Create hook
            debug_assert!(!file.exists());
            let contents = HOOK_TEMPLATE
                .replace("{beacon}", HOOK_BEACON)
                .replace("{hook}", hook);
            fs::write(&file, &contents)
                .wrap_err_with(|| format!("cannot write {}", file.display()))?;
            // Make hook executable
            fs::set_permissions(&file, Permissions::from_mode(0o755))
                .wrap_err_with(|| format!("cannot set permissions of {}", file.display()))?;
            created_count += 1;
            log::debug!("installed hook {}", hook);
        }
        log::info!(
            "{} hooks installed ({} created, {} already installed)",
            total_count,
            created_count,
            total_count - created_count
        );
        Ok(())
    }

    fn run_uninstall(self) -> Result<()> {
        // Determine hooks path
        let hooks_path = git_hooks_path()?;
        // Uninstall hooks
        let total_count = HOOKS.len();
        let mut removed_count = 0usize;
        for hook in HOOKS {
            let file = hooks_path.join(hook);
            if !file.exists() || !is_smee_hook(&file)? {
                log::debug!("hook {} not installed", hook);
                continue;
            }
            // Remove hook
            fs::remove_file(&file).wrap_err_with(|| format!("cannot remove {}", file.display()))?;
            removed_count += 1;
            log::debug!("uninstalled hook {}", hook);
            let bak_file = hooks_path.join(format!("{hook}.{EXT_BAK}"));
            if bak_file.exists() {
                // Restore saved hook
                fs::rename(&bak_file, &file).wrap_err_with(|| {
                    format!(
                        "cannot restore {} as {}",
                        bak_file.display(),
                        file.display()
                    )
                })?;
                log::debug!("hook {} restored as {}", bak_file.display(), file.display());
            }
        }
        log::info!(
            "{} hooks uninstalled ({} removed, {} not installed)",
            total_count,
            removed_count,
            total_count - removed_count
        );
        Ok(())
    }

    fn run_activate(self, args: &ActivateArgs) -> Result<()> {
        let scope = to_git_scope(&args.scope);
        for hook in &args.hooks {
            activate_hook(hook, scope)?;
        }
        Ok(())
    }

    fn run_deactivate(self, args: &DeactivateArgs) -> Result<()> {
        let scope = to_git_scope(&args.scope);
        for hook in &args.hooks {
            deactivate_hook(hook, scope)?;
        }
        Ok(())
    }

    fn run_hook(self, args: &HookArgs) -> Result<()> {
        let event = &args.event;
        log::debug!("running {event} hooks");
        // Short-circuit if Smee is disabled
        if env::var_os(SMEE_DISABLE).is_some() {
            log::warn!("Smee disabled ({SMEE_DISABLE} environment variable set)");
            return Ok(());
        }
        let hooks = collect_hooks(event)?;
        if hooks.is_empty() {
            // No hooks, short-circuit
            return Ok(());
        }
        // Capture stdin
        let opt_stdin_file =
            create_stdin_file().wrap_err("cannot buffer stdin into temporary file")?;
        // Run hooks
        for hook in &hooks {
            let mut command = process::Command::new(&hook.command);
            command.args(&args.args);
            if let Some(stdin_file) = &opt_stdin_file {
                let stdin_file = stdin_file.try_clone()?;
                command.stdin(Stdio::from(stdin_file));
            }
            let status = command
                .status()
                .wrap_err_with(|| format!("cannot execute: {}", hook.command.display()))?;
            let code = status
                .code()
                .ok_or_eyre("hook command terminated by a signal")?;
            if code != 0 {
                bail!("hook {}:{event} exited with code {code}", hook.name);
            }
            log::info!("hook {}:{event} successful", hook.name);
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Hook {
    name: String,
    command: PathBuf,
}

/// Git hooks.
///
/// [Reference](https://git-scm.com/docs/githooks)
#[rustfmt::skip]
const HOOKS: [&str; 21] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    // "pre-receive",
    // "update",
    // "proc-receive",
    // "post-receive",
    // "post-update",
    "reference-transaction",
    // "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    // "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

const KEY_HOOKS_NAMES: &str = "smee.hooks";

const KEY_HOOKS_DIRS: &str = "smee.dirs";

/// Name of the environment variable set to disable Smee.
const SMEE_DISABLE: &str = "SMEE_DISABLE";

/// Text that indicates a hook is managed by Smee.
const HOOK_BEACON: &str = "This file is managed by Smee. Please do not modify it.";

/// Smee hook template.
const HOOK_TEMPLATE: &str = include_str!("hook-template");

/// File extension for hooks saved by Smee.
const EXT_BAK: &str = "smeebak";

/// Reports an error via a log message.
fn report_error(error: &Report) {
    // Use the alternate form that looks like `error: source 0: source 1`
    log::error!("{error:#}");
}

/// Returns whether `file` is a hook managed by Smee.
fn is_smee_hook<P>(file: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    let file = file.as_ref();
    let contents = fs::read_to_string(file)
        .wrap_err_with(|| format!("cannot read file {}", file.display()))?;
    Ok(contents.contains(HOOK_BEACON))
}

fn to_git_scope(hook_scope_args: &HookScopeArgs) -> Scope {
    if hook_scope_args.global {
        Scope::Global
    } else {
        Scope::Local
    }
}

fn activate_hook(hook: &str, scope: Scope) -> Result<()> {
    git_config_add(KEY_HOOKS_NAMES, hook, ValueType::String, false, scope)?;
    log::info!("hook {hook} activated in {scope} scope");
    Ok(())
}

fn deactivate_hook(hook: &str, scope: Scope) -> Result<()> {
    git_config_remove(KEY_HOOKS_NAMES, hook, scope)?;
    log::info!("hook {hook} deactivated in {scope} scope");
    Ok(())
}

fn collect_hooks(event: &str) -> Result<Vec<Hook>> {
    let mut hooks = collect_project_hooks(event)?;
    hooks.extend(collect_user_hooks(event)?);
    Ok(hooks)
}

fn collect_project_hooks(event: &str) -> Result<Vec<Hook>> {
    let mut hooks = Vec::<Hook>::new();
    let work_tree = git_work_tree()?;
    let base_dir = work_tree.join(".config").join("smee").join("hooks");
    if !base_dir.is_dir() {
        return Ok(hooks);
    }
    // Examine <base_dir>/<event>
    let path = base_dir.join(event);
    if let Some(hook) = try_new_hook(String::from("project"), path) {
        hooks.push(hook);
    }
    // Examine <base_dir>/<name>/<event> for all <name>
    for res_entry in base_dir.read_dir()? {
        let entry = res_entry?;
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            // Ignore subdirectories that are invalid UTF-8
            Err(_) => continue,
        };
        let path = &entry.path();
        if !path.is_dir() {
            continue;
        }
        let path = path.join(event);
        if let Some(hook) = try_new_hook(name, path) {
            hooks.push(hook);
        }
    }
    Ok(hooks)
}

fn collect_user_hooks(event: &str) -> Result<Vec<Hook>> {
    let hooks_names = git_config_get_all(KEY_HOOKS_NAMES, ValueType::String)?;
    if hooks_names.is_empty() {
        return Ok(Vec::new());
    }
    let hooks_dirs = git_config_get_all(KEY_HOOKS_DIRS, ValueType::Path)?
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let mut hooks = Vec::<Hook>::with_capacity(hooks_names.len());
    for name in &hooks_names {
        for hooks_dir in &hooks_dirs {
            // Examine <hooks_dir>/<name>/<event>
            let path = hooks_dir.join(name).join(event);
            if let Some(hook) = try_new_hook(name.clone(), path) {
                hooks.push(hook);
            }
        }
    }
    Ok(hooks)
}

fn try_new_hook(name: String, path: PathBuf) -> Option<Hook> {
    if path.is_file() {
        if path.is_executable() {
            let hook = Hook {
                name,
                command: path,
            };
            Some(hook)
        } else {
            log::warn!("{} not executable", path.display());
            None
        }
    } else {
        None
    }
}

fn create_stdin_file() -> Result<Option<File>> {
    let mut buffer = Vec::<u8>::new();
    let len = io::stdin().read_to_end(&mut buffer)?;
    let opt_file = if len > 0 {
        log::debug!("capturing {len} bytes from stdin to temporary file");
        let mut file = tempfile::tempfile()?;
        file.write_all(&buffer)?;
        Some(file)
    } else {
        log::debug!("nothing to capture from stdin");
        None
    };
    Ok(opt_file)
}
