//! The version types knowledge exposes are the ones usher-core defines, so a
//! range read by either is the same type and no caller converts.

use std::any::TypeId;

#[test]
fn knowledge_re_exports_the_core_version_types() {
    assert_eq!(
        TypeId::of::<usher_knowledge::Version>(),
        TypeId::of::<usher_core::Version>(),
        "Version is two different types"
    );
    assert_eq!(
        TypeId::of::<usher_knowledge::VersionRange>(),
        TypeId::of::<usher_core::VersionRange>(),
        "VersionRange is two different types"
    );
}

#[test]
fn a_version_parsed_through_either_crate_reads_the_same() {
    let through_knowledge = usher_knowledge::Version::parse("2.1.329").expect("a version");
    let through_core = usher_core::Version::parse("2.1.329").expect("a version");
    assert_eq!(through_knowledge.to_string(), through_core.to_string());
    assert_eq!(through_core.to_string(), "2.1.329");
}
