//! Tests for write_skills (the bundled-skill materializer).

use super::{write_skills, SKILLS};

#[test]
fn writes_every_bundled_skill_with_content() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path().join(".claude").join("skills");

    let written = write_skills(&base, true).expect("write");
    assert_eq!(written.len(), SKILLS.len(), "all bundled skills written");

    for (name, _) in SKILLS {
        let path = base.join(name).join("SKILL.md");
        let body = std::fs::read_to_string(&path).expect("skill file present");
        assert!(body.contains("---"), "{name} carries frontmatter");
        assert!(body.contains(&format!("name: {name}")), "{name} frontmatter names itself");
    }
}

#[test]
fn non_overwrite_leaves_existing_files_untouched() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path().join("skills");

    // Seed one skill with custom content.
    let (first, _) = SKILLS[0];
    let seeded = base.join(first).join("SKILL.md");
    std::fs::create_dir_all(seeded.parent().expect("parent")).expect("mkdir");
    std::fs::write(&seeded, "CUSTOM").expect("seed");

    let written = write_skills(&base, false).expect("write");

    // The seeded one is skipped; the rest are filled in.
    assert!(!written.contains(&first), "existing skill not overwritten");
    assert_eq!(written.len(), SKILLS.len() - 1);
    assert_eq!(std::fs::read_to_string(&seeded).expect("read"), "CUSTOM");
}

#[test]
fn overwrite_replaces_existing_content() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path().join("skills");

    let (first, _) = SKILLS[0];
    let target = base.join(first).join("SKILL.md");
    std::fs::create_dir_all(target.parent().expect("parent")).expect("mkdir");
    std::fs::write(&target, "STALE").expect("seed");

    write_skills(&base, true).expect("write");
    let body = std::fs::read_to_string(&target).expect("read");
    assert_ne!(body, "STALE", "overwrite replaced the stale content");
}
