# Smee

**Smee** is a *Git hooks manager*.

[![GitHub Release](https://img.shields.io/github/v/release/lpireyn/smee?logo=github&color=blue)](https://github.com/lpireyn/smee/releases/latest)
[![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/lpireyn/smee/ci.yml?branch=main&logo=github&label=CI)](https://github.com/lpireyn/smee/actions/workflows/ci.yml?query=branch%3Amain)
[![Written in Rust](https://img.shields.io/badge/Written_in-Rust-orange?logo=rust)](https://rust-lang.org/)
[![License](https://img.shields.io/github/license/lpireyn/smee?logo=apache&color=purple)](LICENSE)

<img src="assets/mr-smee.png" alt="Mr. Smee (Disney, 1953)" width="151" height="240" align="right"/>

> Mr. Smee is Captain Hook's git.
> Smee is the Captain of Git hooks!

## Features

- Supports *project hooks* and *user hooks*
- Only one command: `smee` &mdash; no dependencies!
- Configured entirely with `git config` &mdash; no configuration file!
- Written in Rust :crab: &mdash; small and blazingly fast!

## Documentation

[Git hooks](https://git-scm.com/docs/githooks) are commands that are run at certain points during Git operations.
Their names correspond to the events that trigger their execution: `pre-commit`, `post-rewrite`, etc.

Smee helps you organize and run Git hooks.

### Start using Smee

To let Smee manage the hooks in a repository, run the following command in that repository:

```shell
smee install
```

Smee distinguishes between two categories of hooks: *project hooks* and *user hooks*.

### Project hooks

Project hooks are versioned along with the sources of a project and are intended to be run by all contributors.
They typically rely on the same tools as the project itself.

**Example:**
a project hook `test` that runs the test suite of the project before each commit.

Project hooks are stored in a project under the `.config/smee/hooks` directory.
If there is only one hook for an event (e.g., `pre-commit`), it can be stored directly under this directory (e.g., `.config/smee/hooks/pre-commit`).
If there are several hooks, it is recommended to organize them in subdirectories (e.g., `.config/smee/hooks/test/pre-commit`, `.config/smee/hooks/style/pre-commit`, etc.).

Project hooks are always run by Smee.

### User hooks

User hooks are activated by a user, either for a specific project or for all of them.
Since they are not shared with other contributors, they can use tools that are only available on that user's system.

**Example:**
a user hook `notifications/commit` that displays a notification on each successful commit.

User hooks can be stored anywhere on your system.
You register a hooks directory with:

```shell
git config set --global --append smee.dirs DIRECTORY
```

Similarly, you deregister a hooks directory with:

```shell
git config unset --global --value=DIRECTORY smee.dirs
```

Once Smee knows where to look for user hooks, you can activate a specific hook with:

```shell
git config set SCOPE --append smee.hooks HOOK
```

where `SCOPE` is either:

- `--global` to activate the hook for all projects
- `--local` to activate the hook only for the local repository

Similarly, you deactivate a hook with:

```shell
git config unset SCOPE --value=HOOK smee.hooks
```

**Note:**
Since Smee is configured entirely within the Git configuration,
you may want to edit the corresponding files directly rather than using the `git config` command.
This can be useful to reorder the active hooks, for example.

### Disable Smee

Sometimes you need to disable Smee for an operation (e.g., you're creating a temporary commit with changes that don't compile).
You can do that by setting the `SMEE_DISABLE` environment variable:

```shell
SMEE_DISABLE=1 git commit -a -m 'wip'
```

**Note:** The actual value of the variable doesn't matter, as long as it is set.

### Stop using Smee

If you ever want to stop using Smee in a repository, run the following command in that repository:

```shell
smee uninstall
```

If there were existing hooks when Smee was installed, they will be restored.

### Configuration

Smee is configured entirely with the Git configuration, under the `smee` section.

| Key                | Description                                                                        |
|--------------------|------------------------------------------------------------------------------------|
| `smee.dirs`        | Directories where Smee looks for user hooks (multiple values)                      |
| `smee.hooks`       | Active user hooks (multiple values)                                                |
| `smee.maxLogLevel` | Maximum log level (valid values: `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`) |

## Changelog

See [CHANGELOG](CHANGELOG.md).

## License

Smee is licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).
See [LICENSE](LICENSE).
