use super::*;
use std::fs;
use tempfile::tempdir;

fn create_manifest(toml_content: &str) -> (Manifest, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("Cargo.toml");

    fs::write(&path, toml_content).unwrap();

    let manifest = Manifest::from_toml_path(&path).unwrap();

    (manifest, dir)
}

fn workspace() -> TablePath {
    TablePath::new().push("workspace")
}

fn workspace_dependencies() -> TablePath {
    TablePath::new().push("workspace").push("dependencies")
}

fn dependencies() -> TablePath {
    TablePath::new().push("dependencies")
}

#[test]
fn test_wrong_table() {
    let toml_content = r#"[workspace]
    members = ["app", "lib", "other"]

    [dependencies]
    test = "1.0"
    "#;
    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("manifest:\n{}", manifest.data());

    let table = TablePath::new().push("dependencies").push("test");
    let value = Item::Value(Value::from("2.0"));
    let result = manifest.add_key(&table, "serde", &value, &InheritMode::None);

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    println!("Result:\n{:?}", result);

    assert!(result.is_err());
}

#[test]
fn get_array() {
    let toml_content = r#"[workspace]
    members = [
    "app", #this decorate 1
    "lib",
    "other" #this decorate test
    ] #this decorate 2
    "#;
    let (manifest, _dir) = create_manifest(toml_content);

    println!("manifest:\n{}", manifest.data());

    let table = workspace();
    let array = manifest.get_array(&table, "members").unwrap();

    println!("======================\n======================\n");

    println!("Getter:\n{}", array);

    assert_eq!(array.len(), 3);
    assert_eq!(array.get(0).and_then(Value::as_str), Some("app"));
    assert_eq!(array.get(1).and_then(Value::as_str), Some("lib"));
}

#[test]
fn add_value_to_array_with_key_non_exist() {
    let toml_content = r#"[workspace]"#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace();
    let value = Item::Value(Value::from("app"));

    manifest
        .add_value_to_array(&table, "members", &value)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let array = manifest.get_array(&table, "members").unwrap();

    assert_eq!(array.get(0).and_then(Value::as_str), Some("app"));
}

#[test]
fn add_value_to_array_with_key_exist_horizontal() {
    let toml_content = r#"[workspace]
    members = ["app", "lib"] #this decorate 2"#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace();
    let value = Item::Value(Value::from("other"));

    manifest
        .add_value_to_array(&table, "members", &value)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let array = manifest.get_array(&table, "members").unwrap();

    assert_eq!(array.len(), 3);
    assert_eq!(array.get(0).and_then(Value::as_str), Some("app"));
    assert_eq!(array.get(1).and_then(Value::as_str), Some("lib"));
    assert_eq!(array.get(2).and_then(Value::as_str), Some("other"));
}

#[test]
fn add_value_to_array_with_key_exist_vertical() {
    let toml_content = r#"[workspace]
    members = [
    "app", #this decorate 1
    "lib" #this decorate test
    ] #this decorate 2
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace();
    let value = Item::Value(Value::from("other"));

    manifest
        .add_value_to_array(&table, "members", &value)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let array = manifest.get_array(&table, "members").unwrap();

    assert_eq!(array.len(), 3);
    assert_eq!(array.get(0).and_then(Value::as_str), Some("app"));
    assert_eq!(array.get(1).and_then(Value::as_str), Some("lib"));
    assert_eq!(array.get(2).and_then(Value::as_str), Some("other"));
}

#[test]
fn update_value_of_array_replace_old() {
    let toml_content = r#"[workspace]
    members = [
    "app", #this decorate 1
    "lib",
    "other", #this decorate test
    ] #this decorate 2
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace();
    let old_value = Item::Value(Value::from("other"));
    let new_value = Item::Value(Value::from("test"));

    manifest
        .update_value_of_array(&table, "members", Some(&old_value), &new_value)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let array = manifest.get_array(&table, "members").unwrap();

    assert_eq!(array.len(), 3);
    assert_eq!(array.get(2).and_then(Value::as_str), Some("test"));
    assert_eq!(array.get(0).and_then(Value::as_str), Some("app"));

    let output = manifest.data().to_string();

    assert!(output.contains("\"test\""));
    assert!(output.contains("] #this decorate 2"));
}

