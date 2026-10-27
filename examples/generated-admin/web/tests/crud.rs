//! Real generated operations through compiled routes, buffered HTTP and owned SQLite files.
use serde_json::{Value, json};
use stackpress_framework_admin::search::encode;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Reap finite request processes even when an assertion or timeout interrupts the parent.
struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Each request has its own deadline and bounded file output, never a listening socket.
fn execute(mut input: Value) -> Value {
    input
        .as_object_mut()
        .unwrap()
        .entry("actor")
        .or_insert(json!("editor"));
    input
        .as_object_mut()
        .unwrap()
        .entry("mount")
        .or_insert(json!("/"));
    let mut source = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    write!(source, "{input}").unwrap();
    source.flush().unwrap();
    let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let mut child = Guard(
        Command::new(env!("CARGO_BIN_EXE_generated-admin-web"))
            .arg("--request")
            .stdin(Stdio::from(std::fs::File::open(source.path()).unwrap()))
            .stdout(Stdio::from(log.reopen().unwrap()))
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "finite CRUD request expired"
        );
        assert!(
            log.as_file().metadata().unwrap().len() < 2_097_152,
            "finite output budget"
        );
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "{input}");
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    serde_json::from_slice(&std::fs::read(log.path()).unwrap()).unwrap()
}

/// One explicitly owned fixture keeps its database alive across finite request processes.
struct Fixture {
    root: tempfile::TempDir,
    database: PathBuf,
    mount: &'static str,
}
impl Fixture {
    /// Generated table definitions and selected handlers are the only schema/data layer.
    fn new(mount: &'static str) -> Self {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let database = root.path().join("admin.sqlite");
        assert_eq!(
            execute(json!({"database":database,"mount":mount,"initialize":true}))["status"],
            200
        );
        Self {
            root,
            database,
            mount,
        }
    }
    /// Exercise the real buffered HTTP adapter with its native method, URL and headers.
    fn request(
        &self,
        path: &str,
        body: Option<Value>,
        actor: &str,
        csrf: bool,
        fault: bool,
    ) -> Value {
        let headers = if csrf {
            json!({"origin":"https://admin.invalid","x-admin-csrf":"finite-session-csrf"})
        } else {
            json!({})
        };
        let result = execute(
            json!({"database":self.database,"mount":self.mount,"path":format!("{}{path}",self.mount),"method":if body.is_some(){"POST"}else{"GET"},"body":body,"actor":actor,"headers":headers,"fault":fault}),
        );
        assert!(self.root.path().exists());
        assert!(!result["body"].as_str().unwrap().contains("ADMIN_PRIVATE"));
        result
    }
    /// Submit a strict generated envelope and assert its actual native HTTP status.
    fn api(&self, module: &str, action: &str, body: Value, status: u16) -> Value {
        let result = self.request(
            &format!("api/{module}/{action}"),
            Some(body),
            "editor",
            true,
            false,
        );
        assert_eq!(result["status"], status, "{result}");
        serde_json::from_str(result["body"].as_str().unwrap()).unwrap()
    }
    /// Admit no raw fixture SQL or hidden model API.
    fn create(&self, module: &str, input: Value) {
        assert_eq!(
            self.api(module, "create", json!({"input":input}), 200)["code"],
            200
        );
    }
}

