use super::*;

#[test]
fn prepend_preserves_unrelated_entries_and_removes_only_selected_duplicates() {
    let old = r#"C:\Windows\System32;D:\Tools;"D:\Node";D:\OtherNode;"#;
    let dirs = vec![r"d:\node".to_owned()];
    let result = edit_path(old, &dirs, false, |s| s.to_owned());
    assert_eq!(
        result,
        r"d:\node;C:\Windows\System32;D:\Tools;D:\OtherNode;"
    );
    assert_eq!(edit_path(&result, &dirs, false, |s| s.to_owned()), result);
    assert_eq!(
        edit_path(&result, &dirs, true, |s| s.to_owned()),
        r"C:\Windows\System32;D:\Tools;D:\OtherNode;"
    );
}

#[test]
fn removal_matches_expanded_paths_but_preserves_raw_variables_and_siblings() {
    let dirs = vec![
        r"D:\Runtime\tools".into(),
        r"D:\Runtime\tools\Scripts".into(),
    ];
    let value = r"%ROOT%\tools;%ROOT%\tools\Scripts;%ROOT%\tools-other;%SystemRoot%";
    assert_eq!(
        edit_path(value, &dirs, true, |s| s.replace("%ROOT%", r"D:\Runtime")),
        r"%ROOT%\tools-other;%SystemRoot%"
    );
}

#[test]
fn machine_conflict_requires_explicit_machine_scope() {
    assert_eq!(choose_scope(false, true, false), Scope::Machine);
    assert_eq!(choose_scope(false, false, false), Scope::User);
    assert_eq!(choose_scope(true, true, false), Scope::User);
    assert_eq!(choose_scope(true, false, true), Scope::Machine);
}

