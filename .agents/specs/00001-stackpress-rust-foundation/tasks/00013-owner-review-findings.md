# Task 00013: owner visual review findings

Owner feedback from the Slice 8 review preview (main session), 2026-10-28. Each item names the page, what the owner saw, and the coordinator's triage. These are owner directives unless marked as a question. The review is still in progress; later findings are appended here.

## Bugs

| ID | Where | Owner saw | Triage |
| --- | --- | --- | --- |
| F01 | `/review` → Profiles / Projects / Tasks | "not working" | Before a review user is chosen, admin page navigation returns the raw JSON envelope `{"code":403,"status":"Forbidden",...}`. A document (page) request must never show a JSON envelope: render an HTML page that explains the denial and the next action (control 003/005: denied states explain the next action). This is a generated-admin host finding, not only a review-host one. In the review host, the default caller should be Editor so the links work on first open; Denied stays an explicit scenario. |
| F02 | `/review` → 012: Late failure, 012: Pending stream | "not working" | Both pages stay on "Loading page. If interrupted, reload this page." The browser reports an incomplete chunked response and nothing else changes, so a person cannot tell failure or interruption from slow loading. Needs a visible, safe failure/interrupted state (no false success, no hydrated partial page), within the accepted S7/S8 streaming contract. Design checkpoint required. |
| F03 | Gallery rich content | "encodedrelativeexplicit link": no space between links | Fixture issue: the sample HTML has adjacent anchors with no separating text. The sanitizer behaviour is correct (unsafe hrefs removed). Fix the fixture and label it. |

## Intentional, but confusing to a reviewer

| ID | Where | Owner saw | Triage |
| --- | --- | --- | --- |
| F04 | Gallery `#resource`, Notes textarea, stream `#details`, admin label `Name </script> 🦀` | Script-like text shown on the page ("is this intentional?") | Intentional: hostile sample strings proving that content is shown as plain text and never executed. Proposed change: keep them only in a clearly labelled "Safety samples" section of the review/gallery, and use realistic data everywhere else. Keep the generated admin's hostile label in the automated proofs; for the owner review, either label it or use a realistic fixture schema. |
| F05 | Gallery "Typed nested rows" (missing/null/false/true select + "Set null") | "not familiar with this UI; was this fieldset from frui?" | Not from frui: a developer-facing demonstration of a nested optional boolean's states. Not an acceptable end-user control. Propose either a proper nullable-boolean control, consistent with frui conventions, or removal from the gallery; owner decision at the design checkpoint. |

## Design changes (owner directives)

Use Lucide icons per the [chrisai-iconography](../../../skills/chrisai-iconography/SKILL.md) skill. Use `../frui` (read-only reference; never modify or build it) for comparable components and conventions.

| ID | Component | Directive |
| --- | --- | --- |
| D01 | Collection rows (text list, metadata key/value) | Input(s) and row actions inline on one row; Move up / Move down / Remove become Lucide icon buttons on the right of the input, with accessible names. Remove the visible per-row labels ("Value 1", "Value 2", "key 4", "value 4") while keeping accessible labelling. Key, value and buttons all on one line. |
| D02 | Tag list ("Editable tags") | Removable chips plus an add input and button, similar to shadcn tag-input examples. |
| D03 | Data table | Sortable headers must not look like buttons: header text with a sort indicator icon, still keyboard-operable and announcing sort state. Remove the visible "Select page" label (keep it accessible). The example must have many more columns and rows so overflow behaviour can be judged; compare with frui's table overflow measures. |
| D04 | Rating | Interactive star icons (Lucide), keyboard-operable, with a clear way to clear. |
| D05 | Generated admin `/generated-admin/roles` | "not styled at all": apply the framework styling. |
