//! Test observable schema composition and managed output without a listener or database.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::SystemTime,
};

use stackpress_idea_parser::{AttributeValue, SchemaConfig, TypeConfig};

/// Select the checked-in app schema independently of the process working directory.
fn input() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema.idea")
}

/// Allocate canonical fixture paths accepted by the existing managed writer on macOS.
fn fixture() -> tempfile::TempDir {
    tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
}

/// Locate a required source declaration with a useful failure rather than defaulting it.
fn model<'a>(schema: &'a SchemaConfig, name: &str) -> &'a TypeConfig {
    schema.model.get(name).expect(name)
}

/// Insert one test field after a blog model's standalone opening brace, retaining its attributes.
fn with_blog_field(source: &str, name: &str, field: &str) -> String {
    let header = format!("model {name} \n");
    assert_eq!(source.matches(&header).count(), 1);
    let start = source.find(&header).unwrap();
    let opening = start + source[start..].find("\n{\n").unwrap() + 3;
    format!("{}  {field}\n{}", &source[..opening], &source[opening..])
}

/// Capture bytes and modification times, including the ownership marker, to prove no-write checks.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    (
                        fs::read(&path).unwrap(),
                        fs::metadata(path).unwrap().modified().unwrap(),
                    ),
                );
            }
        }
    }
    files
}

/// Execute the real finite CLI; the outer recorded test runner supplies its process deadline.
fn cli(output: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_blog-tool"))
        .args(["generate", "--output"])
        .arg(output)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn imports_all_eight_models_and_preserves_blog_enum_and_relations() {
    let graph = blog_tool::compose(&input()).unwrap();
    let names: BTreeSet<_> = graph.model.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        BTreeSet::from([
            "Application",
            "Article",
            "Auth",
            "Catalog",
            "Category",
            "Comment",
            "Profile",
            "Session",
        ])
    );
    assert!(graph.r#use.is_empty(), "all package imports are composed");
    let status = graph.r#enum.get("PublishStatus").unwrap();
    let variants: Vec<_> = status
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str().unwrap()))
        .collect();
    assert_eq!(
        variants,
        [
            ("DRAFT", "DRAFT"),
            ("REVIEW", "REVIEW"),
            ("PUBLISHED", "PUBLISHED"),
            ("ARCHIVED", "ARCHIVED")
        ]
    );

    // G52 removes only the dangling Category owner; imported Profile fields remain intact.
    let category = model(&graph, "Category");
    assert!(
        !category
            .columns
            .iter()
            .any(|f| f.name == "profile" || f.name == "profileId")
    );
    let profile = model(&graph, "Profile");
    for (name, target) in [
        ("articles", "Article"),
        ("comments", "Comment"),
        ("auth", "Auth"),
    ] {
        let field = profile.columns.iter().find(|f| f.name == name).unwrap();
        assert!(field.multiple);
        assert_eq!(field.column_type, target);
    }
    let article = model(&graph, "Article");
    let owner = article
        .columns
        .iter()
        .find(|f| f.name == "profile")
        .unwrap();
    let AttributeValue::Array(args) = &owner.attributes["relation"] else {
        panic!("relation arguments");
    };
    assert_eq!(args[0]["local"].as_str(), Some("profileId"));
    assert_eq!(args[0]["foreign"].as_str(), Some("id"));
    let catalog = model(&graph, "Catalog");
    let keys: Vec<_> = catalog
        .columns
        .iter()
        .filter(|f| f.attributes.contains_key("id"))
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(keys, ["categoryId", "articleId"]);
    let AttributeValue::Array(paths) = &catalog.attributes["query"] else {
        panic!("query arguments");
    };
    assert!(
        paths
            .iter()
            .any(|q| q.as_str() == Some("article.profile.*"))
    );
    let comment = model(&graph, "Comment");
    assert_eq!(
        comment
            .columns
            .iter()
            .filter(|f| f.attributes.contains_key("relation"))
            .count(),
        2
    );
}

#[test]
fn generation_is_repeatable_and_check_preserves_bytes_and_mtimes() {
    let fixture = fixture();
    let output = fixture.path().join("generated");
    let missing = blog_tool::generate(&input(), &output, true).unwrap();
    assert!(!missing.is_current());
    assert!(!output.exists(), "check must not publish a missing root");
    blog_tool::generate(&input(), &output, false).unwrap();
    let original = snapshot(&output);
    assert!(original.contains_key(Path::new("source/article.rs")));
    assert!(original.contains_key(Path::new("stores/catalog.rs")));
    for _ in 0..2 {
        assert!(
            blog_tool::generate(&input(), &output, true)
                .unwrap()
                .is_current()
        );
        assert_eq!(snapshot(&output), original);
    }
    assert!(
        blog_tool::generate(&input(), &output, false)
            .unwrap()
            .is_current()
    );
    assert_eq!(
        snapshot(&output),
        original,
        "identical install must also be a no-op"
    );
}

#[test]
fn check_rejects_manual_drift_and_deleted_output_without_repair() {
    let fixture = fixture();
    let output = fixture.path().join("generated");
    blog_tool::generate(&input(), &output, false).unwrap();
    let path = output.join("source/article.rs");
    let original = fs::read(&path).unwrap();
    let mut edited = original.clone();
    edited.extend_from_slice(b"\n// manual drift negative control\n");
    fs::write(&path, &edited).unwrap();
    let prior = snapshot(&output);
    let error = blog_tool::generate(&input(), &output, true).unwrap_err();
    assert!(
        error.to_string().contains("manually edited generated file"),
        "{error}"
    );
    assert_eq!(snapshot(&output), prior);

    // Restore only the test-owned bytes, then test missing output independently of manual drift.
    fs::write(&path, original).unwrap();
    fs::remove_file(&path).unwrap();
    let prior = snapshot(&output);
    assert!(!cli(&output, &["--check"]).status.success());
    assert!(
        !path.exists(),
        "check must not recreate a deleted generated file"
    );
    assert_eq!(snapshot(&output), prior);
}

