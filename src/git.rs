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

#![allow(dead_code)]

use std::path::PathBuf;
use std::process;

use eyre::OptionExt;
use eyre::Result;
use eyre::WrapErr;
use eyre::eyre;

/// Git configuration entry value type.
///
/// [Reference](https://git-scm.com/docs/git-config#Documentation/git-config.txt---typetype)
#[derive(Clone, Copy, Debug, Default)]
pub enum ValueType {
    #[default]
    String,
    Bool,
    Int,
    BoolOrInt,
    Path,
    ExpiryDate,
    Color,
}

pub fn git<A, S, F, T>(args: A, mapper: F) -> Result<T>
where
    A: AsRef<[S]>,
    S: AsRef<str>,
    F: FnOnce(i32, &process::Output) -> Option<Result<T>>,
{
    fn git_command_line(args: &[&str]) -> String {
        format!("git {}", args.join(" "))
    }

    let args = args.as_ref().iter().map(|s| s.as_ref()).collect::<Vec<_>>();
    log::debug!("executing: {}", git_command_line(&args));
    let output = process::Command::new("git")
        .args(&args)
        .output()
        .wrap_err_with(|| format!("cannot execute: {}", git_command_line(&args)))?;
    let code = output
        .status
        .code()
        .ok_or_eyre("Git terminated by a signal")?;
    mapper(code, &output).ok_or_else(|| {
        eyre!(
            "command exited with code {code}: {}",
            git_command_line(&args)
        )
    })?
}

pub fn git_work_tree() -> Result<PathBuf> {
    git_rev_parse_path(&["--show-toplevel"])
}

pub fn git_hooks_path() -> Result<PathBuf> {
    git_rev_parse_path(&["--git-path", "hooks"])
}

pub fn git_config_get<K>(key: K, value_type: ValueType) -> Result<Option<String>>
where
    K: AsRef<str>,
{
    let key = key.as_ref();
    git(
        ["config", "get", value_type.git_option(), "--", key],
        |code, output| {
            match code {
                // Key found
                0 => {
                    let res_value = git_output_to_string(output.stdout.clone())
                        .wrap_err_with(|| format!("invalid Git configuration entry '{key}'"));
                    Some(res_value.map(|value| Some(value.trim().to_string())))
                }
                // Key not found
                1 => Some(Ok(None)),
                // Error
                _ => None,
            }
        },
    )
}

pub fn git_config_get_all<K>(key: K, value_type: ValueType) -> Result<Vec<String>>
where
    K: AsRef<str>,
{
    let key = key.as_ref();
    git(
        ["config", "get", "--all", value_type.git_option(), "--", key],
        |code, output| {
            match code {
                // Key found
                0 => {
                    let res_values = git_output_to_string(output.stdout.clone())
                        .wrap_err_with(|| format!("invalid Git configuration entry '{key}'"));
                    Some(res_values.map(|s| s.lines().map(|s| s.to_string()).collect()))
                }
                // Key not found
                1 => Some(Ok(Vec::new())),
                // Error
                _ => None,
            }
        },
    )
}

impl ValueType {
    fn git_option(&self) -> &'static str {
        match self {
            Self::String => "--no-type",
            Self::Bool => "--type=bool",
            Self::Int => "--type=int",
            Self::BoolOrInt => "--type=bool-or-int",
            Self::Path => "--type=path",
            Self::ExpiryDate => "--type=expiry-date",
            Self::Color => "--type=color",
        }
    }
}

fn git_output_to_string(output: Vec<u8>) -> Result<String> {
    String::from_utf8(output).wrap_err("Git output is invalid UTF-8")
}

fn git_rev_parse_path(args: &[&str]) -> Result<PathBuf> {
    let mut all_args = vec!["rev-parse", "--path-format=absolute"];
    all_args.extend(args);
    git(all_args, |code, output| {
        if code != 0 {
            return None;
        }
        let res_path = git_output_to_string(output.stdout.clone());
        Some(res_path.map(|s| PathBuf::from(s.trim())))
    })
}
