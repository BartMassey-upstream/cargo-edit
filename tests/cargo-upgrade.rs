#[macro_use]
extern crate pretty_assertions;

mod utils;

use crate::utils::{
    clone_out_test, copy_workspace_test, execute_command, execute_command_for_pkg,
    execute_command_in_dir, get_toml, setup_alt_registry_config,
};
use assert_cmd::Command;

// Verify that an upgraded Cargo.toml matches what we expect.
#[test]
fn upgrade_as_expected() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.source");

    execute_command(&["upgrade"], &manifest);

    let upgraded = get_toml(&manifest);
    let target = get_toml("tests/fixtures/upgrade/Cargo.toml.target");

    assert_eq!(target.to_string(), upgraded.to_string());
}

#[test]
fn upgrade_all() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with the dependency `docopt@0.8.0`
    execute_command(&["add", "docopt", "--vers", "0.8.0"], &manifest);

    // Now, upgrade `docopt` to the latest version
    execute_command(&["upgrade"], &manifest);

    // Verify that `docopt` has been updated successfully.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["docopt"].as_str(),
        Some("99999.0.0")
    );
}

#[test]
fn upgrade_all_allow_prerelease() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with `docopt`
    execute_command(&["add", "docopt", "--vers", "0.8"], &manifest);

    // Now, upgrade `docopt` to the latest version
    execute_command(&["upgrade", "--allow-prerelease"], &manifest);

    // Verify that `docopt` has been updated successfully.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["docopt"].as_str(),
        Some("99999.0.0-alpha.1")
    );
}

#[test]
fn upgrade_prereleased_without_the_flag() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with alpha `b`.
    execute_command(&["add", "b", "--vers", "0.8.0-alpha"], &manifest);

    // Now, upgrade `b` to its latest version
    execute_command(&["upgrade", "b"], &manifest);

    // Verify that `b` has been updated successfully to a prerelease version.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["b"].as_str(),
        Some("99999.0.0-alpha.1")
    );
}

#[test]
fn upgrade_prerelease_already_prereleased() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with stable `a` and alpha `b`.
    execute_command(&["add", "a", "--vers", "1.0"], &manifest);
    execute_command(&["add", "b", "--vers", "0.8.0-alpha"], &manifest);

    // Now, upgrade all dependencies to their latest versions
    execute_command(&["upgrade"], &manifest);

    // Verify that `a` has been updated successfully to a stable version.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["a"].as_str(),
        Some("99999.0.0")
    );
    // Verify that `b` has been updated successfully to a prerelease version.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["b"].as_str(),
        Some("99999.0.0-alpha.1")
    );
}

#[test]
fn upgrade_all_dry_run() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with the dependency `docopt@0.8`
    execute_command(&["add", "docopt", "--vers", "0.8"], &manifest);

    // Now, upgrade `docopt` to the latest version
    execute_command(&["upgrade", "--dry-run"], &manifest);

    // Verify that `docopt` has not been updated.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["docopt"].as_str(),
        Some("0.8")
    );
}

#[test]
fn upgrade_all_allow_prerelease_dry_run() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with the dependency `docopt@0.8`
    execute_command(&["add", "docopt", "--vers", "0.8"], &manifest);

    // Now, upgrade `docopt` to the latest version
    execute_command(&["upgrade", "--allow-prerelease", "--dry-run"], &manifest);

    // Verify that `docopt` has not been updated.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["docopt"].as_str(),
        Some("0.8")
    );
}

#[test]
fn upgrade_specified_only() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with the dependencies `env_proxy` and `docopt`
    execute_command(&["add", "docopt", "--vers", "0.8"], &manifest);
    execute_command(&["add", "env_proxy", "--vers", "0.1.1"], &manifest);

    // Update `docopt` to the latest version
    execute_command(&["upgrade", "docopt"], &manifest);

    // Verify that `docopt` was upgraded, but not `env_proxy`
    let dependencies = &get_toml(&manifest)["dependencies"];
    assert_eq!(dependencies["docopt"].as_str(), Some("99999.0.0"));
    assert_eq!(dependencies["env_proxy"].as_str(), Some("0.1.1"));
}

