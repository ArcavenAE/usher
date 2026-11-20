//! Probes live only in `src/probes/`, the scan over them reads every nested
//! file and knows eleven ways to start a process, no ignored test sits outside
//! `src/probes/`, and the launcher has no method that sets a variable.

use std::{fs, path::Path};

use usher_probes::structural::{
    find_bypasses_in_tree, find_env_setters, find_launch_bypasses, find_misplaced_ignored_tests,
};
use usher_testkit::ScratchHome;

const NEEDLES: [&str; 11] = [
    "Command::new",
    "process::Command",
    "scratch.command",
    ".command(",
    "ScratchHome::new",
    "usher_testkit::probe",
    "std::process",
    "libc::",
    "posix_spawn",
    "fork(",
    "exec",
];

/// The attribute, built in two pieces so this file does not hold it itself.
fn ignore_attribute() -> String {
    concat!("#[", "ignore").to_string()
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    fs::write(path, text).expect("a file");
}

#[test]
fn each_of_the_eleven_needles_alone_is_a_bypass_and_a_launcher_call_is_not() {
    assert!(find_launch_bypasses("fn clean() { launcher.run(\"x\", &[]); }").is_empty());
    for needle in NEEDLES {
        let fixture = format!("fn bypass() {{ {needle} }}");
        assert!(
            !find_launch_bypasses(&fixture).is_empty(),
            "the scan missed the needle `{needle}`"
        );
    }
}

#[test]
fn the_scan_reads_nested_directories_and_names_the_file() {
    let scratch = ScratchHome::new().expect("scratch home");
    let clean = scratch.root().join("clean");
    write(
        &clean.join("a/b/ok.rs"),
        "fn ok() { launcher.run(\"x\", &[]); }\n",
    );
    // Control: a tree with only clean files finds nothing.
    assert!(
        find_bypasses_in_tree(&clean)
            .expect("the tree reads")
            .is_empty()
    );

    let dirty = scratch.root().join("dirty");
    write(
        &dirty.join("a/b/ok.rs"),
        "fn ok() { launcher.run(\"x\", &[]); }\n",
    );
    write(
        &dirty.join("a/b/c/deep.rs"),
        "fn bypass(s: &S) { s.thing.command(\"x\"); }\n",
    );
    let found = find_bypasses_in_tree(&dirty).expect("the tree reads");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].0.ends_with("a/b/c/deep.rs"), "{found:?}");
    assert!(found[0].1.contains(".command("), "{found:?}");

    // The real probes directory exists, holds a file and is clean.
    let probes = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/probes");
    assert!(
        probes.join("mod.rs").is_file(),
        "src/probes/mod.rs is missing"
    );
    assert!(
        find_bypasses_in_tree(&probes)
            .expect("the real tree reads")
            .is_empty()
    );
}

#[test]
fn an_ignored_test_outside_src_probes_fails_the_placement_check() {
    let scratch = ScratchHome::new().expect("scratch home");
    let attribute = ignore_attribute();
    let fixture = format!("{attribute} = \"child\"]\n#[test]\nfn child() {{}}\n");

    // Control: ignored tests inside src/probes, nested or not, are placed.
    let placed = scratch.root().join("placed");
    write(&placed.join("src/probes/mod.rs"), &fixture);
    write(&placed.join("src/probes/deep/more.rs"), &fixture);
    write(&placed.join("tests/plain.rs"), "#[test]\nfn plain() {}\n");
    assert!(
        find_misplaced_ignored_tests(&placed)
            .expect("the crate reads")
            .is_empty()
    );

    let misplaced = scratch.root().join("misplaced");
    write(&misplaced.join("src/probes/mod.rs"), &fixture);
    write(&misplaced.join("tests/child.rs"), &fixture);
    write(&misplaced.join("src/lib.rs"), &fixture);
    let found = find_misplaced_ignored_tests(&misplaced).expect("the crate reads");
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found.iter().any(|path| path.ends_with("tests/child.rs")));
    assert!(found.iter().any(|path| path.ends_with("src/lib.rs")));

    // This crate holds none.
    let real = find_misplaced_ignored_tests(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("this crate reads");
    assert!(
        real.is_empty(),
        "ignored tests outside src/probes: {real:?}"
    );
}

