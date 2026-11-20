//! The structural checks between harness knowledge and the probes in the repo,
//! and the scans that keep every process start in one place.
//!
//! The scans read raw source text. They do not skip comments or strings, so a
//! comment or a string that writes one of the call forms they look for fails
//! them too; reword it, for example as "the process builder", and never change
//! the scan. The needles are built from pieces in this file so that it does not
//! hold them itself.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// One way the two sides disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// An item names a probe that the repo does not have.
    MissingProbe(String),
    /// A probe in the repo is named by no item.
    UncarriedProbe(String),
}

/// Checks that every probe name carried by knowledge resolves to a probe in
/// the repo, and that every probe in the repo is carried by an item. Both
/// kinds of mismatch are reported together, missing probes first.
pub fn check_probe_names(carried: &[&str], in_repo: &[&str]) -> Result<(), Vec<Mismatch>> {
    let mut mismatches = Vec::new();
    for name in carried {
        let reported = mismatches.contains(&Mismatch::MissingProbe((*name).to_string()));
        if !in_repo.contains(name) && !reported {
            mismatches.push(Mismatch::MissingProbe((*name).to_string()));
        }
    }
    for name in in_repo {
        let reported = mismatches.contains(&Mismatch::UncarriedProbe((*name).to_string()));
        if !carried.contains(name) && !reported {
            mismatches.push(Mismatch::UncarriedProbe((*name).to_string()));
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches)
    }
}

/// The twelve call forms of starting a process, built in pieces so this file
/// does not hold them.
const NEEDLES: [&str; 12] = [
    concat!("Command", "::new"),
    concat!("process", "::Command"),
    concat!("scratch", ".command"),
    concat!(".comm", "and("),
    concat!("ScratchHome", "::new"),
    concat!("usher_testkit", "::probe"),
    concat!("std", "::process"),
    concat!("libc", "::"),
    concat!("posix", "_spawn"),
    concat!("for", "k("),
    concat!("ex", "ec("),
    concat!("ex", "ecv"),
];

/// The methods that would set an environment variable on a launcher.
const ENV_SETTERS: [&str; 7] = [
    concat!("fn ", "env("),
    concat!("fn ", "envs("),
    concat!("fn ", "env_clear("),
    concat!("fn ", "env_remove("),
    concat!("fn ", "set_env("),
    concat!("fn ", "with_env("),
    concat!("fn ", "add_env("),
];

/// The marker of an ignored test.
const IGNORE: &str = concat!("#[", "ignore");

/// The one method that may start a process: the launcher's, in `src/run.rs`.
const SPAWN_SITE: &str = "pub fn run(&self";

/// The ways a source file starts a process on its own instead of through the
/// launcher: each entry is a needle found in `source`. Empty when it is clean.
pub fn find_launch_bypasses(source: &str) -> Vec<String> {
    found(source, &NEEDLES)
}

/// Every `.rs` file under `dir`, nested directories included, that starts a
/// process on its own: the file and the needles found in it, joined by commas.
/// The one exemption is the launcher's spawn site: the first line of its `run`
/// method that holds a needle, in the file `run.rs` directly under a directory
/// named `src`. A second needle line in that method, or any needle elsewhere in
/// that file, is flagged like any other.
pub fn find_bypasses_in_tree(dir: &Path) -> io::Result<Vec<(PathBuf, String)>> {
    let mut flagged = Vec::new();
    for file in rs_files(dir)? {
        let text = fs::read_to_string(&file)?;
        let is_spawn_file = dir.file_name().is_some_and(|name| name == "src")
            && file
                .strip_prefix(dir)
                .is_ok_and(|rel| rel == Path::new("run.rs"));
        let scanned = if is_spawn_file {
            without_spawn_site(&text)
        } else {
            text
        };
        let hits = find_launch_bypasses(&scanned);
        if !hits.is_empty() {
            flagged.push((file, hits.join(", ")));
        }
    }
    Ok(flagged)
}

/// `source` with the one needle line of the launcher's `run` method blanked.
fn without_spawn_site(source: &str) -> String {
    let Some(start) = source.find(SPAWN_SITE) else {
        return source.to_string();
    };
    let Some(open) = source[start..].find('{').map(|at| start + at) else {
        return source.to_string();
    };
    let mut depth = 0usize;
    let mut end = source.len();
    for (at, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = open + at + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    let mut blanked = false;
    let method: String = source[start..end]
        .split_inclusive('\n')
        .map(|line| {
            if !blanked && !find_launch_bypasses(line).is_empty() {
                blanked = true;
                "\n"
            } else {
                line
            }
        })
        .collect();
    format!("{}{}{}", &source[..start], method, &source[end..])
}

/// Every `.rs` file under `crate_dir/src` and `crate_dir/tests` outside the
/// directory `crate_dir/src/probes` that holds an ignored test.
pub fn find_misplaced_ignored_tests(crate_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let probes = crate_dir.join("src").join("probes");
    let mut misplaced = Vec::new();
    for root in [crate_dir.join("src"), crate_dir.join("tests")] {
        if !root.is_dir() {
            continue;
        }
        for file in rs_files(&root)? {
            if file.starts_with(&probes) {
                continue;
            }
            if fs::read_to_string(&file)?.contains(IGNORE) {
                misplaced.push(file);
            }
        }
    }
    Ok(misplaced)
}

/// The methods in `source` that set an environment variable on a launcher.
pub fn find_env_setters(source: &str) -> Vec<String> {
    found(source, &ENV_SETTERS)
}

fn found(source: &str, needles: &[&str]) -> Vec<String> {
    needles
        .iter()
        .filter(|needle| source.contains(**needle))
        .map(|needle| (*needle).to_string())
        .collect()
}

/// Every `.rs` file under `dir`, nested directories included, in name order.
fn rs_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<_>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files.extend(rs_files(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(files)
}
