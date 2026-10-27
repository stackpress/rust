//! Real generated CSV preview, maintenance conflict reads, atomic batches and buffered downloads.
use serde_json::{Value, json};
use stackpress_framework_admin::{
    search::encode,
    transfer::{self, Limits},
};
use std::{
    io::Write,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn execute(input: Value) -> Value {
    let mut source = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    write!(source, "{input}").unwrap();
    source.flush().unwrap();
    let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let mut child = Guard(
        Command::new(env!("CARGO_BIN_EXE_generated-admin-web"))
            .arg("--request")
            .stdin(Stdio::from(source.reopen().unwrap()))
            .stdout(Stdio::from(log.reopen().unwrap()))
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "finite transfer deadline"
        );
        assert!(log.as_file().metadata().unwrap().len() < 8_388_608);
        if let Some(code) = child.0.try_wait().unwrap() {
            assert!(code.success(), "finite child failed");
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let value: Value = serde_json::from_slice(&std::fs::read(log.path()).unwrap()).unwrap();
    assert!(!value.to_string().contains("ADMIN_PRIVATE"));
    value
}
struct Fixture {
    root: tempfile::TempDir,
    mount: &'static str,
}
impl Fixture {
    fn new(mount: &'static str) -> Self {
        let value = Self {
            root: tempfile::tempdir_in("/private/tmp").unwrap(),
            mount,
        };
        assert_eq!(
            value.call("GET", "/", None, "editor", false, false, None, true)["status"],
            200
        );
        value
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "finite protocol fixture parameters mirror independent HTTP inputs"
    )]
    fn call(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
        actor: &str,
        csrf: bool,
        fault: bool,
        limits: Option<Limits>,
        initialize: bool,
    ) -> Value {
        let mut input = json!({"database":self.root.path().join("db.sqlite"),"mount":self.mount,"path":if initialize{"/".into()}else{format!("{}{path}",self.mount)},"method":method,"actor":actor,"initialize":initialize,"fault":fault,"body":body,"headers":if csrf{json!({"origin":"https://admin.invalid","x-admin-csrf":"finite-session-csrf"})}else{json!({})}});
        if let Some(limits) = limits {
            input["transfer_limits"] = json!(limits);
        }
        execute(input)
    }
    fn api(&self, module: &str, action: &str, body: Value, status: u16) -> Value {
        let reply = self.call(
            "POST",
            &format!("api/{module}/{action}"),
            Some(body),
            "editor",
            true,
            false,
            None,
            false,
        );
        assert_eq!(reply["status"], status, "{reply}");
        serde_json::from_str(reply["body"].as_str().unwrap()).unwrap()
    }
    fn import(
        &self,
        module: &str,
        action: &str,
        csv: &str,
        guards: bool,
        fault: bool,
        status: u16,
    ) -> Value {
        let reply = self.call(
            "POST",
            &format!("api/{module}/{action}"),
            Some(json!({"csv":csv,"decode_export_guards":guards,"confirm":action=="confirm"})),
            "editor",
            true,
            fault,
            None,
            false,
        );
        assert_eq!(reply["status"], status, "{reply}");
        serde_json::from_str(reply["body"].as_str().unwrap()).unwrap()
    }
    fn create(&self, module: &str, input: Value) {
        self.api(module, "create", json!({"input":input}), 200);
    }
}
#[test]
fn preview_is_read_only_and_confirm_is_atomic_create_only() {
    for mount in ["/", "/admin/"] {
        let f = Fixture::new(mount);
        let csv = "id,name\na,Alpha\nb,Beta";
        let preview = f.import("profile", "preview", csv, false, false, 200);
        assert_eq!(preview["results"]["rows"].as_array().unwrap().len(), 2);
        f.api("profile", "get", json!({"key":{"id":"a"}}), 404);
        let done = f.import("profile", "confirm", csv, false, false, 200);
        assert_eq!(done["results"]["imported"], 2);
        assert_eq!(
            f.api("profile", "get", json!({"key":{"id":"a"}}), 200)["results"]["name"],
            "Alpha"
        );
        let conflict = f.import("profile", "preview", csv, false, false, 200);
        assert_eq!(
            conflict["results"]["rows"][0]["errors"][0]["code"],
            "conflict"
        );
        f.api("profile", "remove", json!({"key":{"id":"a"}}), 200);
        let conflict = f.import(
            "profile",
            "preview",
            "id,name\na,Changed",
            false,
            false,
            200,
        );
        assert_eq!(
            conflict["results"]["rows"][0]["errors"][0]["code"], "conflict",
            "inactive identities still conflict"
        );
        f.import(
            "profile",
            "confirm",
            "id,name\nc,Valid\nd,",
            false,
            false,
            400,
        );
        f.api("profile", "get", json!({"key":{"id":"c"}}), 404);
        let invalid = f.import(
            "profile",
            "preview",
            "id,name\na,\nx,Okay\nx,Again",
            false,
            false,
            200,
        );
        assert!(
            invalid["results"]["rows"][0]["errors"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["code"] == "conflict")
        );
        assert_eq!(
            invalid["results"]["rows"][2]["errors"][0]["code"],
            "conflict"
        );
        // A foreign-key failure after an earlier successful INSERT rolls the exact batch back.
        f.import(
            "project",
            "confirm",
            "id,ownerId,title\nj1,b,Good\nj2,absent,Failure",
            false,
            false,
            500,
        );
        f.api("project", "get", json!({"key":{"id":"j1"}}), 404);
        let unknown = f.import(
            "profile",
            "confirm",
            "id,name\nu,Uncertain\nv,Also uncertain",
            false,
            true,
            500,
        );
        assert_eq!(unknown["errors"]["outcome"], "unknown");
        assert!(unknown.get("results").is_none());
        f.api("profile", "get", json!({"key":{"id":"u"}}), 200);
        f.api("profile", "get", json!({"key":{"id":"v"}}), 200);
    }
}
#[test]
fn limits_headers_denials_and_required_omission_refuse_without_writes() {
    let f = Fixture::new("/");
    for csv in [
        "id,id\na,b",
        "id,private\na,hidden",
        "id,name\nx,\"unterminated",
    ] {
        f.import("profile", "preview", csv, false, false, 400);
    }
    let omission = f.import(
        "profile",
        "preview",
        "id,name\n\\O,Missing",
        false,
        false,
        200,
    );
    assert!(
        !omission["results"]["rows"][0]["errors"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    f.import(
        "profile",
        "confirm",
        "id,name\n\\O,Missing",
        false,
        false,
        400,
    );
    let denied = f.call(
        "POST",
        "api/profile/preview",
        Some(json!({"csv":"\"bad","confirm":false})),
        "reader",
        true,
        false,
        None,
        false,
    );
    assert_eq!(denied["status"], 403);
    assert!(!denied["body"].as_str().unwrap().contains("CSV"));
    assert_eq!(
        f.call(
            "GET",
            "profile/import",
            None,
            "reader",
            false,
            false,
            None,
            false
        )["status"],
        403
    );
    assert_eq!(
        f.call(
            "POST",
            "api/profile/preview",
            Some(json!({"csv":"id,name\nx,X"})),
            "editor",
            false,
            false,
            None,
            false
        )["status"],
        403
    );
    assert_eq!(
        f.call(
            "POST",
            "api/profile/confirm",
            Some(json!({"csv":"id,name\nx,X"})),
            "editor",
            true,
            false,
            None,
            false
        )["status"],
        400
    );
    for limits in [
        Limits {
            rows: 1,
            ..Default::default()
        },
        Limits {
            columns: 1,
            ..Default::default()
        },
        Limits {
            cell: 2,
            ..Default::default()
        },
        Limits {
            bytes: 8,
            ..Default::default()
        },
    ] {
        assert!(
            f.call(
                "POST",
                "api/profile/preview",
                Some(json!({"csv":"id,name\na,Alpha\nb,Beta"})),
                "editor",
                true,
                false,
                Some(limits),
                false
            )["status"]
                .as_u64()
                .unwrap()
                >= 400
        );
    }
    let huge = "x".repeat(1_048_577);
    assert_eq!(
        f.call(
            "POST",
            "api/profile/preview",
            Some(json!({"csv":huge})),
            "editor",
            true,
            false,
            None,
            false
        )["status"],
        // The shared HTTP body admission rejects this before the CSV host runs.
        400
    );
    let rows = format!(
        "id,name\n{}",
        (0..101)
            .map(|n| format!("r{n},Name"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    f.import("profile", "preview", &rows, false, false, 400);
    f.api("profile", "get", json!({"key":{"id":"a"}}), 404);
    assert_eq!(
        f.call(
            "POST",
            "api/profile/batch",
            Some(json!({"items":[]})),
            "editor",
            true,
            false,
            None,
            false
        )["status"],
        404
    );
}
#[test]
fn deterministic_public_export_ignores_paging_and_guard_decoding_is_explicit() {
    let f = Fixture::new("/admin/");
    let texts = [
        "=1+1",
        "+cmd",
        "-cmd",
        "@SUM(A1)",
        "\ttab",
        "\rcarriage",
        "'=literal",
        "\\N",
        "\\O",
        "\\path",
    ];
    for (n, text) in texts.iter().enumerate() {
        f.create("profile", json!({"id":format!("p{n}"),"name":text}));
    }
    let state = encode(&json!({"skip":7,"limit":1,"order":["name","desc"]}).to_string());
    let export = f.call(
        "GET",
        &format!("profile/export?state={state}"),
        None,
        "reader",
        false,
        false,
        None,
        false,
    );
    assert_eq!(export["status"], 200, "{export}");
    assert!(
        export["headers"]["content-disposition"]
            .as_str()
            .unwrap()
            .contains("attachment")
    );
    let csv = export["body"].as_str().unwrap();
    assert_eq!(
        f.call(
            "GET",
            &format!("profile/export?state={state}"),
            None,
            "reader",
            false,
            false,
            Some(Limits {
                columns: 1,
                cell: 1,
                ..Default::default()
            }),
            false
        )["body"],
        csv
    );

    let records = transfer::parse(csv.as_bytes(), Limits::default()).unwrap();
    assert_eq!(records.len(), 11);
    assert!(!records[0].iter().any(|f| f == "secret" || f == "projects"));
    let id = records[0].iter().position(|f| f == "id").unwrap();
    let name = records[0].iter().position(|f| f == "name").unwrap();
    assert_eq!(records[1][id], "p9");
    let again = f.call(
        "GET",
        &format!("profile/export?state={state}"),
        None,
        "reader",
        false,
        false,
        None,
        false,
    );
    assert_eq!(again["body"], csv);
    let plain = f.import("profile", "preview", csv, false, false, 200);
    let decoded = f.import("profile", "preview", csv, true, false, 200);
    for row in decoded["results"]["rows"].as_array().unwrap() {
        let key = row["values"]["id"].as_str().unwrap();
        let n: usize = key[1..].parse().unwrap();
        assert_eq!(row["values"]["name"], texts[n]);
    }
    assert_ne!(plain["results"]["rows"], decoded["results"]["rows"]);
    assert!(
        records
            .iter()
            .skip(1)
            .take(4)
            .all(|r| !r[name].starts_with('='))
    );
    // Confirm into fresh real databases, proving the public values survive an actual import.
    for guards in [false, true] {
        let roundtrip = Fixture::new("/");
        roundtrip.import("profile", "confirm", csv, guards, false, 200);
        for (n, original) in texts.iter().enumerate() {
            let result =
                roundtrip.api("profile", "get", json!({"key":{"id":format!("p{n}")}}), 200);
            let guarded = original.starts_with(['=', '+', '-', '@', '\t', '\r', '\'']);
            let expected = if !guards && guarded {
                format!("'{original}")
            } else {
                original.to_string()
            };
            assert_eq!(result["results"]["name"], expected);
        }
    }
    let refused = f.call(
        "GET",
        "profile/export",
        None,
        "reader",
        false,
        false,
        Some(Limits {
            rows: 1,
            ..Default::default()
        }),
        false,
    );
    assert_eq!(refused["status"], 400);
    assert!(refused["body"].as_str().unwrap().contains("narrow"));
    let refused = f.call(
        "GET",
        "profile/export",
        None,
        "reader",
        false,
        false,
        Some(Limits {
            bytes: 20,
            ..Default::default()
        }),
        false,
    );
    assert_eq!(refused["status"], 400);
}
#[test]
fn related_import_and_export_preserve_parent_scope() {
    let f = Fixture::new("/");
    f.create("profile", json!({"id":"p","name":"Parent"}));
    for id in ["j1", "j2"] {
        f.create("project", json!({"id":id,"ownerId":"p","title":id}));
    }
    let parent =
        generated_admin_web::host::related("project", json!({"id":"j1"}), "projectId", json!("j1"));
    let body = json!({"csv":"id,title,done,priority\nt1,Related,false,0","confirm":true});
    let preview = f.call(
        "POST",
        &format!("api/task/preview?{parent}"),
        Some(body.clone()),
        "editor",
        true,
        false,
        None,
        false,
    );
    assert_eq!(preview["status"], 200);
    let applied = f.call(
        "POST",
        &format!("api/task/confirm?{parent}"),
        Some(body),
        "editor",
        true,
        false,
        None,
        false,
    );
    assert_eq!(applied["status"], 200, "{applied}");
    f.create(
        "task",
        json!({"id":"other","projectId":"j2","title":"Other"}),
    );
    let export = f.call(
        "GET",
        &format!("task/export?{parent}"),
        None,
        "reader",
        false,
        false,
        None,
        false,
    );
    assert_eq!(export["status"], 200, "{export}");
    assert!(export["body"].as_str().unwrap().contains("Related"));
    assert!(!export["body"].as_str().unwrap().contains("Other"));
    let denied = f.call(
        "POST",
        &format!("api/task/confirm?{parent}"),
        Some(json!({"csv":"id,projectId,title\nescape,j2,Wrong","confirm":true})),
        "editor",
        true,
        false,
        None,
        false,
    );
    assert_eq!(denied["status"], 400);
    f.api("task", "get", json!({"key":{"id":"escape"}}), 404);
    let state = encode(&json!({"filters":{"projectId":"j2"}}).to_string());
    assert_eq!(
        f.call(
            "GET",
            &format!("task/export?{parent}&state={state}"),
            None,
            "reader",
            false,
            false,
            None,
            false
        )["status"],
        400
    );
}

/// Generated operation result limits cannot bypass the export refusal or its narrowing guidance.
#[test]
fn large_export_refuses_without_truncating_and_can_be_narrowed() {
    let f = Fixture::new("/");
    for prefix in ["Alpha", "Beta"] {
        f.create(
            "profile",
            json!({"id":prefix,"name":format!("{prefix}{}","x".repeat(600_000))}),
        );
    }
    let refused = f.call(
        "GET",
        "profile/export",
        None,
        "reader",
        false,
        false,
        None,
        false,
    );
    assert!(refused["status"].as_u64().unwrap() >= 400, "{refused}");
    assert!(refused["body"].as_str().unwrap().contains("narrow"));
    let state = encode(&json!({"q":"Alpha"}).to_string());
    let narrowed = f.call(
        "GET",
        &format!("profile/export?state={state}"),
        None,
        "reader",
        false,
        false,
        None,
        false,
    );
    assert_eq!(narrowed["status"], 200, "{narrowed}");
    let text = narrowed["body"].as_str().unwrap();
    assert!(text.len() > 600_000 && text.len() < 1_048_576);
    assert!(!text.contains("Beta"));
}
