//! Capabilities and the support level usher reports for a harness version.

use std::fmt;

use crate::Version;
use crate::schema::KnowledgeVersion;

/// One thing usher can do for a harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// List the settings sources of the harness.
    Inventory,
    /// Install or update the harness.
    Install,
    /// Map each setting in effect to its provenance.
    Map,
    /// Compare two environments.
    Compare,
    /// Place a setting in a chosen layer.
    Place,
    /// Apply a confirmed change.
    Apply,
}

impl Capability {
    /// Every capability, in report order.
    pub const ALL: [Capability; 6] = [
        Capability::Inventory,
        Capability::Install,
        Capability::Map,
        Capability::Compare,
        Capability::Place,
        Capability::Apply,
    ];

    /// The name used in the knowledge file and in reports.
    pub fn name(self) -> &'static str {
        match self {
            Capability::Inventory => "inventory",
            Capability::Install => "install",
            Capability::Map => "map",
            Capability::Compare => "compare",
            Capability::Place => "place",
            Capability::Apply => "apply",
        }
    }

    /// Parses a capability name; `None` for anything outside the set.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What usher offers for one harness at one installed version.
///
/// A capability that is not offered reads as unsupported. An unknown harness
/// or a version with no support entry offers nothing, and says so with
/// [`SupportReport::known`], so absence is never a clean result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportReport {
    pub(crate) harness: String,
    pub(crate) installed: Version,
    pub(crate) known: bool,
    pub(crate) offered: Vec<Capability>,
    pub(crate) knowledge: KnowledgeVersion,
}

impl SupportReport {
    /// The harness the report is about.
    pub fn harness(&self) -> &str {
        &self.harness
    }

    /// The installed harness version the report was built for.
    pub fn installed(&self) -> Version {
        self.installed
    }

    /// Whether the knowledge file has a support entry for this harness and
    /// version at all.
    pub fn known(&self) -> bool {
        self.known
    }

    /// The knowledge file version the report relies on.
    pub fn knowledge_version(&self) -> KnowledgeVersion {
        self.knowledge
    }

    /// Whether `capability` is offered.
    pub fn offers(&self, capability: Capability) -> bool {
        self.offered.contains(&capability)
    }

    /// The capabilities offered, in report order.
    pub fn offered(&self) -> Vec<Capability> {
        Capability::ALL
            .into_iter()
            .filter(|c| self.offers(*c))
            .collect()
    }

    /// The capabilities that read as unsupported, in report order.
    pub fn unsupported(&self) -> Vec<Capability> {
        Capability::ALL
            .into_iter()
            .filter(|c| !self.offers(*c))
            .collect()
    }

    /// One line per capability, then the knowledge version used, ready to
    /// print.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.known {
            lines.push(format!(
                "{} {}: no support entry in the knowledge file, so nothing is offered",
                self.harness, self.installed
            ));
        }
        for capability in Capability::ALL {
            let state = if self.offers(capability) {
                "offered"
            } else {
                "unsupported"
            };
            lines.push(format!(
                "{} {}: {capability} {state}",
                self.harness, self.installed
            ));
        }
        lines.push(format!("{}", self.knowledge));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_names_round_trip_and_unknown_names_are_none() {
        for capability in Capability::ALL {
            assert_eq!(Capability::parse(capability.name()), Some(capability));
        }
        assert_eq!(Capability::parse("teleport"), None);
        assert_eq!(Capability::parse("Inventory"), None);
    }
}
