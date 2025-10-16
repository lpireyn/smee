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

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

/// Smee, a Git hooks manager.
#[derive(Debug, Parser)]
#[command(version)]
pub struct SmeeCommand {
    /// Do not print information log messages.
    #[arg(conflicts_with = "verbose", long, short = 'q')]
    pub quiet: bool,

    /// Print debug log messages.
    #[arg(conflicts_with = "quiet", long, short = 'v')]
    pub verbose: bool,

    /// Specify when to use colors in printed messages.
    ///
    /// [no value: always]
    // NOTE: This option is not global so as not to interfere with the open arguments of the `hook` subcommand
    // TODO: Make Clap document the `default_missing_value` rather than mimicking it with `[no value: ...]`
    #[arg(
        default_missing_value = "always",
        default_value = "auto",
        long,
        num_args = 0..=1,
        require_equals = true,
        value_name = "WHEN"
    )]
    pub color: ColorPolicy,

    #[command(subcommand)]
    pub subcommand: SmeeSubcommand,
}

/// Color policy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ColorPolicy {
    /// Use colors for printed messages
    /// if the `NO_COLOR` environment variable is not set
    /// and the standard output is a TTY.
    #[default]
    Auto,

    /// Use colors for printed messages.
    #[value(aliases = ["on", "yes"])]
    Always,

    /// Do not use colors for printed messages.
    #[value(aliases = ["off", "no"])]
    Never,
}

#[derive(Debug, Subcommand)]
#[command()]
pub enum SmeeSubcommand {
    /// Install Smee in a Git repository.
    #[command()]
    Install,

    /// Uninstall Smee from a Git repository.
    #[command()]
    Uninstall,

    /// Activate a user hook.
    #[command()]
    Activate(ActivateArgs),

    /// Deactivate a user hook.
    #[command()]
    Deactivate(DeactivateArgs),

    /// Run a hook with Smee.
    #[command()]
    Hook(HookArgs),
}

#[derive(Args, Debug)]
pub struct ActivateArgs {
    #[command(flatten)]
    pub scope: HookScopeArgs,

    /// Hook(s) to activate.
    #[arg(required = true, value_name = "HOOK")]
    pub hooks: Vec<String>,
}

#[derive(Args, Debug)]
pub struct DeactivateArgs {
    #[command(flatten)]
    pub scope: HookScopeArgs,

    /// Hook(s) to deactivate.
    #[arg(required = true, value_name = "HOOK")]
    pub hooks: Vec<String>,
}

#[derive(Args, Debug)]
pub struct HookArgs {
    /// Event (e.g., `pre-commit`).
    #[arg(value_name = "EVENT")]
    pub event: String,

    /// Arguments.
    #[arg(value_name = "ARGS")]
    pub args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct HookScopeArgs {
    /// Apply to the local project only (default).
    #[arg(conflicts_with = "global", long)]
    pub local: bool,

    /// Apply to all projects.
    #[arg(conflicts_with = "local", long)]
    pub global: bool,
}