#[test]
fn upgrade_skip_compatible() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    execute_command(&["add", "test_breaking", "--vers", "0.1"], &manifest);
    execute_command(&["add", "test_nonbreaking", "--vers", "0.1"], &manifest);

    execute_command(&["upgrade", "--skip-compatible"], &manifest);

    // Verify that `test_breaking` was upgraded, but not `test_nonbreaking`
    let dependencies = &get_toml(&manifest)["dependencies"];
    assert_eq!(dependencies["test_breaking"].as_str(), Some("0.2.0"));
    assert_eq!(dependencies["test_nonbreaking"].as_str(), Some("0.1"));
}

#[test]
fn fails_to_upgrade_missing_dependency() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // `failure` is not a dependency. Try to upgrade it anyway.
    execute_command(&["upgrade", "failure"], &manifest);

    // Verify that `failure` has not been added
    assert!(get_toml(&manifest)["dependencies"]["failure"].is_none());
}

#[test]
fn upgrade_optional_dependency() {
    // Set up a Cargo.toml with an optional dependency `test_optional_dependency` verifies that this
    // is correct.
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");
    execute_command(
        &["add", "docopt", "--vers", ">=0.1.1", "--optional"],
        &manifest,
    );

    // Now, update without including the `optional` flag.
    execute_command(&["upgrade"], &manifest);

    // Dependency present afterwards - correct version, and still optional.
    let toml = get_toml(&manifest);
    let val = &toml["dependencies"]["docopt"];
    assert_eq!(val["version"].as_str(), Some("99999.0.0"));
    assert_eq!(val["optional"].as_bool(), Some(true));
}

#[test]
fn upgrade_with_exclude() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest with the dependency `docopt@0.8`
    execute_command(&["add", "docopt", "--vers", "0.8"], &manifest);

    // Then upgrade everything except `docopt`
    execute_command(&["upgrade", "--exclude", "docopt"], &manifest);

    // And finally verify that `docopt` has not been updated.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["docopt"].as_str(),
        Some("0.8")
    );
}

#[test]
fn upgrade_renamed_dependency_with_exclude() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.renamed_dep");

    // Upgrade everything except `rx` aka `regex`
    execute_command(&["upgrade", "--exclude", "rx"], &manifest);

    // And verify that `rx` has not been updated.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["rx"]["version"].as_str(),
        Some("0.2")
    );
}

#[test]
fn upgrade_renamed_dependency_all() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.renamed_dep");

    execute_command(&["upgrade"], &manifest);

    let toml = get_toml(&manifest);

    let dep1 = &toml["dependencies"]["te"];
    assert_eq!(dep1["version"].as_str(), Some("99999.0.0"));

    let dep2 = &toml["dependencies"]["rx"];
    assert_eq!(dep2["version"].as_str(), Some("99999.0.0"));
}

#[test]
fn upgrade_renamed_dependency_inline_specified_only() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.renamed_dep");

    execute_command(&["upgrade", "toml_edit"], &manifest);

    let toml = get_toml(&manifest);
    let dep = &toml["dependencies"]["te"];
    assert_eq!(dep["version"].as_str(), Some("99999.0.0"));
}

#[test]
fn upgrade_renamed_dependency_table_specified_only() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.renamed_dep");

    execute_command(&["upgrade", "regex"], &manifest);

    let toml = get_toml(&manifest);
    let dep = &toml["dependencies"]["rx"];
    assert_eq!(dep["version"].as_str(), Some("99999.0.0"));
}