/// All three models execute create/search/detail/update/copy and exact public errors.
#[test]
fn generated_core_crud_policy_and_unknown_outcome() {
    for mount in ["/", "/admin/"] {
        let fixture = Fixture::new(mount);
        fixture.create("profile", json!({"id":"p","name":"Parent"}));
        fixture.create("project", json!({"id":"j","ownerId":"p","title":"Project"}));
        fixture.create(
            "task",
            json!({"id":"t","projectId":"j","title":"Task","done":false,"note":null,"priority":0}),
        );
        for (module, id, field) in [
            ("profile", "p", "name"),
            ("project", "j", "title"),
            ("task", "t", "title"),
        ] {
            let query = json!({"total":true});
            let page = fixture.api(module, "search", query, 200);
            assert_eq!(page["results"]["total"], 1);
            assert_eq!(page["results"]["rows"][0]["row"]["id"], id);
            for action in ["get", "detail"] {
                assert_eq!(
                    fixture.api(module, action, json!({"key":{"id":id}}), 200)["results"]["id"],
                    id
                );
            }
            let mut patch = serde_json::Map::new();
            patch.insert(field.into(), json!("Updated </script> 🦀"));
            assert_eq!(
                fixture.api(
                    module,
                    "update",
                    json!({"key":{"id":id},"patch":patch}),
                    200
                )["results"][field],
                "Updated </script> 🦀"
            );
            let key = encode(&json!({"id":id}).to_string());
            for action in ["detail", "update", "copy"] {
                let page = fixture.request(
                    &format!("{module}/{action}?key={key}"),
                    None,
                    "editor",
                    false,
                    false,
                );
                assert_eq!(page["status"], 200, "{page}");
                let html = page["body"].as_str().unwrap();
                assert!(!html.contains("Unable to render"));
                assert!(!html.contains("<script> 🦀"));
                if action == "copy" {
                    let seed = html
                        .split("<script id=\"admin-seed\" type=\"application/json\">")
                        .nth(1)
                        .unwrap()
                        .split("</script>")
                        .next()
                        .unwrap();
                    let snapshot: Value = serde_json::from_str(seed).unwrap();
                    assert_eq!(snapshot["key"], json!({}));
                }
            }
            let denied = fixture.request(
                &format!("api/{module}/create"),
                Some(json!({"not":"a DTO"})),
                "reader",
                true,
                false,
            );
            assert_eq!(denied["status"], 403);
            let denied = fixture.request(
                &format!("{module}/update?key=invalid"),
                None,
                "reader",
                false,
                false,
            );
            assert_eq!(denied["status"], 403, "policy precedes bad URL decoding");
            fixture.api(module, "get", json!({"key":{"id":"absent"}}), 404);
            fixture.api(module, "update", json!({"key":{"id":id},"patch":{}}), 400);
        }
        assert_eq!(
            fixture.request(
                "api/profile/create",
                Some(json!({"input":{"id":"csrf","name":"No"}})),
                "editor",
                false,
                false
            )["status"],
            403
        );
        let invalid = fixture.api(
            "profile",
            "create",
            json!({"input":{"id":"invalid","name":""}}),
            400,
        );
        assert_eq!(invalid["errors"]["fields"][0]["path"], json!(["name"]));
        fixture.api(
            "profile",
            "create",
            json!({"input":{"id":"private","name":"x","hostOnly":"no"}}),
            400,
        );
        let unknown = fixture.request(
            "api/profile/create",
            Some(json!({"input":{"id":"uncertain","name":"Uncertain"}})),
            "editor",
            true,
            true,
        );
        assert_eq!(unknown["status"], 500);
        let body: Value = serde_json::from_str(unknown["body"].as_str().unwrap()).unwrap();
        assert_eq!(body["errors"]["outcome"], "unknown");
        assert_eq!(
            fixture.api("profile", "get", json!({"key":{"id":"uncertain"}}), 200)["results"]["name"],
            "Uncertain",
            "effect actually committed before its acknowledgement was lost"
        );
        fixture.api("profile", "remove", json!({"key":{"id":"p"}}), 200);
        fixture.api("profile", "get", json!({"key":{"id":"p"}}), 404);
        assert_eq!(
            fixture.request(
                "profile/restore?key=%7B%22id%22%3A%22p%22%7D",
                None,
                "editor",
                false,
                false
            )["status"],
            200
        );
        fixture.api("profile", "restore", json!({"key":{"id":"p"}}), 200);
        fixture.api("profile", "get", json!({"key":{"id":"p"}}), 200);
        assert_eq!(
            fixture.request("project/remove", None, "editor", false, false)["status"],
            404,
            "nonactive models expose no removal page"
        );
        assert_eq!(
            fixture.request(
                "api/project/restore",
                Some(json!({"key":{"id":"j"}})),
                "editor",
                true,
                false
            )["status"],
            404
        );
    }
}