#[test]
fn update_value_of_array_append_new() {
    let toml_content = r#"[workspace]
    members = [
    "app", #this decorate 1
    "lib",
    "other", #this decorate test
    ] #this decorate 2
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace();
    let new_value = Item::Value(Value::from("test"));

    manifest
        .update_value_of_array(&table, "members", None, &new_value)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let array = manifest.get_array(&table, "members").unwrap();

    assert_eq!(array.len(), 4);
    assert_eq!(array.get(2).and_then(Value::as_str), Some("other"));

    let output = manifest.data().to_string();

    assert!(output.contains("\"test\""));
    assert!(output.contains("] #this decorate 2"));
}

#[test]
fn remove_value_from_array() {
    let toml_content = r#"[workspace]
    members = [
    "app", #this decorate 1
    "lib",
    "other", #this decorate test
    ] #this decorate 2
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace();
    let value = Item::Value(Value::from("app"));

    manifest
        .remove_value_from_array(&table, "members", &value)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let array = manifest.get_array(&table, "members").unwrap();

    assert_eq!(array.len(), 2);
}

#[test]
fn get_key() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (manifest, _dir) = create_manifest(toml_content);

    println!("MANIFEST:\n{}", manifest.data());

    let table = workspace_dependencies();
    let item = manifest.get_key(&table, "anyhow").unwrap();

    println!("======================\n======================\n");

    println!("GET:\n{}", item);

    assert_eq!(item.as_value().and_then(Value::as_str), Some("1.0"));
}

#[test]
fn add_key_non_inherit_added() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();
    let value = Item::Value(Value::from("2.0"));

    manifest
        .add_key(&table, "serde", &value, &InheritMode::None)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "serde").unwrap();

    assert_eq!(item.as_value().and_then(Value::as_str), Some("2.0"));
}

#[test]
fn add_key_non_inherit_not_replace_existed() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();
    let value = Item::Value(Value::from("2.0"));

    manifest
        .add_key(&table, "anyhow", &value, &InheritMode::None)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "anyhow").unwrap();

    assert_eq!(item.as_value().and_then(Value::as_str), Some("1.0"));
}

#[test]
fn add_key_inherit_full() {
    let toml_content = r#"
    [dependencies]
    toml_edit = "0.25" #this is decorate test3
    anyhow.workspace = true #this is decorate 3
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = dependencies();

    let mut source_table = Table::new();
    source_table.insert("version", Value::from("1.0").into());

    // let value = into_inline_table_item(Item::Table(source_table)).unwrap();
    let value = Item::Table(source_table);

    manifest
        .add_key(&table, "serde", &value, &InheritMode::Full)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "serde").unwrap();

    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn add_key_inherit_partial() {
    let toml_content = r#"
    [dependencies]
    toml_edit = "0.25" #this is decorate test3
    anyhow.workspace = true #this is decorate 3
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = dependencies();

    let mut source_table = Table::new();
    source_table.insert("version", Value::from("1.0").into());

    // let value = into_inline_table_item(Item::Table(source_table)).unwrap();
    let value = Item::Table(source_table);

    manifest
        .add_key(&table, "serde", &value, &InheritMode::Partial)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "serde").unwrap();

    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn update_key_non_inherit() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();
    let value = Item::Value(Value::from("2.0"));

    manifest
        .update_key(&table, "anyhow", &value, &InheritMode::None)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "anyhow").unwrap();

    assert_eq!(item.as_value().and_then(Value::as_str), Some("2.0"));
}

#[test]
fn update_key_inherit_full() {
    let toml_content = r#"
    [dependencies]
    toml_edit = "0.25" #this is decorate test3
    anyhow.workspace = true #this is decorate 3
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = dependencies();

    let mut source_table = Table::new();
    source_table.insert("optional", true.into());

    let value = Item::Table(source_table);
    // let value = into_inline_table_item(Item::Table(source_table)).unwrap();
    // let value = "2.0".into();

    manifest
        .update_key(&table, "toml_edit", &value, &InheritMode::Full)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "toml_edit").unwrap();

    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn update_key_inherit_partial() {
    let toml_content = r#"
    [dependencies]
    toml_edit = "0.25" #this is decorate test3
    anyhow.workspace = true #this is decorate 3
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = dependencies();

    let mut source_table = Table::new();
    source_table.insert("optional", true.into());

    let value = Item::Table(source_table);
    // let value = into_inline_table_item(Item::Table(source_table)).unwrap();
    // let value = "2.0".into();

    println!("FROM VALUE SOURCE:\n{}\n", value);

    manifest
        .update_key(&table, "toml_edit", &value, &InheritMode::Partial)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_key(&table, "toml_edit").unwrap();

    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn remove_key() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    manifest.remove_key(&table, "semver").unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    assert!(manifest.get_key(&table, "semver").is_err());
}