#[test]
fn upgrade_alt_registry_dependency_all() {
    let (tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.alt_registry");
    setup_alt_registry_config(tmpdir.path());

    // The alternative registry test commands are run
    // from the test directory, as cargo metadata probes for
    // cargo config relative to the invocation directory, not
    // the manifest path.
    execute_command_in_dir(&["upgrade"], tmpdir.path());

    let toml = get_toml(&manifest);

    let dep1 = &toml["dependencies"]["toml_edit"];
    assert_eq!(dep1["version"].as_str(), Some("99999.0.0"));
    assert_eq!(dep1["registry"].as_str(), Some("alternative"));

    let dep2 = &toml["dependencies"]["regex"];
    assert_eq!(dep2["version"].as_str(), Some("99999.0.0"));
    assert_eq!(dep2["registry"].as_str(), Some("alternative"));
}

#[test]
fn upgrade_alt_registry_dependency_inline_specified_only() {
    let (tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.alt_registry");
    setup_alt_registry_config(tmpdir.path());

    execute_command_in_dir(&["upgrade", "toml_edit"], tmpdir.path());

    let toml = get_toml(&manifest);
    let dep = &toml["dependencies"]["toml_edit"];
    assert_eq!(dep["version"].as_str(), Some("99999.0.0"));
    assert_eq!(dep["registry"].as_str(), Some("alternative"));
}

#[test]
fn upgrade_alt_registry_dependency_table_specified_only() {
    let (tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.alt_registry");
    setup_alt_registry_config(tmpdir.path());

    execute_command_in_dir(&["upgrade", "regex"], tmpdir.path());

    let toml = get_toml(&manifest);
    let dep = &toml["dependencies"]["regex"];
    assert_eq!(dep["version"].as_str(), Some("99999.0.0"));
    assert_eq!(dep["registry"].as_str(), Some("alternative"));
}

#[test]
fn upgrade_at() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/add/Cargo.toml.sample");

    // Setup manifest
    execute_command(&["add", "docopt", "--vers", "0.8"], &manifest);

    // Now, upgrade `docopt` to a version that seems unlikely to ever get published.
    execute_command(&["upgrade", "docopt@1000000.0.0"], &manifest);

    // Verify that `docopt` has been updated to the specified version.
    assert_eq!(
        get_toml(&manifest)["dependencies"]["docopt"].as_str(),
        Some("1000000.0.0")
    );
}

#[test]
fn all_flag_is_deprecated() {
    let (_tmpdir, root_manifest, _workspace_manifests) = copy_workspace_test();

    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&["upgrade", "--all", "--manifest-path", &root_manifest])
        .env("CARGO_IS_TEST", "1")
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "The flag `--all` has been deprecated in favor of `--workspace`",
        ));
}

#[test]
fn upgrade_workspace_all() {
    let (_tmpdir, root_manifest, workspace_manifests) = copy_workspace_test();

    execute_command(&["upgrade", "--workspace"], &root_manifest);

    // All of the workspace members have `libc` as a dependency.
    for workspace_member in workspace_manifests {
        assert_eq!(
            get_toml(&workspace_member)["dependencies"]["libc"].as_str(),
            Some("99999.0.0")
        );
    }
}

#[test]
fn upgrade_workspace_workspace() {
    let (_tmpdir, root_manifest, workspace_manifests) = copy_workspace_test();

    execute_command(&["upgrade", "--workspace"], &root_manifest);

    // All of the workspace members have `libc` as a dependency.
    for workspace_member in workspace_manifests {
        assert_eq!(
            get_toml(&workspace_member)["dependencies"]["libc"].as_str(),
            Some("99999.0.0")
        );
    }
}

#[test]
fn upgrade_dependency_in_workspace_member() {
    let (tmpdir, _root_manifest, workspace_manifests) = copy_workspace_test();
    execute_command_for_pkg(&["upgrade", "libc"], "one", &tmpdir);

    let one = workspace_manifests
        .iter()
        .map(|manifest| get_toml(manifest))
        .find(|manifest| manifest["package"]["name"].as_str() == Some("one"))
        .expect("Couldn't find workspace member `one'");

    assert_eq!(
        one["dependencies"]["libc"]
            .as_str()
            .expect("libc dependency did not exist"),
        "99999.0.0",
    );
}

/// Detect if attempting to run against a workspace root and give a helpful warning.
#[test]
#[cfg(feature = "test-external-apis")]
fn detect_workspace() {
    let (_tmpdir, root_manifest, _workspace_manifests) = copy_workspace_test();

    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&["upgrade", "--manifest-path", &root_manifest])
        .assert()
        .code(1)
        .stderr(
            "Command failed due to unhandled error: Found virtual manifest, but this command \
         requires running against an actual package in this workspace. Try adding `--workspace`.",
        );
}

#[test]
fn invalid_manifest() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.invalid");

    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&["upgrade", "--manifest-path", &manifest])
        .env("CARGO_IS_TEST", "1")
        .assert()
        .code(1)
        .stderr(
            "\
Command failed due to unhandled error: Unable to parse Cargo.toml

Caused by: Manifest not valid TOML
Caused by: TOML parse error at line 1, column 6
  |
1 | This is clearly not a valid Cargo.toml.
  |      ^
Unexpected `i`
Expected `.` or `=`

",
        );
}

