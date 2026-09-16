use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn deb_manifest_lists_share_paths() {
    let root = repo_root();
    let manifest = std::fs::read_to_string(root.join("apps/marius-agenda/Cargo.toml")).expect("manifest");
    assert!(manifest.contains("package.metadata.deb"));
    assert!(manifest.contains("usr/share/marius-agenda/assets"));
    assert!(manifest.contains("usr/share/applications/marius-agenda.desktop"));
    assert!(manifest.contains("depends = \"$auto\""));
    assert!(manifest.contains("marius-agenda.xml"));
    assert!(manifest.contains("maintainer-scripts"));
    assert!(!manifest.contains("chromium"));
}

#[test]
fn desktop_entry_uses_packaged_icon() {
    let root = repo_root();
    let desktop =
        std::fs::read_to_string(root.join("apps/marius-agenda/marius-agenda.desktop")).expect("desktop");
    assert!(desktop.contains("Icon=marius-agenda"));
    assert!(desktop.contains("Exec=marius-agenda --gui %f"));
    assert!(desktop.contains("application/x-marius-project"));
}

#[test]
fn packaging_inputs_exist_on_disk() {
    let root = repo_root();
    assert!(root.join("assets").is_dir());
    assert!(root.join("fonts").is_dir());
    assert!(root.join("scripts/package-deb.sh").is_file());
    assert!(root.join("scripts/package-snap.sh").is_file());
    assert!(root.join("snap/snapcraft.yaml").is_file());
    let snap_yaml = std::fs::read_to_string(root.join("snap/snapcraft.yaml")).expect("snapcraft.yaml");
    assert!(snap_yaml.contains("icon: packaging/icons/marius-agenda-256.png"));
    assert!(root.join("snap/gui/marius-agenda.desktop").is_file());
    let snap_desktop =
        std::fs::read_to_string(root.join("snap/gui/marius-agenda.desktop")).expect("snap desktop");
    assert!(snap_desktop.contains("Icon=${SNAP}/meta/gui/marius-agenda.png"));
    assert!(root.join("apps/marius-agenda/marius-agenda.desktop").is_file());
    assert!(root.join("packaging/icons/marius-agenda-128.png").is_file());
    assert!(root.join("packaging/icons/marius-agenda-magic-source.png").is_file());
}