#[test]
fn get_items_at() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (manifest, _dir) = create_manifest(toml_content);

    println!("MANIFEST:\n{}", manifest.data());

    let table_path = workspace_dependencies();
    let item = manifest.get_items_at(&table_path).unwrap();

    println!("======================\n======================\n");

    println!("GET:\n{}", item);

    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("anyhow")
            .and_then(Item::as_value)
            .and_then(Value::as_str),
        Some("1.0")
    );
}

#[test]
fn add_items_at_non_inherit() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    let value = Item::Table({
        let mut table = Table::new();
        table.insert("serde", "1.0".into());
        table.insert("anyhow", "2.0".into());
        table
    });

    manifest
        .add_items_at(&table, &value, &InheritMode::None)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_items_at(&table).unwrap();
    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("anyhow")
            .and_then(Item::as_value)
            .and_then(Value::as_str),
        Some("1.0")
    );

    assert_eq!(
        table
            .get("serde")
            .and_then(Item::as_value)
            .and_then(Value::as_str),
        Some("1.0")
    );
}

#[test]
fn add_items_at_inherit_full() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    let value = Item::Table({
        let mut table = Table::new();
        table.insert("serde", "1.0".into());
        // table.insert(
        //     "serde",
        //     Item::Value(Value::InlineTable({
        //         let mut inline_table = InlineTable::new();
        //         inline_table.insert("version", "1.0".into());
        //         inline_table.insert("optional", true.into());
        //         inline_table
        //     })),
        // );
        // table.insert(
        //     "serde",
        //     Item::Table({
        //         let mut table = Table::new();
        //         table.insert("version", "1.0".into());
        //         table.insert("optional", true.into());
        //         table
        //     }),
        // );
        table.insert("anyhow", "2.0".into());
        table
    });

    manifest
        .add_items_at(&table, &value, &InheritMode::Full)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_items_at(&table).unwrap();
    let table = item.as_table_like().unwrap();

    let serde = table.get("serde").unwrap().as_table_like().unwrap();
    assert_eq!(
        serde
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );

    let anyhow = table.get("anyhow").unwrap().as_value();
    assert_eq!(anyhow.and_then(Value::as_str), Some("1.0"));
}

#[test]
fn add_items_at_inherit_partial() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    let value = Item::Table({
        let mut table = Table::new();
        // table.insert("serde", "1.0".into());
        table.insert(
            "serde",
            Item::Value(Value::InlineTable({
                let mut inline_table = InlineTable::new();
                inline_table.insert("version", "1.0".into());
                inline_table.insert("optional", true.into());
                inline_table
            })),
        );
        // table.insert(
        //     "serde",
        //     Item::Table({
        //         let mut table = Table::new();
        //         table.insert("version", "1.0".into());
        //         table.insert("optional", true.into());
        //         table
        //     }),
        // );
        table.insert("anyhow", "2.0".into());
        table
    });

    manifest
        .add_items_at(&table, &value, &InheritMode::Partial)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_items_at(&table).unwrap();
    let table = item.as_table_like().unwrap();

    let serde = table.get("serde").unwrap().as_table_like().unwrap();
    assert_eq!(
        serde
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );

    let anyhow = table.get("anyhow").unwrap().as_value();
    assert_eq!(anyhow.and_then(Value::as_str), Some("1.0"));
}

#[test]
fn update_items_at_non_inherit() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    let value = Item::Table({
        let mut table = Table::new();
        table.insert("serde", "1.0".into());
        table.insert("anyhow", "2.0".into());
        table
    });

    manifest
        .update_items_at(&table, &value, &InheritMode::None)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_items_at(&table).unwrap();
    let table = item.as_table_like().unwrap();

    assert_eq!(
        table
            .get("anyhow")
            .and_then(Item::as_value)
            .and_then(Value::as_str),
        Some("2.0")
    );

    assert_eq!(
        table
            .get("serde")
            .and_then(Item::as_value)
            .and_then(Value::as_str),
        Some("1.0")
    );
}

