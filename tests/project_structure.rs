use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn deployment_and_documentation_files_are_present() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "publish.ps1",
        "game_page.json",
        "catalog_thumbnail.png",
        "README.md",
        "AGENTS.md",
        "CODE_STANDARDS.md",
        "GAME_DEVELOPMENT_GUIDE.md",
        "MACROQUAD_TOOLKIT.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "required project file is missing: {relative}"
        );
    }

    let page: Value = serde_json::from_str(
        &fs::read_to_string(root.join("game_page.json")).expect("game_page.json must be readable"),
    )
    .expect("game_page.json must be valid JSON");
    assert_eq!(page["title"], "Tiny Necromancer");
    assert_eq!(page["wasm"], "tiny_necromancer");
    assert_eq!(page["layout"], "viewport");
}
