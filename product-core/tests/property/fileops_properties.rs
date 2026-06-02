//! TC-P010–P011: File write safety property tests (ADR-018)

use proptest::prelude::*;
use product_lib::fileops;

/// TC-P011: Write + re-read is identity
/// ∀content:String: read(atomic_write(path, content)) = content
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn tc_p011_write_read_identity(content in "[\\PC]{0,1000}") {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.md");
        fileops::write_file_atomic(&path, &content).expect("write");
        let read_back = std::fs::read_to_string(&path).expect("read");
        prop_assert_eq!(&read_back, &content);
    }
}

/// TC-P010: Atomic write never leaves partial content
/// After write, file is either the new content or does not exist (never partial)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn tc_p010_atomic_write_no_partial(
        original in "[a-z]{10,50}",
        new_content in "[A-Z]{10,50}",
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.md");

        // Write original
        std::fs::write(&path, &original).expect("write original");

        // Atomic overwrite
        fileops::write_file_atomic(&path, &new_content).expect("atomic write");

        // Read back — must be exactly the new content
        let read_back = std::fs::read_to_string(&path).expect("read");
        prop_assert_eq!(&read_back, &new_content);

        // No tmp files should remain
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .expect("readdir")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.contains(".product-tmp."))
                    .unwrap_or(false)
            })
            .collect();
        prop_assert!(entries.is_empty(), "tmp files should be cleaned up");
    }
}
