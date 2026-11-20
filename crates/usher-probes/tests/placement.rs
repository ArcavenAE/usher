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
