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

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::CommandFactory;
use clap::ValueEnum;
use clap_complete::Shell;

fn main() -> Result<(), Box<dyn Error>> {
    let mut command = smee::cli::SmeeCommand::command();
    let out_dir = PathBuf::from("target/complete");
    fs::create_dir_all(&out_dir)?;
    for &shell in Shell::value_variants() {
        clap_complete::generate_to(shell, &mut command, "smee", &out_dir)?;
    }
    Ok(())
}