#[test]
fn update_items_at_inherit_full() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    let value = Item::Table({
        let mut table = Table::new();
        // table.insert("serde", "1.0".into());
        // table.insert("anyhow", "2.0".into());

        // =================
        // table.insert(
        //     "serde",
        //     Item::Value(Value::InlineTable({
        //         let mut inline_table = InlineTable::new();
        //         inline_table.insert("version", "1.0".into());
        //         inline_table.insert("optional", true.into());
        //         inline_table
        //     })),
        // );
        // table.insert(
        //     "anyhow",
        //     Item::Value(Value::InlineTable({
        //         let mut inline_table = InlineTable::new();
        //         inline_table.insert("version", "2.0".into());
        //         inline_table.insert("optional", true.into());
        //         inline_table
        //     })),
        // );

        // =================
        table.insert(
            "serde",
            Item::Table({
                let mut table = Table::new();
                table.insert("version", "1.0".into());
                table.insert("optional", true.into());
                table
            }),
        );
        table.insert(
            "anyhow",
            Item::Table({
                let mut table = Table::new();
                table.insert("version", "2.0".into());
                table.insert("optional", true.into());
                table
            }),
        );
        table
    });

    manifest
        .update_items_at(&table, &value, &InheritMode::Full)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_items_at(&table).unwrap();
    let table = item.as_table_like().unwrap();

    let serde = table.get("serde").unwrap().as_table_like().unwrap();
    assert_eq!(
        serde
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn update_items_at_inherit_partial() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    let value = Item::Table({
        let mut table = Table::new();
        // table.insert("serde", "1.0".into());
        // table.insert("anyhow", "2.0".into());

        // =================
        // table.insert(
        //     "serde",
        //     Item::Value(Value::InlineTable({
        //         let mut inline_table = InlineTable::new();
        //         inline_table.insert("version", "1.0".into());
        //         inline_table.insert("optional", true.into());
        //         inline_table
        //     })),
        // );
        // table.insert(
        //     "anyhow",
        //     Item::Value(Value::InlineTable({
        //         let mut inline_table = InlineTable::new();
        //         inline_table.insert("version", "2.0".into());
        //         inline_table.insert("optional", true.into());
        //         inline_table
        //     })),
        // );

        // =================
        table.insert(
            "serde",
            Item::Table({
                let mut table = Table::new();
                table.insert("version", "1.0".into());
                table.insert("optional", true.into());
                table
            }),
        );
        table.insert(
            "anyhow",
            Item::Table({
                let mut table = Table::new();
                table.insert("version", "2.0".into());
                table.insert("optional", true.into());
                table
            }),
        );
        table
    });

    manifest
        .update_items_at(&table, &value, &InheritMode::Partial)
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    let item = manifest.get_items_at(&table).unwrap();
    let table = item.as_table_like().unwrap();

    let serde = table.get("serde").unwrap().as_table_like().unwrap();
    assert_eq!(
        serde
            .get("workspace")
            .and_then(Item::as_value)
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn remove_items_at_non_target() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    manifest.remove_items_at(&table, None).unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    assert!(
        manifest
            .get_items_at(&table)
            .unwrap()
            .as_table_like()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn remove_items_at_with_target() {
    let toml_content = r#"
    [workspace.dependencies]
    anyhow = "1.0" #this is decorate test2
    semver = { version = "1.0", features = ["serde"] }
    clap = { version = "4.6", features = ["derive", "wrap_help"] }
    "#;

    let (mut manifest, _dir) = create_manifest(toml_content);

    println!("BEFORE:\n{}", manifest.data());

    let table = workspace_dependencies();

    manifest
        .remove_items_at(&table, Some(&["semver".into(), "anyhow".into()]))
        .unwrap();

    println!("======================\n======================\n");

    println!("AFTER:\n{}", manifest.data());

    assert!(manifest.get_key(&table, "semver").is_err());
    assert!(manifest.get_key(&table, "anyhow").is_err());
}