#[test]
fn changed_schema_is_stale_and_invalid_input_cannot_replace_output() {
    let fixture = fixture();
    let output = fixture.path().join("generated");
    blog_tool::generate(&input(), &output, false).unwrap();
    let prior = snapshot(&output);
    let changed = fixture.path().join("schema.idea");
    let source = fs::read_to_string(input()).unwrap();
    let source = with_blog_field(&source, "Article", "subtitle String?");
    fs::write(&changed, source).unwrap();
    assert!(
        !blog_tool::generate(&changed, &output, true)
            .unwrap()
            .is_current()
    );
    assert_eq!(snapshot(&output), prior);

    // G52 is an app-schema correction; the framework still refuses the original dangling edge.
    let source = fs::read_to_string(input()).unwrap();
    let source = with_blog_field(
        &source,
        "Category",
        "profile Profile @relation({ local \"profileId\" foreign \"id\" })",
    );
    fs::write(&changed, source).unwrap();
    let error = blog_tool::generate(&changed, &output, false).unwrap_err();
    assert!(
        format!("{error:?}").contains("unknown local relation field"),
        "{error:?}"
    );
    assert_eq!(snapshot(&output), prior);
}

#[test]
fn cli_generates_checks_and_reports_invalid_arguments() {
    let fixture = fixture();
    let output = fixture.path().join("generated");
    let installed = cli(&output, &[]);
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    assert!(cli(&output, &["--check"]).status.success());
    let prior = snapshot(&output);
    for args in [&["--input"][..], &["--output"][..], &["--unknown"][..]] {
        assert!(!cli(&output, args).status.success());
        assert_eq!(snapshot(&output), prior);
    }
    let usage = Command::new(env!("CARGO_BIN_EXE_blog-tool"))
        .output()
        .unwrap();
    assert!(!usage.status.success());
}

#[test]
fn full_application_emits_every_selected_role_and_checks_without_writes() {
    let fixture = fixture();
    let output = fixture.path().join("generated");
    blog_tool::generate_application(&input(), &output, false).unwrap();
    let prior = snapshot(&output);
    for path in [
        "source/article.rs",
        "stores/family.rs",
        "ui/article.rs",
        "client/framework.rs",
    ] {
        assert!(
            prior.contains_key(Path::new(path)),
            "missing application partition: {path}"
        );
    }
    assert!(
        blog_tool::generate_application(&input(), &output, true)
            .unwrap()
            .is_current()
    );
    assert_eq!(snapshot(&output), prior);

    // Metadata support and a compiled renderer are separate capabilities. Keep all toolbar props.
    let config = blog_tool::compose(&input()).unwrap();
    let graph = stackpress_framework_schema::Schema::normalize_source(
        &config,
        stackpress_framework_schema::codegen::NormalizeOptions::default(),
        &[],
    )
    .unwrap();
    let definitions = stackpress_framework_admin::codegen::source_definitions().unwrap();
    let ui = graph.project_source_ui(&definitions).unwrap();
    let mut unavailable = Vec::new();
    for model in &ui.models {
        for field in &model.fields {
            for role in &field.roles {
                if !role.available {
                    unavailable.push(format!("{}.{}:{:?}", model.id, field.wire_name, role.role));
                }
            }
        }
    }
    assert!(
        unavailable.is_empty(),
        "unavailable blog roles: {unavailable:?}"
    );
    let article = ui
        .models
        .iter()
        .find(|model| model.id == "Article")
        .unwrap();
    let contents = article
        .fields
        .iter()
        .find(|field| field.wire_name == "contents")
        .unwrap();
    let editor = contents
        .roles
        .iter()
        .find(|role| role.role == stackpress_framework_schema::Role::Field)
        .unwrap();
    assert_eq!(editor.props.len(), 14);
    assert!(editor.available);
    // Keep the exact rich editor and raw HTML adapters; a textarea/text replacement must fail.
    let emitted = fs::read_to_string(output.join("ui/article.rs")).unwrap();
    assert!(emitted.contains("source::roles::EditorField"));
    assert!(emitted.contains("source::roles::HtmlView"));
}

#[cfg(feature = "native-proof")]
#[test]
fn emitted_native_family_compiles_and_keeps_all_models_and_defaults() {
    use blog_tool::generated::{source, stores};
    use stackpress_framework_schema::source::SourceSchema;
    use stackpress_framework_sql::source::ApiModels;

    /// The same generated Profile type supplies both the session and API family contracts.
    fn family<M: ApiModels>() {}
    family::<stores::family::Models>();
    // The model named Catalog owns `catalog`; the generator moves its infrastructure catalog.
    let catalog = source::_catalog::describe();
    assert_eq!(catalog.models.len(), 8);
    assert_eq!(catalog.enums.len(), 1);
    let _: Option<stores::category::CategoryStore> = None;
    let _: Option<stores::catalog::CatalogStore> = None;
    let _: Option<stores::article::ArticleActions> = None;
    let _: Option<stores::comment::CommentActions> = None;
    let article = source::article::ArticleSchema::new("explicit-seed".into(), 0);
    let defaults = article.defaults_at(0).unwrap();
    assert_eq!(defaults["status"], "DRAFT");
    assert_eq!(defaults["active"], true);
    let profile = source::profile::ProfileSchema::describe();
    assert!(profile.fields.iter().any(|f| f.name == "articles"));
}
