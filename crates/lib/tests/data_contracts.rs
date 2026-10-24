//! Observable data/clone/parser contracts for milestone M1.

use stackpress_lib::data::{
    VisitOutcome, make_array, make_object, parse_arg_string, parse_args, parse_body,
    parse_json_object, parse_multipart, parse_query, should_be_an_array,
};
use stackpress_lib::{
    DataMap, DataPath, DataSet, Error, ErrorKind, FileData, Flow, Nest, ParseLimits, PathSegment,
    Result, SharedNative, Timestamp, Value,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Shared default budgets for representative bounded boundary cases.
fn limits() -> ParseLimits {
    // Keep representative tests on the production default policy unless a boundary case overrides
    // it.

    ParseLimits::default()
}

/// Unrelated buffers retain their allocation through set/remove/merge and dense edits.
#[test]
fn mutation_does_not_clone_unrelated_large_payloads() -> Result<()> {
    // Capture allocation identity rather than timing: whole-tree cloning changes it.
    let mut data = Nest::new();
    data.set("large", Value::Bytes(vec![7; 200_000]))?
        .set_path("rows.0.buffer", Value::Bytes(vec![1; 16_000]))?;
    let root_buffer = data
        .get("large")
        .and_then(Value::as_bytes)
        .unwrap()
        .as_ptr();
    let row_buffer = data
        .path("rows.0.buffer")
        .and_then(Value::as_bytes)
        .unwrap()
        .as_ptr();
    // Many small writes must not copy either sibling buffer, even through an array.
    for i in 0..128 {
        data.set(&format!("key-{i}"), i)?
            .set_path("rows.0.count", i)?;
    }
    data.remove_path("key-0")?;
    data.merge(Nest::from_object(
        serde_json::json!({"key-1":"replaced","extra":true}),
    )?)?;
    assert_eq!(
        root_buffer,
        data.get("large")
            .and_then(Value::as_bytes)
            .unwrap()
            .as_ptr()
    );
    assert_eq!(
        row_buffer,
        data.path("rows.0.buffer")
            .and_then(Value::as_bytes)
            .unwrap()
            .as_ptr()
    );
    Ok(())
}

/// Cached deltas agree with an independent full-tree budget check across edit histories.
#[test]
fn incremental_budgets_match_full_validation_and_fail_atomically() -> Result<()> {
    // Use small final budgets and varied edit histories; a fresh full-tree traversal is the
    // independent size oracle.

    let limits = ParseLimits {
        max_bytes: 48,
        max_entries: 20,
        max_depth: 6,
        ..limits()
    };
    let mut tight = Nest::with_limits(limits);
    let paths = [
        "items.0.name",
        "items.1.name",
        "items.4",
        "items.2",
        "profile.flag",
        "profile",
        "items.0",
        "other",
    ];
    // A relaxed candidate chooses shape; full Value validation is the budget oracle.
    for i in 0..160 {
        let before = tight.to_json()?;
        let mut candidate = Nest::from_value(tight.value().clone(), ParseLimits::default())?;
        let path = paths[(i * 7 + i / 9) % paths.len()];
        let value = if i % 3 == 0 {
            Value::String("payload".repeat(i % 4))
        } else {
            Value::Bool(i % 2 == 0)
        };
        let overlay =
            Nest::from_object(serde_json::json!({"profile":{"flag":false},"merged":[i%4]}))?;
        // Mix replacement, deletion, whole-key overlay and reset so cached totals cannot drift by
        // operation family.
        let candidate_result = if i % 31 == 0 {
            candidate.clear();
            Ok(())
        } else if i % 7 == 0 {
            candidate.merge(overlay.clone()).map(|_| ())
        } else if i % 5 == 0 {
            candidate.remove_path(path).map(|_| ())
        } else {
            candidate.set_path(path, value.clone()).map(|_| ())
        };
        let expected = candidate_result.is_ok() && candidate.value().validate(&limits).is_ok();
        let actual = if i % 31 == 0 {
            tight.clear();
            Ok(())
        } else if i % 7 == 0 {
            tight.merge(overlay).map(|_| ())
        } else if i % 5 == 0 {
            tight.remove_path(path).map(|_| ())
        } else {
            tight.set_path(path, value).map(|_| ())
        };
        assert_eq!(actual.is_ok(), expected, "operation {i} at {path}");
        assert_eq!(
            tight.to_json()?,
            if expected {
                candidate.to_json()?
            } else {
                before
            }
        );
        tight.value().validate(&limits)?;
    }
    // Deletion can increase bytes by materializing sparse numeric keys; reject atomically.
    let mut small = Nest::with_limits(ParseLimits {
        max_bytes: 2,
        ..limits
    });
    small.set("a", Value::Array(vec![Value::Null; 4]))?;
    let before = small.to_json()?;
    assert!(small.remove_path("a.0").is_err());
    assert_eq!(small.to_json()?, before);
    // Failed append payload validation must leave the existing vector and totals intact.
    let append = DataPath::from_segments([PathSegment::Key("a".into()), PathSegment::Append])?;
    assert!(small.set_path(&append, Value::String("xx".into())).is_err());
    assert_eq!(small.to_json()?, before);
    // The largest canonical identity cannot be incremented; resolve it before creating or
    // converting anything.
    let mut maximum = Nest::new();
    maximum.set(
        "items",
        Value::Object(
            [(usize::MAX.to_string(), Value::Null)]
                .into_iter()
                .collect(),
        ),
    )?;
    let append = DataPath::from_segments([PathSegment::Key("items".into()), PathSegment::Append])?;
    let before = maximum.to_json()?;
    assert!(maximum.set_path(&append, true).is_err());
    assert!(maximum.remove_path(&append).is_err());
    assert_eq!(maximum.to_json()?, before);
    Ok(())
}

/// Selected store budgets survive explicit JSON conversion instead of reverting to defaults.
#[test]
fn selected_json_budgets_and_nested_error_context_are_preserved() -> Result<()> {
    // This string deliberately exceeds the default wire budget but fits its explicit finite
    // workload.
    let policy = ParseLimits {
        max_bytes: ParseLimits::default().max_bytes + 128,
        ..limits()
    };
    let text = "x".repeat(ParseLimits::default().max_bytes + 1);
    let value = Value::from(text);
    assert!(value.to_json().is_err());
    assert!(value.to_json_with(&policy)?.is_string());
    let mut data = Nest::with_limits(policy);
    data.set("text", value)?;
    assert!(data.to_json()?["text"].is_string());
    assert!(data.path_as::<String>("text")?.is_some());
    assert!(serde_json::to_string(&data)?.len() > ParseLimits::default().max_bytes);
    // A selected subtree must retain the unsupported descendant, not overwrite its path with the
    // parent alone.
    let mut native = Nest::new();
    native.set_path("profile.native", SharedNative::new(7u32))?;
    assert_eq!(
        native
            .path_as::<serde_json::Value>("profile")
            .unwrap_err()
            .path(),
        Some("profile.native")
    );
    Ok(())
}

/// User destructors can unwind; ordinary committed data must retain a valid budget cache.
#[test]
fn native_drop_unwind_cannot_stale_mutation_totals() -> Result<()> {
    struct PanicsOnDrop;
    impl Drop for PanicsOnDrop {
        /// Simulate application-owned native cleanup failing after its final shared owner.
        fn drop(&mut self) {
            // The framework must not run this destructor between tree and cache publication.
            panic!("intentional native destructor panic");
        }
    }
    for operation in ["set", "merge", "clear", "remove"] {
        // A one-field store fits exactly; stale pre-edit totals would corrupt the following
        // boundary checks.
        let limits = ParseLimits {
            max_bytes: 4,
            max_entries: 4,
            ..limits()
        };
        let mut data = Nest::with_limits(limits);
        data.set("key", SharedNative::new(PanicsOnDrop))?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match operation {
            "set" => {
                data.set("key", Value::Array(vec![Value::Null; 2])).unwrap();
            }
            "merge" => {
                data.merge(Nest::from_object(serde_json::json!({"key":[null,null]})).unwrap())
                    .unwrap();
            }
            "remove" => drop(data.remove("key").unwrap()),
            _ => data.clear(),
        }));
        assert!(result.is_err(), "the user panic must propagate normally");
        if operation == "clear" || operation == "remove" {
            assert!(data.is_empty());
            // Clearing must release old usage even though its removed value unwound.
            data.set("key", Value::Array(vec![Value::Null; 2]))?;
        }
        assert_eq!(
            data.get("key")
                .and_then(Value::as_array)
                .map(|array| array.len()),
            Some(2)
        );
        assert!(
            data.set("x", true).is_err(),
            "the committed full store still enforces its limit"
        );
        data.remove("key")?;
        data.set("x", true)?;
        data.value().validate(&limits)?;
    }
    Ok(())
}