#[test]
fn invalid_root_manifest_all() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.invalid");

    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&["upgrade", "--workspace", "--manifest-path", &manifest])
        .env("CARGO_IS_TEST", "1")
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "Command failed due to unhandled error: Failed to get workspace metadata",
        ));
}

#[test]
fn invalid_root_manifest_workspace() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.invalid");

    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&["upgrade", "--workspace", "--manifest-path", &manifest])
        .env("CARGO_IS_TEST", "1")
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "Command failed due to unhandled error: Failed to get workspace metadata",
        ));
}

#[test]
fn unknown_flags() {
    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&["upgrade", "foo", "--flag"])
        .env("CARGO_IS_TEST", "1")
        .assert()
        .code(1)
        .stderr(
            "error: Found argument '--flag' which wasn't expected, or isn't valid in this context

USAGE:
    cargo upgrade [FLAGS] [OPTIONS] [--] [dependency]...

For more information try --help
",
        );
}

// Verify that an upgraded Cargo.toml matches what we expect.
#[test]
#[cfg(feature = "test-external-apis")]
fn upgrade_to_lockfile() {
    let (tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.lockfile_source");
    std::fs::copy(
        std::path::Path::new("tests/fixtures/upgrade/Cargo.lock"),
        tmpdir.path().join("Cargo.lock"),
    )
    .unwrap_or_else(|err| panic!("could not copy test lock file: {}", err));
    execute_command(&["upgrade", "--to-lockfile"], &manifest);

    let upgraded = get_toml(&manifest);
    let target = get_toml("tests/fixtures/upgrade/Cargo.toml.lockfile_target");

    assert_eq!(target.to_string(), upgraded.to_string());
}

#[test]
#[cfg(feature = "test-external-apis")]
fn upgrade_workspace_to_lockfile_all() {
    let (tmpdir, root_manifest, _workspace_manifests) = copy_workspace_test();

    execute_command(&["upgrade", "--workspace", "--to-lockfile"], &root_manifest);

    // The members one and two both request different, semver incompatible
    // versions of rand. Test that both were upgraded correctly.
    let one_upgraded = get_toml(tmpdir.path().join("one/Cargo.toml").to_str().unwrap());
    let one_target = get_toml("tests/fixtures/workspace/one/Cargo.toml.lockfile_target");
    assert_eq!(one_target.to_string(), one_upgraded.to_string());

    let two_upgraded = get_toml(tmpdir.path().join("two/Cargo.toml").to_str().unwrap());
    let two_target = get_toml("tests/fixtures/workspace/two/Cargo.toml.lockfile_target");
    assert_eq!(two_target.to_string(), two_upgraded.to_string());
}

#[test]
#[cfg(feature = "test-external-apis")]
fn upgrade_workspace_to_lockfile_workspace() {
    let (tmpdir, root_manifest, _workspace_manifests) = copy_workspace_test();

    execute_command(&["upgrade", "--workspace", "--to-lockfile"], &root_manifest);

    // The members one and two both request different, semver incompatible
    // versions of rand. Test that both were upgraded correctly.
    let one_upgraded = get_toml(tmpdir.path().join("one/Cargo.toml").to_str().unwrap());
    let one_target = get_toml("tests/fixtures/workspace/one/Cargo.toml.lockfile_target");
    assert_eq!(one_target.to_string(), one_upgraded.to_string());

    let two_upgraded = get_toml(tmpdir.path().join("two/Cargo.toml").to_str().unwrap());
    let two_target = get_toml("tests/fixtures/workspace/two/Cargo.toml.lockfile_target");
    assert_eq!(two_target.to_string(), two_upgraded.to_string());
}

#[test]
#[cfg(feature = "test-external-apis")]
fn upgrade_prints_messages() {
    let (_tmpdir, manifest) = clone_out_test("tests/fixtures/upgrade/Cargo.toml.source");

    Command::cargo_bin("cargo-upgrade")
        .expect("can find bin")
        .args(&[
            "upgrade",
            "docopt",
            &format!("--manifest-path={}", manifest),
        ])
        .assert()
        .success()
        .stderr(predicates::str::contains("docopt v0.8 -> v"));
}