/// Scoped related search/create cannot escape its selected parent, including tampered envelopes.
#[test]
fn related_scope_and_typed_search_urls() {
    let fixture = Fixture::new("/admin/");
    fixture.create("profile", json!({"id":"p","name":"Parent"}));
    for id in ["j1", "j2"] {
        fixture.create("project", json!({"id":id,"ownerId":"p","title":id}));
    }
    for (id, parent, done) in [("t1", "j1", false), ("t2", "j2", false), ("t3", "j1", true)] {
        fixture.create(
            "task",
            json!({"id":id,"projectId":parent,"title":id,"done":done}),
        );
    }
    let scope =
        generated_admin_web::host::related("project", json!({"id":"j1"}), "projectId", json!("j1"));
    let page = fixture.request(
        &format!("task/search?{scope}"),
        None,
        "editor",
        false,
        false,
    );
    assert_eq!(page["status"], 200);
    let html = page["body"].as_str().unwrap();
    assert!(html.contains("t1"));
    assert!(!html.contains("t2"));
    let query = json!({"q":null,"filters":[{"field":"done","predicate":{"op":"eq","value":false}},{"field":"priority","predicate":{"op":"eq","value":0}}],"sort":[],"includes":[],"skip":0,"take":1,"total":true});
    let page = fixture.request(
        &format!("api/task/search?{scope}"),
        Some(query),
        "editor",
        true,
        false,
    );
    assert_eq!(page["status"], 200);
    let page: Value = serde_json::from_str(page["body"].as_str().unwrap()).unwrap();
    assert_eq!(page["results"]["total"], 1);
    assert_eq!(page["results"]["rows"][0]["row"]["id"], "t1");
    let created = fixture.request(
        &format!("api/task/create?{scope}"),
        Some(json!({"input":{"id":"related","title":"Related"}})),
        "editor",
        true,
        false,
    );
    assert_eq!(created["status"], 200);
    assert_eq!(
        fixture.api("task", "get", json!({"key":{"id":"related"}}), 200)["results"]["projectId"],
        "j1"
    );
    let escaped = fixture.request(
        &format!("api/task/create?{scope}"),
        Some(json!({"input":{"id":"escape","projectId":"j2","title":"Escape"}})),
        "editor",
        true,
        false,
    );
    assert_eq!(escaped["status"], 400);
    fixture.api("task", "get", json!({"key":{"id":"escape"}}), 404);
    for url in [
        "task/search?state=%7B%22skip%22%3A100001%7D",
        "task/search?state=%7B%22limit%22%3A0%7D",
        "task/search?state=%7B%7D&state=%7B%7D",
    ] {
        assert_eq!(
            fixture.request(url, None, "editor", false, false)["status"],
            400
        );
    }
    // A child from another parent cannot be loaded through a valid scoped detail link.
    assert_eq!(
        fixture.request(
            &format!("task/detail?{scope}&key={}", encode(r#"{"id":"t2"}"#)),
            None,
            "editor",
            false,
            false
        )["status"],
        404
    );
    assert_eq!(
        fixture.request(
            &format!("task/update?{scope}&key={}", encode(r#"{"id":"t1"}"#)),
            None,
            "editor",
            false,
            false
        )["status"],
        400
    );
    assert_eq!(
        fixture.request(
            "api/task/search?state=%7B%7D",
            Some(json!({})),
            "editor",
            false,
            false
        )["status"],
        400
    );
    for operation in ["batch", "delete", "upsert"] {
        assert_eq!(
            fixture.request(
                &format!("api/task/{operation}"),
                Some(json!({})),
                "editor",
                true,
                false
            )["status"],
            404
        );
    }
    assert!(Path::new(&fixture.database).exists());
}
