//! Real controller state transitions independent of browser event plumbing.
use super::*;
use stackpress_example_profile_ui_contracts::{Profile, REVISION};
use std::{cell::RefCell, rc::Rc};

/// Make a typed public result with no duplicated search implementation.
fn reply(name: &str) -> SearchReply {
    SearchReply {
        revision: REVISION,
        rows: vec![Profile {
            id: "abc".into(),
            name: name.into(),
        }],
        total: 1,
    }
}

/// Invalid input, late completion, empty result, errors and disposal share one controller.
#[test]
fn controller_preserves_newest_result_and_disposal() {
    let owner = Owner::new();
    let completions = Rc::new(RefCell::new(Vec::<SearchCompletion>::new()));
    let (controller, transport) = owner.with(|| {
        let controller = SearchController::new(SearchPage {
            input: SearchInput::new("abc".into()).unwrap(),
            reply: reply("Seed"),
        });
        let callbacks = Rc::clone(&completions);
        let transport = UnsyncCallback::new(move |(_, done)| callbacks.borrow_mut().push(done));
        (controller, transport)
    });
    assert_eq!(controller.total.get_untracked(), 1);
    assert!(completions.borrow().is_empty(), "no initial request");
    controller.id.set(String::new());
    controller.submit(transport);
    assert!(!controller.error.get_untracked().is_empty());
    assert!(completions.borrow().is_empty());
    controller.id.set("abc".into());
    controller.submit(transport);
    controller.submit(transport);
    completions.borrow()[1](Ok(reply("Newest")));
    completions.borrow()[0](Ok(reply("Late")));
    assert_eq!(controller.rows.get_untracked()[0].1, "Newest");
    assert!(!controller.pending.get_untracked());
    controller.submit(transport);
    completions.borrow()[2](Ok(SearchReply {
        revision: REVISION,
        rows: Vec::new(),
        total: 0,
    }));
    assert_eq!(controller.total.get_untracked(), 0);
    assert!(controller.rows.get_untracked().is_empty());
    controller.submit(transport);
    completions.borrow()[3](Err("Safe failure".into()));
    assert_eq!(controller.error.get_untracked(), "Safe failure");
    controller.submit(transport);
    let late = std::sync::Arc::clone(&completions.borrow()[4]);
    owner.cleanup();
    late(Ok(reply("Disposed")));
    assert!(controller.total.try_get_untracked().is_none());
}