#[test]
fn the_launcher_has_no_method_that_sets_an_environment_variable() {
    assert!(
        find_env_setters("impl Launcher { pub fn run(&self) {} pub fn harness_home(&self) {} }")
            .is_empty()
    );
    for method in [
        "fn env(",
        "fn envs(",
        "fn env_clear(",
        "fn env_remove(",
        "fn set_env(",
        "fn with_env(",
        "fn add_env(",
    ] {
        let fixture = format!("impl Launcher {{ pub {method}&mut self) {{}} }}");
        assert!(
            !find_env_setters(&fixture).is_empty(),
            "the scan missed `{method}`"
        );
    }

    let run = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/run.rs"))
        .expect("run.rs reads");
    assert!(find_env_setters(&run).is_empty());
}

// One scan covers all of this crate's src with the eleven needles. The one
// exemption is the launcher's spawn site: the `run` method of `Launcher`, in
// the file `run.rs`.
const SPAWN_SITE: &str = "impl Launcher<'_> {
    /// Starts `program` with `args` and returns its output.
    pub fn run(&self, program: impl AsRef<OsStr>, args: &[&str]) -> io::Result<Output> {
        self.starts.set(self.starts.get() + 1);
        self.scratch.command(program).args(args).output()
    }

    /// How many processes this launcher has started.
    pub fn starts(&self) -> usize {
        self.starts.get()
    }
}
";

fn tree_with(scratch: &ScratchHome, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let root = scratch.root().join(name).join("src");
    for (file, text) in files {
        write(&root.join(file), text);
    }
    root
}

#[test]
fn the_launchers_spawn_site_passes_and_a_needle_anywhere_else_in_src_fails() {
    let scratch = ScratchHome::new().expect("scratch home");
    let site = tree_with(
        &scratch,
        "site",
        &[("run.rs", SPAWN_SITE), ("lib.rs", "pub mod run;\n")],
    );
    assert!(
        find_bypasses_in_tree(&site)
            .expect("the tree reads")
            .is_empty(),
        "the launcher's own spawn site was refused"
    );

    for (file, text) in [
        ("lib.rs", "fn f() { Command::new(\"x\"); }\n"),
        (
            "deep/er/mod.rs",
            "fn f(s: &S) { s.thing.command(\"x\"); }\n",
        ),
        ("network.rs", "use std::process::Command;\n"),
    ] {
        let tree = tree_with(
            &scratch,
            &format!("needle-{}", file.replace('/', "-")),
            &[("run.rs", SPAWN_SITE), (file, text)],
        );
        let found = find_bypasses_in_tree(&tree).expect("the tree reads");
        assert_eq!(found.len(), 1, "for {file}: {found:?}");
        assert!(found[0].0.ends_with(file), "for {file}: {found:?}");
    }
}

#[test]
fn a_second_spawn_in_the_exempt_file_fails_wherever_it_sits() {
    let scratch = ScratchHome::new().expect("scratch home");
    // Control: the file with only the spawn site is clean.
    let clean = tree_with(&scratch, "clean", &[("run.rs", SPAWN_SITE)]);
    assert!(
        find_bypasses_in_tree(&clean)
            .expect("the tree reads")
            .is_empty()
    );

    let second = [
        // A free function in the same file.
        format!("{SPAWN_SITE}\nfn sneaky() {{ std::process::Command::new(\"y\"); }}\n"),
        // Another method of the same impl.
        SPAWN_SITE.replace(
            "    /// How many processes",
            "    pub fn other(&self) {\n        self.scratch.command(\"y\");\n    }\n\n    /// How many processes",
        ),
        // A second spawn inside the exempt function itself.
        SPAWN_SITE.replace(
            "        self.scratch.command(program).args(args).output()",
            "        self.scratch.command(\"y\").output().ok();\n        self.scratch.command(program).args(args).output()",
        ),
    ];
    for (n, text) in second.iter().enumerate() {
        assert_ne!(text, SPAWN_SITE, "case {n} changed nothing");
        let tree = tree_with(&scratch, &format!("second-{n}"), &[("run.rs", text)]);
        let found = find_bypasses_in_tree(&tree).expect("the tree reads");
        assert!(!found.is_empty(), "a second spawn went unseen (case {n})");
        assert!(found.iter().all(|(path, _)| path.ends_with("run.rs")));
    }

    // The spawn site's own text in another file is not exempt there.
    let moved = tree_with(&scratch, "moved", &[("other.rs", SPAWN_SITE)]);
    assert!(
        !find_bypasses_in_tree(&moved)
            .expect("the tree reads")
            .is_empty(),
        "the exemption followed the function's text into another file"
    );
}