#[test]
fn windows_path_key_handles_quotes_slashes_case_and_extended_prefix() {
    assert_eq!(key(r#""D:/Node/""#), key(r"\\?\d:\node"));
    assert_ne!(key(r"D:\Node"), key(r"D:\Node-old"));
    assert_eq!(
        display_path(r"\\?\UNC\server\share\node"),
        r"\\server\share\node"
    );
}

#[test]
fn unsupported_families_cannot_modify_path() {
    assert_eq!(command("node"), Some("node"));
    assert_eq!(command("python"), Some("python"));
    assert_eq!(command("mariadb"), None);
    assert_eq!(command("dotnet-sdk"), None);
}

fn environment(machine: &str, user: &str) -> Environment {
    let mut env = Environment::default();
    env.machine.insert(
        "PATH".into(),
        environment::Value {
            text: machine.into(),
            kind: 2,
        },
    );
    env.user.insert(
        "PATH".into(),
        environment::Value {
            text: user.into(),
            kind: 2,
        },
    );
    env
}

#[test]
fn resolver_prefers_machine_and_detects_an_external_reordering_without_saved_state() {
    let root = tempfile::tempdir().unwrap();
    let machine = root.path().join("machine");
    let user = root.path().join("user");
    std::fs::create_dir_all(&machine).unwrap();
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(machine.join("node.cmd"), b"fixture").unwrap();
    std::fs::write(user.join("node.exe"), b"fixture").unwrap();
    let env = environment(&machine.to_string_lossy(), &user.to_string_lossy());
    assert_eq!(
        env.first("node", None).unwrap(),
        machine.join("node.cmd").to_string_lossy()
    );
    let env = environment("", &user.to_string_lossy());
    assert_eq!(
        env.first("node", None).unwrap(),
        user.join("node.exe").to_string_lossy()
    );
}

#[test]
fn expanded_registered_paths_block_uninstall_including_nested_scripts() {
    let root = tempfile::tempdir().unwrap();
    let install = root.path().join("python");
    let scripts = install.join("Scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    let mut env = environment("", "%RUNTIME%/Scripts");
    env.user.insert(
        "RUNTIME".into(),
        environment::Value {
            text: install.to_string_lossy().into(),
            kind: 1,
        },
    );
    assert!(DefaultsSnapshot(env).protect(&install).is_err());
    assert!(
        DefaultsSnapshot(environment(
            "",
            &root.path().join("python-other").to_string_lossy()
        ))
        .protect(&install)
        .is_ok()
    );
}

#[test]
fn raw_registry_strings_do_not_expand_and_missing_variables_are_preserved() {
    let mut env = environment("", "");
    env.machine.insert(
        "RUNTIME".into(),
        environment::Value {
            text: "machine".into(),
            kind: 1,
        },
    );
    env.user.insert(
        "RUNTIME".into(),
        environment::Value {
            text: "user".into(),
            kind: 1,
        },
    );
    assert_eq!(env.expand("%RUNTIME%", Scope::Machine, 2), "machine");
    assert_eq!(env.expand("%RUNTIME%", Scope::User, 2), "user");
    assert_eq!(env.expand("%RUNTIME%", Scope::User, 1), "%RUNTIME%");
    assert_eq!(
        env.expand("%AEVRIK_NONEXISTENT_VARIABLE%", Scope::User, 2),
        "%AEVRIK_NONEXISTENT_VARIABLE%"
    );
}

#[test]
fn python_includes_scripts_and_unrepresentable_paths_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let exe = root.path().join("Python tools").join("python.exe");
    let paths = directories(&exe, "python").unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths[1].ends_with("Scripts"));
    assert_eq!(directories(&exe, "node").unwrap().len(), 1);
    for folder in ["bad;folder", "bad%folder", "bad\"folder"] {
        assert!(directories(&root.path().join(folder).join("node.exe"), "node").is_err());
    }
}

#[test]
fn user_path_reference_expands_system_path_and_composite_entries_are_resolved() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a").to_string_lossy().into_owned();
    let b = root.path().join("b").to_string_lossy().into_owned();
    let mut env = environment(&a, "%PATH%;%TOOL_PATH%");
    env.user.insert(
        "TOOL_PATH".into(),
        environment::Value {
            text: b.clone(),
            kind: 1,
        },
    );
    assert_eq!(env.paths(Some(Scope::User)), vec![a, b]);
    env.user.insert(
        "TOOL_PATH".into(),
        environment::Value {
            text: "%RUNTIME%".into(),
            kind: 1,
        },
    );
    env.user.insert(
        "RUNTIME".into(),
        environment::Value {
            text: "must-not-expand".into(),
            kind: 1,
        },
    );
    assert_eq!(env.expand("%TOOL_PATH%", Scope::User, 2), "%RUNTIME%");
}

#[test]
fn resolver_honors_custom_pathext_and_ignores_path_fragments() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("node.tool"), b"fixture").unwrap();
    std::fs::write(root.path().join("node.exe"), b"fixture").unwrap();
    let mut env = environment(&root.path().to_string_lossy(), "");
    env.user.insert(
        "PATHEXT".into(),
        environment::Value {
            text: ".TOOL;.EXE;../evil".into(),
            kind: 1,
        },
    );
    assert_eq!(env.extensions(), vec![".tool", ".exe"]);
    assert!(env.first("node", None).unwrap().ends_with("node.tool"));
}

