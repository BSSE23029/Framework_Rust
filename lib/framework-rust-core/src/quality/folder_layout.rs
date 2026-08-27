use std::{fs, path::Path};

const MAX_FILES_PER_FOLDER: usize = 5;

#[test]
fn maintained_folders_have_at_most_five_files() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .expect("core crate is inside the workspace");
    let mut violations = Vec::new();

    inspect(workspace_root, workspace_root, &mut violations);

    assert!(
        violations.is_empty(),
        "folders exceed the {MAX_FILES_PER_FOLDER}-file limit:\n{}",
        violations.join("\n")
    );
}

fn inspect(root: &Path, directory: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    let mut file_count = 0;
    let mut children = Vec::new();

    for entry in entries {
        let entry = entry.expect("directory entry is readable");
        let path = entry.path();
        let file_type = entry.file_type().expect("entry type is readable");
        if file_type.is_file() {
            file_count += 1;
        } else if file_type.is_dir() && entry.file_name() != "target" {
            children.push(path);
        }
    }

    if file_count > MAX_FILES_PER_FOLDER {
        let relative = directory.strip_prefix(root).unwrap_or(directory);
        violations.push(format!("{}: {file_count} files", relative.display()));
    }

    for child in children {
        inspect(root, &child, violations);
    }
}
