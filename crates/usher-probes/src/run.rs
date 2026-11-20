//! The probe runner: find the harness, check the control, then the claim.

use std::{ffi::OsStr, fmt, path::PathBuf};

use usher_testkit::{Probe, ScratchHome, probe};

use crate::fragment::{Date, Fragment, Network, Outcome, bare_version};

/// Why a claim could not be measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unmeasured {
    /// The claim needs credentials the probe must not have.
    NeedsCredentials,
    /// The claim needs a network the probe runs without.
    NeedsNetwork,
    /// The run broke; the text says how.
    Broken(String),
}

/// A probe that could not run. It is never a pass and never a refutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRun {
    kind: Unmeasured,
    text: String,
}

/// What a probe is about and where its harness is looked for.
#[derive(Debug, Clone, Copy)]
pub struct Spec<'a> {
    /// The probe's name.
    pub name: &'a str,
    /// The harness binary name.
    pub binary: &'a str,
    /// Where to look for it; `PATH` when `None`.
    pub search_path: Option<&'a OsStr>,
    /// The network state the run is under, recorded in the fragment.
    pub network: Network,
}

/// The harness a probe runs against.
#[derive(Debug)]
pub struct Harness<'a> {
    /// Where the binary was found.
    pub path: PathBuf,
    /// Its bare version number.
    pub version: String,
    /// The scratch home it runs under.
    pub scratch: &'a ScratchHome,
}

impl fmt::Display for Unmeasured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unmeasured::NeedsCredentials => f.write_str("needs credentials"),
            Unmeasured::NeedsNetwork => f.write_str("needs network"),
            Unmeasured::Broken(text) => f.write_str(text),
        }
    }
}

impl NotRun {
    fn new(kind: Unmeasured) -> NotRun {
        let text = kind.to_string();
        NotRun { kind, text }
    }

    /// The kind of reason.
    pub fn kind(&self) -> &Unmeasured {
        &self.kind
    }

    /// The reason, without the `not run:` prefix.
    pub fn reason(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for NotRun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not run: {}", self.text)
    }
}

/// Runs one probe. The control runs first; when it fails, the claim is not run
/// and the probe is `NotRun`. A claim that holds is `Passed`, one that does not
/// is `Failed`, and one that cannot be measured is `NotRun`.
pub fn run<C, K>(
    spec: &Spec<'_>,
    scratch: &ScratchHome,
    today: Date,
    control: C,
    claim: K,
) -> Result<Fragment, NotRun>
where
    C: FnOnce(&Harness<'_>) -> Result<(), String>,
    K: FnOnce(&Harness<'_>) -> Result<bool, Unmeasured>,
{
    let (path, version_line) = match probe(spec.binary, spec.search_path, scratch) {
        Probe::Found { path, version } => (path, version),
        Probe::Missing { reason } => return Err(NotRun::new(Unmeasured::Broken(reason))),
    };
    let Some(version) = bare_version(&version_line) else {
        return Err(NotRun::new(Unmeasured::Broken(format!(
            "`{} --version` printed no version number",
            spec.binary
        ))));
    };
    let harness = Harness {
        path,
        version,
        scratch,
    };

    control(&harness)
        .map_err(|text| NotRun::new(Unmeasured::Broken(format!("the control failed: {text}"))))?;
    let held = claim(&harness).map_err(NotRun::new)?;

    let outcome = if held {
        Outcome::Passed
    } else {
        Outcome::Failed
    };
    Fragment::new(&harness.version, outcome, today, spec.network)
        .map_err(|err| NotRun::new(Unmeasured::Broken(err.to_string())))
}

/// Runs one probe and panics with the `not run:` message when it cannot run,
/// so a maintainer's `--ignored` run never reads an unrun probe as ok. The
/// fragment is printed on stdout.
pub fn run_or_panic<C, K>(
    spec: &Spec<'_>,
    scratch: &ScratchHome,
    today: Date,
    control: C,
    claim: K,
) -> Fragment
where
    C: FnOnce(&Harness<'_>) -> Result<(), String>,
    K: FnOnce(&Harness<'_>) -> Result<bool, Unmeasured>,
{
    match run(spec, scratch, today, control, claim) {
        Ok(fragment) => {
            println!("probe {}: {}", spec.name, fragment.render());
            fragment
        }
        Err(not_run) => panic!("{not_run}"),
    }
}