#[test]
fn all_of_this_crates_src_is_clean_under_the_one_scan() {
    // Control: the scan sees a bypass in a tree before its silence here means anything.
    let scratch = ScratchHome::new().expect("scratch home");
    let dirty = tree_with(
        &scratch,
        "dirty",
        &[("lib.rs", "fn f() { Command::new(\"x\"); }\n")],
    );
    assert!(
        !find_bypasses_in_tree(&dirty)
            .expect("the tree reads")
            .is_empty()
    );

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let found = find_bypasses_in_tree(&src).expect("the real src reads");
    assert!(
        found.is_empty(),
        "src starts a process on its own: {found:?}"
    );
}

// Moved from launcher.rs with the old scan, which read src except run.rs and
// network.rs for three needles: the fixtures it used are still bypasses.
#[test]
fn the_fixtures_of_the_removed_scan_are_still_bypasses() {
    for fixture in [
        "fn bypass() { std::process::Command::new(\"x\"); }",
        "use std::process::Command;",
        "fn bypass(s: &S) { s.scratch.command(\"x\"); }",
        "let c = Command::new(\"sh\");",
    ] {
        assert!(
            !find_launch_bypasses(fixture).is_empty(),
            "the scan missed: {fixture}"
        );
    }
    assert!(find_launch_bypasses("fn clean() { launcher.run(\"x\", &[]); }").is_empty());
}

// Q8: the crate has one spawn site. network.rs builds an argv and starts
// nothing, so it holds no spawn; one added back there fails.
#[test]
fn network_rs_holds_no_spawn_and_one_added_back_fails() {
    let argv_only = "pub fn denied_argv(program: &str) -> Vec<String> {\n    vec![SANDBOX_EXEC.to_string(), program.to_string()]\n}\n";
    let spawning = "pub fn run_denied(scratch: &ScratchHome, program: &str) -> io::Result<Output> {\n    scratch.command(SANDBOX_EXEC).arg(program).output()\n}\n";
    let scratch = ScratchHome::new().expect("scratch home");

    let pure = tree_with(
        &scratch,
        "net-pure",
        &[("run.rs", SPAWN_SITE), ("network.rs", argv_only)],
    );
    assert!(
        find_bypasses_in_tree(&pure)
            .expect("the tree reads")
            .is_empty()
    );

    let back = tree_with(
        &scratch,
        "net-back",
        &[("run.rs", SPAWN_SITE), ("network.rs", spawning)],
    );
    let found = find_bypasses_in_tree(&back).expect("the tree reads");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].0.ends_with("network.rs"), "{found:?}");

    let real = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/network.rs"))
        .expect("network.rs reads");
    assert!(
        find_launch_bypasses(&real).is_empty(),
        "network.rs starts a process: {:?}",
        find_launch_bypasses(&real)
    );
}

// Q8: the exec needle is a call form, and the scan reads code, not comments.
#[test]
fn exec_is_a_call_form_and_comments_are_not_scanned() {
    for flagged in [
        "fn f() { exec(\"x\"); }",
        "fn f() { execvp(\"x\", &[]); }",
        "fn f() { libc::execve(p, a, e); }",
        "fn f() { Command::new(\"x\"); } // a trailing comment does not hide it",
        "// a comment line\nfn f() { Command::new(\"x\"); }",
    ] {
        assert!(
            !find_launch_bypasses(flagged).is_empty(),
            "the scan missed: {flagged}"
        );
    }
    for clean in [
        "const SANDBOX_EXEC: &str = \"/usr/bin/sandbox-exec\";",
        "// runs under sandbox-exec(1), which is not exec( here",
        "/// The sandbox-exec tool; Command::new is not called.",
        "//! Module doc: std::process, exec( and execvp stay in prose.",
        "fn f() { launcher.run(\"x\", &[]); } // calls exec( through the launcher",
        "fn executes_now() { let executed = 1; }",
    ] {
        assert!(
            find_launch_bypasses(clean).is_empty(),
            "the scan flagged: {clean}"
        );
    }
}
