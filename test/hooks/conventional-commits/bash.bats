#!/usr/bin/env bats

setup_file() {
    project_dir="$BATS_TEST_DIRNAME"/../../..
    hooks_dir="$project_dir/hooks"
    export hook_cmd="$hooks_dir/conventional-commits/bash/commit-msg"
}

@test "valid messages" {
    for n in {1..3}; do
        printf '# valid %d\n' $n >&3
        run "$hook_cmd" "$BATS_TEST_DIRNAME/valid${n}.txt"
    done
}

@test "special messages" {
    for action in amend fixup squash; do
        printf '# %s\n' "$action" >&3
        run "$hook_cmd" "$BATS_TEST_DIRNAME/${action}.txt"
    done
}

@test "invalid messages" {
    bats_require_minimum_version 1.5.0
    for n in {1..11}; do
        printf '# invalid %d\n' $n >&3
        run ! "$hook_cmd" "$BATS_TEST_DIRNAME/invalid${n}.txt"
    done
}

# Conventional Commits examples
# https://www.conventionalcommits.org/en/v1.0.0/
@test "conventional commits examples" {
    for n in {1..7}; do
        printf '# example %d\n' $n >&3
        run "$hook_cmd" "$BATS_TEST_DIRNAME/example${n}.txt"
    done
}