#[cfg(windows)]
#[test]
fn plans_require_a_managed_install_and_stale_confirmation_never_writes_path() {
    let root = tempfile::tempdir().unwrap();
    let manager = PackageManager::new(root.path().to_owned()).unwrap();
    let package = super::super::catalog::all()
        .unwrap()
        .into_iter()
        .find(|p| p.family == "node")
        .unwrap();
    let empty = environment("", "");
    assert!(manager.default_plan(&package.id, false, &empty).is_err());
    assert!(manager.default_plan("../external", false, &empty).is_err());
    let directory = root.path().join(&package.id).join(&package.version);
    let executable = directory.join(&package.executable);
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    std::fs::write(&executable, b"fixture; never executed").unwrap();
    std::fs::write(
        directory.join(".aevrik-package.json"),
        serde_json::to_vec(&package).unwrap(),
    )
    .unwrap();
    let plan = manager.default_plan(&package.id, false, &empty).unwrap();
    assert_eq!(plan.scope, Scope::User);
    let changed = environment(&executable.parent().unwrap().to_string_lossy(), "");
    let machine_plan = manager.default_plan(&package.id, false, &changed).unwrap();
    assert_eq!(machine_plan.scope, Scope::Machine);
    assert_ne!(plan.fingerprint, machine_plan.fingerprint);
    let status = DefaultsSnapshot(changed).status("node", Some(&executable));
    assert!(status.active && status.on_path);
    assert!(
        !DefaultsSnapshot(empty)
            .status("node", Some(&executable))
            .active
    );
    // This deliberately invalid hash is checked before backups or any registry write.
    let error = manager
        .apply_default(&package.id, false, Scope::User, "never-valid-fingerprint")
        .unwrap_err();
    assert!(error.to_string().contains("changed since"));
    assert!(!std::fs::read_dir(root.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("path-backup-")
    }));
    std::fs::remove_file(&executable).unwrap();
    assert!(
        manager
            .default_plan(&package.id, false, &environment("", ""))
            .is_err()
    );
}

#[cfg(windows)]
#[test]
fn builtin_profile_paths_select_machine_scope_and_block_uninstall() {
    const FIXTURE: &str = "AEVRIK_TEST_BUILTIN_PROFILE_ROOT";
    let Some(root) = std::env::var_os(FIXTURE) else {
        // Give only this child test bootstrap variables pointing to a disposable folder.
        // Do not call set_var in the multithreaded test runner or write real profile folders.
        let root = tempfile::tempdir().unwrap();
        let test = concat!(
            module_path!(),
            "::builtin_profile_paths_select_machine_scope_and_block_uninstall"
        );
        let test = test.split_once("::").unwrap().1;
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--nocapture"])
            .env(FIXTURE, root.path())
            .env("ALLUSERSPROFILE", root.path())
            .env("PUBLIC", root.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("1 passed"),
            "child test must actually run"
        );
        return;
    };
    let root = PathBuf::from(root);
    let tools = root.join("Tools");
    std::fs::create_dir_all(&tools).unwrap();
    std::fs::write(
        tools.join("node.exe"),
        b"system runtime fixture; never executed",
    )
    .unwrap();

    let manager = PackageManager::new(root.join("managed")).unwrap();
    let package = super::super::catalog::all()
        .unwrap()
        .into_iter()
        .find(|p| p.family == "node")
        .unwrap();
    let directory = manager.root.join(&package.id).join(&package.version);
    let exe = directory.join(&package.executable);
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    std::fs::write(&exe, b"selected runtime fixture; never executed").unwrap();
    std::fs::write(
        directory.join(".aevrik-package.json"),
        serde_json::to_vec(&package).unwrap(),
    )
    .unwrap();

    for name in ["ALLUSERSPROFILE", "PUBLIC", "allusersprofile", "Public"] {
        let raw = format!("%{name}%\\Tools");
        // Neither bootstrap variable exists in the registry maps in this fixture.
        let env = environment(&raw, "");
        assert_eq!(
            env.paths(Some(Scope::Machine)),
            vec![tools.to_string_lossy().into_owned()],
            "{name}"
        );
        let plan = manager.default_plan(&package.id, false, &env).unwrap();
        assert_eq!(plan.scope, Scope::Machine, "{name}");
        assert_eq!(
            plan.current_executable,
            Some(tools.join("node.exe").to_string_lossy().into_owned())
        );
        assert!(
            DefaultsSnapshot(env).protect(&tools).is_err(),
            "system PATH: {name}"
        );
        assert!(
            DefaultsSnapshot(environment("", &raw))
                .protect(&tools)
                .is_err(),
            "user PATH: {name}"
        );
        assert!(
            DefaultsSnapshot(environment(&raw, ""))
                .protect(&root.join("Tools-other"))
                .is_ok()
        );
        // REG_SZ still preserves literal percent expressions.
        assert_eq!(environment("", "").expand(&raw, Scope::Machine, 1), raw);
    }
}