/// Dot paths are convenient while literal keys and typed reads stay unambiguous.
#[test]
fn nested_crud_preserves_presence_shapes_and_atomic_failure() -> Result<()> {
    // Set up literal and nested keys together so lookup spelling and atomic traversal errors are
    // observable.

    let mut data = Nest::new();
    data.set("literal.key", true)?
        .set_path("profile.name", "Ada")?
        .set_path("profile.age", 42)?;
    assert_eq!(data.get("literal.key").and_then(Value::as_bool), Some(true));
    assert!(data.path("literal.key").is_none());
    assert_eq!(data.path_as::<String>("profile.name")?, Some("Ada".into()));
    assert_eq!(data.path_as::<i64>("profile.age")?, Some(42));
    assert!(data.path_as::<String>("profile.age").is_err());
    data.set("null", Value::Null)?
        .set("empty", Value::Object(DataMap::new()))?;
    assert!(data.has("null"));
    assert!(data.get("null").is_some_and(Value::is_null));
    assert!(!data.has("missing"));
    // Snapshot observable data before a failing traversal; rejection must leave it byte-for-byte
    // unchanged.
    let before = data.stringify(false)?;
    assert!(data.set_path("profile.age.child", true).is_err());
    assert_eq!(data.stringify(false)?, before);
    assert!(data.path("profile..name").is_none());
    assert!(data.path_as::<String>("profile..name").is_err());
    assert!(data.remove_path("profile.unknown")?.is_none());
    assert!(data.remove_path("profile.name")?.is_some());
    assert!(data.path("profile.name").is_none());
    let age = DataPath::dot("profile.age")?;
    assert!(data.has_path(&age) && data.has_path("profile.age"));
    assert_eq!(data.path_as::<i64>(&age)?, Some(42));
    assert_eq!(
        data.path_as::<String>(&age).unwrap_err().path(),
        Some("profile.age")
    );
    assert!(data.remove("literal.key")?.is_some());
    let literal = DataPath::from_segments([
        PathSegment::Key("literal.key".into()),
        PathSegment::Index(0),
        PathSegment::Key("0".into()),
        PathSegment::Append,
    ])?;
    assert_eq!(literal.to_string(), "[\"literal.key\"].0[\"0\"][]");
    Ok(())
}

