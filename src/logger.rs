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

use log::Level;
use log::LevelFilter;
use log::Log;
use log::Metadata;
use log::Record;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stderr;
use owo_colors::style;

/// Simple logger that prints messages to stderr, with a bit of colors.
#[derive(Debug)]
struct Logger {
    max_level: LevelFilter,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let (prefix, style) = match record.level() {
                Level::Error => ("error", style().bright_red()),
                Level::Warn => ("warning", style().yellow()),
                Level::Info => ("info", style().green()),
                Level::Debug => ("debug", style().blue()),
                Level::Trace => ("trace", style().white()),
            };
            eprintln!(
                "{} {}",
                format!("{prefix}:").if_supports_color(Stderr, |s| s.style(style)),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Installs the logger.
///
/// # Panics
///
/// This function panics if a logger has already been installed.
pub fn install(max_level: LevelFilter) {
    let logger = Logger { max_level };
    // NOTE: `SetLoggerError` does *not* implement `Error`
    log::set_boxed_logger(Box::new(logger)).expect("logger already installed");
    log::set_max_level(max_level);
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    /// Demo of the different levels.
    ///
    /// This test should be run with the `--no-capture` option.
    #[test]
    fn demo() {
        super::install(LevelFilter::Trace);
        log::error!("this is a message at the ERROR level");
        log::warn!("this is a message at the WARN level");
        log::info!("this is a message at the INFO level");
        log::debug!("this is a message at the DEBUG level");
        log::trace!("this is a message at the TRACE level");
    }
}
