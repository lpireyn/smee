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

    #[command(subcommand)]
    pub subcommand: SmeeSubcommand,
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

    /// Run a hook with Smee.
    #[command()]
    Hook(HookArgs),
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