/// Sparse insertion/deletion never shifts an existing numeric identity.
#[test]
fn numeric_normalization_append_and_twelve_element_order() -> Result<()> {
    // Insert backwards through 12 to detect lexicographic sorting and sparse-allocation mistakes.

    let mut data = Nest::new();
    for i in (0..13).rev() {
        data.set_path(&format!("numbers.{i}"), i)?;
    }
    let values = data
        .path("numbers")
        .and_then(Value::as_array)
        .expect("dense array");
    assert_eq!(
        values.iter().map(Value::as_i64).collect::<Vec<_>>(),
        (0..13).map(Some).collect::<Vec<_>>()
    );
    // Remove an interior numeric identity; later positions must stay keyed by their original
    // indices.
    data.remove_path("numbers.2")?;
    assert!(data.path("numbers").is_some_and(Value::is_object));
    assert_eq!(data.path("numbers.3").and_then(Value::as_i64), Some(3));
    data.set_path(
        &DataPath::from_segments([PathSegment::Key("numbers".into()), PathSegment::Append])?,
        99,
    )?;
    assert_eq!(data.path("numbers.13").and_then(Value::as_i64), Some(99));
    // Filling the gap should restore dense numeric order without lexicographic reordering.
    data.set_path("numbers.2", 2)?;
    assert_eq!(
        data.path("numbers")
            .and_then(Value::as_array)
            .map(|v| v.len()),
        Some(14)
    );
    data.set_path("sparse.1000000000000", true)?
        .set_path("sparse.01", false)?;
    assert!(data.path("sparse").is_some_and(Value::is_object));
    let before = data.stringify(false)?;
    assert!(data.remove_path("numbers.999")?.is_none());
    assert_eq!(data.stringify(false)?, before);
    Ok(())
}

/// Clones own ordinary buffers but only explicitly Shared payloads retain identity.
#[test]
fn clone_shares_only_named_native_variant_and_redacts_payload() -> Result<()> {
    // Use mutable ordinary buffers and a drop-counted native payload to distinguish deep copies
    // from explicit sharing.

    struct DropCount(Arc<AtomicUsize>);
    impl Drop for DropCount {
        /// Record destruction after the final shared owner.
        fn drop(&mut self) {
            // Count final native destruction so temporary Value clones cannot hide a leaked shared
            // owner.

            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let shared = SharedNative::new(DropCount(Arc::clone(&drops)));
    let mut data = Nest::new();
    data.set_path("profile.name", "Ada")?
        .set("bytes", Value::Bytes(vec![0, 255]))?
        .set("file", FileData::new(vec![1, 2], "secret.txt", "x/type"))?
        .set("native", shared.clone())?;
    // Mutate the clone only; assertions below require the original ordinary data to remain
    // independent.
    let mut copy = data.clone();
    copy.set_path("profile.name", "Grace")?
        .set("bytes", Value::Bytes(vec![7]))?;
    assert_eq!(data.path_as::<String>("profile.name")?, Some("Ada".into()));
    assert_eq!(
        data.get("bytes").and_then(Value::as_bytes),
        Some([0, 255].as_slice())
    );
    let Value::File(mut file) = copy.get("file").expect("file").clone() else {
        panic!("file variant")
    };
    file.data[0] = 9;
    assert!(matches!(data.get("file"),Some(Value::File(file)) if file.data == [1,2]));
    assert!(
        shared.ptr_eq(
            copy.get("native")
                .and_then(Value::as_shared)
                .expect("shared")
        )
    );
    assert!(shared.downcast_ref::<DropCount>().is_some());
    assert!(shared.downcast_ref::<String>().is_none());
    assert!(!format!("{data:?}").contains("Ada"));
    assert!(!format!("{shared:?}").contains("secret"));
    drop(data);
    drop(copy);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    // Only dropping the final explicitly Shared handle may destroy its native payload.
    drop(shared);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    Ok(())
}

/// Non-JSON values retain their types and report the exact unsupported output path.
#[test]
fn explicit_serialization_rejects_non_json_variants_and_invalid_numbers() -> Result<()> {
    // Exercise valid rich values that deliberately cannot cross the default JSON boundary.

    assert!(Value::number(f64::NAN).is_err());
    assert!(Value::number(f64::INFINITY).is_err());
    assert!(Timestamp::new(0, 1_000_000_000).is_err());
    let date = Timestamp::new(-1, 123)?;
    assert_eq!(date.nanoseconds(), 123);
    for value in [
        Value::DateTime(date),
        Value::Bytes(vec![]),
        Value::File(FileData::new(vec![], "", "x")),
        Value::Shared(SharedNative::new(7u32)),
    ] {
        let mut data = Nest::new();
        data.set_path("profile.payload", value)?;
        // Assert the unsupported rich child reports its actual nested location rather than silently
        // becoming null/text.
        let error = data.to_json().expect_err("no implicit wire conversion");
        assert_eq!(error.path(), Some("profile.payload"));
        assert!(serde_json::to_string(&data).is_err());
    }
    let value: Value = serde_json::from_str(r#"{"value":false,"count":0,"empty":"","null":null}"#)?;
    assert_eq!(
        value.to_json()?,
        serde_json::json!({"value":false,"count":0,"empty":"","null":null})
    );
    Ok(())
}

/// Overlay replaces whole top-level values while preserving insertion order.
#[test]
fn overlays_collections_and_explicit_shape_helpers() -> Result<()> {
    // Use overlapping nested objects to prove top-level replacement rather than accidental
    // recursive merge.

    let mut data = Nest::from_object(serde_json::json!({"profile":{"a":1},"first":true}))?;
    data.merge(Nest::from_object(
        serde_json::json!({"profile":{"b":2},"last":false}),
    )?)?;
    assert!(data.path("profile.a").is_none());
    assert_eq!(
        data.keys().map(String::as_str).collect::<Vec<_>>(),
        ["profile", "first", "last"]
    );
    assert_eq!(data.values().count(), 3);
    let mut map: DataMap<String, Value> = [("b".into(), 2.into()), ("a".into(), 1.into())]
        .into_iter()
        .collect();
    assert!(!should_be_an_array(&map));
    assert_eq!(
        make_array(map.clone())
            .iter()
            .map(Value::as_i64)
            .collect::<Vec<_>>(),
        [Some(1), Some(2)]
    );
    map = make_object(vec!["a".into(), "b".into()]);
    assert!(should_be_an_array(&map));
    // Duplicates must collapse while preserving the first insertion order of distinct values.
    let set: DataSet<_> = ["b", "a", "b"].into_iter().collect();
    assert_eq!(set.iter().copied().collect::<Vec<_>>(), ["b", "a"]);
    assert!(set.get_index(2).is_none());
    assert!(Nest::from_object(serde_json::json!([])).is_err());
    data.clear();
    assert!(data.is_empty());
    Ok(())
}

/// Query decoding, scalar coercion and JSON insertion share path policy, not bugs.
#[test]
fn query_parsing_preserves_falsy_values_nested_arrays_and_decoding() -> Result<()> {
    // Combine empty/falsy values with nested/repeated keys and escaped delimiters in one real
    // query.

    let data = parse_query(
        concat!(
            "?empty=&zero=0&flag=false&nil=null&tags[]=a&tags[]=b",
            "&profile[name]=Ada+Lovelace&json=%7B%22x%22%3A1%7D&raw=%2526"
        ),
        &limits(),
    )?;
    assert_eq!(data.path_as::<String>("empty")?, Some("".into()));
    assert_eq!(data.path_as::<i64>("zero")?, Some(0));
    assert_eq!(data.path_as::<bool>("flag")?, Some(false));
    assert!(data.path("nil").is_some_and(Value::is_null));
    assert_eq!(
        data.path_as::<Vec<String>>("tags")?,
        Some(vec!["a".into(), "b".into()])
    );
    assert_eq!(
        data.path_as::<String>("profile.name")?,
        Some("Ada Lovelace".into())
    );
    assert_eq!(data.path_as::<i64>("json.x")?, Some(1));
    assert_eq!(data.path_as::<String>("raw")?, Some("%26".into()));
    assert!(parse_query("", &limits())?.is_empty());
    // Malformed UTF-8 and truncated percent escapes must fail instead of producing replacement
    // characters.
    assert!(parse_query("x=%FF", &limits()).is_err());
    assert!(parse_query("x=%A", &limits()).is_err());
    assert!(parse_query("x[a]suffix=z", &limits()).is_err());
    assert!(parse_query("x[a=z", &limits()).is_err());
    assert_eq!(
        parse_query("x=1&x=2", &limits())?.path_as::<i64>("x")?,
        Some(2)
    );
    assert_eq!(
        parse_query("x={invalid", &limits())?.path_as::<String>("x")?,
        Some("{invalid".into())
    );
    Ok(())
}

/// CLI values containing query delimiters remain one supplied argument.
#[test]
fn args_and_small_quoted_tokenizer_preserve_literal_values() -> Result<()> {
    // Supply argv with query delimiters and a terminator; no value may become another field.

    let args = [
        "skip",
        "--token",
        "a&b=c+d%20",
        "-xyz",
        "--name=Grace Hopper",
        "--roles[]",
        "ADMIN",
        "--roles[]",
        "USER",
        "--",
        "-literal",
    ];
    let args = args.map(String::from);
    let data = parse_args(&args, 1, &limits())?;
    assert_eq!(data.path_as::<String>("token")?, Some("a&b=c+d%20".into()));
    assert_eq!(data.path_as::<String>("name")?, Some("Grace Hopper".into()));
    assert_eq!(data.path_as::<bool>("z")?, Some(true));
    assert_eq!(data.path_as::<String>("0")?, Some("-literal".into()));
    assert_eq!(
        data.path_as::<Vec<String>>("roles")?,
        Some(vec!["ADMIN".into(), "USER".into()])
    );
    assert_eq!(
        parse_arg_string(r#"--name "Ada Lovelace" --empty '' -v"#, &limits())?
            .path_as::<String>("empty")?,
        Some("".into())
    );
    // Incomplete quotes and missing long-option values are errors, not silently repaired input.
    assert!(parse_arg_string("--name 'unclosed", &limits()).is_err());
    assert!(parse_arg_string("--missing", &limits()).is_err());
    assert!(parse_arg_string("- --", &limits())?.is_empty());
    assert!(parse_args(&args, 100, &limits())?.is_empty());
    Ok(())
}

/// Buffered multipart preserves raw binary bytes and ignores nameless dispositions.
#[test]
fn multipart_binary_empty_files_nested_fields_and_media_parameters() -> Result<()> {
    // Build framed bytes containing an invalid UTF-8 byte and a false boundary candidate.

    let mut body = concat!(
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"profile[name]\"\r\n\r\nAda\r\n",
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"file\"; filename=\"raw.bin\"\r\n",
        "Content-Type: application/octet-stream\r\n\r\n"
    )
    .as_bytes()
    .to_vec();
    let bytes = b"\0\xFF\r\n--BOUNDx\r\nend";
    body.extend_from_slice(bytes);
    body.extend_from_slice(
        concat!(
            "\r\n--BOUND\r\n",
            "Content-Disposition: form-data; name=\"empty\"; filename=\"\"\r\n\r\n\r\n",
            "--BOUND\r\n\r\nignored\r\n--BOUND--\r\n"
        )
        .as_bytes(),
    );
    let data = parse_body("Multipart/Form-Data; boundary=\"BOUND\"", &body, &limits())?;
    assert_eq!(data.path_as::<String>("profile.name")?, Some("Ada".into()));
    assert!(matches!(
        data.get("file"),
        Some(Value::File(file)) if file.data == bytes && file.name == "raw.bin"
    ));
    assert!(matches!(data.get("empty"),Some(Value::File(file)) if file.data.is_empty()));
    assert_eq!(data.len(), 3);
    // Quoted semicolons/escaped quotes/spaces belong to filename metadata, not MIME framing.
    let quoted_file = concat!(
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"file\"; filename=\" a;\\\"b\\\".txt \"\r\n",
        "\r\nx\r\n--BOUND--\r\n"
    )
    .as_bytes();
    let quoted = parse_multipart(quoted_file, "BOUND", &limits())?;
    assert!(matches!(quoted.get("file"),Some(Value::File(file)) if file.name == " a;\"b\".txt "));
    // Truncation and missing/invalid boundary framing must reject the complete buffered input.
    assert!(parse_multipart(&body[..body.len() - 12], "BOUND", &limits()).is_err());
    assert!(parse_multipart(b"", "BOUND", &limits()).is_err());
    assert!(parse_multipart(&body, "bad\r\n", &limits()).is_err());
    assert!(parse_body("multipart/form-data", &body, &limits()).is_err());
    assert!(parse_body("text/plain", b"hello", &limits()).is_err());
    assert!(parse_body("application/json; charset=\"UTF\"junk", b"{}", &limits()).is_err());
    assert!(parse_body("application/json; charset=bad value", b"{}", &limits()).is_err());
    assert_eq!(
        parse_body(
            "Application/JSON; charset=\"UTF-8\"",
            br#" {"active":false}"#,
            &limits()
        )?
        .path_as::<bool>("active")?,
        Some(false)
    );
    assert!(parse_json_object("invalid", &limits()).is_err());
    assert!(parse_json_object("[]", &limits()).is_err());
    Ok(())
}

/// Every budget failure is observable and never partially commits a mutation.
#[test]
fn finite_limits_cover_bytes_depth_entries_and_part_headers() -> Result<()> {
    // Vary one finite budget at a time so the rejection cause and unchanged state stay clear.

    let small = ParseLimits {
        max_bytes: 8,
        ..limits()
    };
    assert!(parse_query("long=value", &small).is_err());
    let mut data = Nest::with_limits(small);
    data.set("a", "b")?;
    assert!(data.set("c", "too much data").is_err());
    assert_eq!(data.stringify(false)?, r#"{"a":"b"}"#);
    // A valid JSON object still fails when its nested container depth exceeds the chosen limit.
    let shallow = ParseLimits {
        max_depth: 1,
        ..limits()
    };
    assert!(parse_json_object(r#"{"a":{"b":1}}"#, &shallow).is_err());
    let few = ParseLimits {
        max_entries: 2,
        ..limits()
    };
    assert!(parse_query("a=1&b=2", &few).is_err());
    assert!(parse_json_object(r#"{"a":1,"b":2}"#, &few).is_err());
    // Multipart header budget is independent of the total buffered body budget.
    let headers = ParseLimits {
        max_part_headers: 2,
        ..limits()
    };
    assert!(
        parse_multipart(
            b"--B\r\nContent-Disposition: form-data; name=\"x\"\r\n\r\n1\r\n--B--",
            "B",
            &headers
        )
        .is_err()
    );
    assert!(DataPath::dot("").is_err());
    assert!(DataPath::dot("a..b").is_err());
    assert!(DataPath::dot(&"x.".repeat(65)).is_err());
    assert!(DataPath::dot("a.999999999999999999999999999").is_err());
    // Reject oversized/deep spelling at the path boundary, before collecting all supplied
    // components.
    assert!(DataPath::dot(&"key.".repeat(5000)).is_err());
    assert!(parse_query(&format!("x{}=1", "[a]".repeat(65)), &limits()).is_err());
    Ok(())
}

/// Named async visitors demonstrate sequential stop/error behavior without boxing.
async fn stop_on_second(key: String, value: &Value) -> Result<Flow> {
    // Suspend before inspecting the element so the visitor's borrowed value must remain valid
    // across await.

    tokio::task::yield_now().await;
    assert!(value.as_i64().is_some());
    Ok(if key == "1" {
        Flow::Abort
    } else {
        Flow::Continue
    })
}
/// Visiting a missing/empty collection differs from completing a nonempty one.
#[tokio::test]
async fn visitors_are_sequential_and_can_abort() -> Result<()> {
    // Use a three-item list so cooperative stop differs from empty/missing and full completion.

    let data = Nest::from_object(serde_json::json!({"list":[1,2,3],"empty":[]}))?;
    assert_eq!(
        data.visit("list", stop_on_second).await?,
        VisitOutcome::Aborted
    );
    assert_eq!(
        data.visit("empty", stop_on_second).await?,
        VisitOutcome::Empty
    );
    assert_eq!(
        data.visit("missing", stop_on_second).await?,
        VisitOutcome::Empty
    );
    assert!(data.visit("list.0", stop_on_second).await.is_err());
    Ok(())
}

/// Common input failures use owned diagnostic errors rather than panics.
#[test]
fn errors_preserve_origins_spans_and_public_disclosure() -> Result<()> {
    // Capture a deliberate source line, then enrich the error without losing its original
    // cause/location.

    let line = line!() + 1;
    let error = Error::new("private cause").with_code(419).with_span(1, 2)?;
    assert_eq!(error.origin().line, line);
    assert_eq!(error.public_view().error, "Internal Server Error");
    assert_eq!(error.public_view().status, "Page Expired");
    assert_eq!(error.span().expect("span").end, 2);
    assert!(Error::new("x").with_span(3, 2).is_err());
    assert_eq!(Error::invalid("bad field").kind(), ErrorKind::InvalidInput);
    Ok(())
}
