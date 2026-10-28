# Slice 8 owner review checklist

Status: consolidated M0–M4 owner-findings re-review for the [approved review preview](00013-review-preview-design.md). Owner acceptance is recorded separately. Installed steps (011 and the installed half of 013) wait for Part B.

Use two sessions only: first the main root / hydrate / disabled session for 001–010, 012 and the disabled half of 013, then the compiled plugin session for the plugin half of 013. Open `http://127.0.0.1:<assigned-port>/review`, replacing the port with the one reported by devmetrics; this host uses that exact origin. Keep `/review` in a second tab for its labelled caller/failure fixtures; a fresh visitor defaults to Editor; choose Editor to reset an earlier caller selection. These fixtures are not a login system. Record pass/fail and observations per control.

## 00013-001 — useful content before JavaScript

Paths: `/review`, `/ui-gallery/gallery`, `/generated-admin/profile/search`.

1. Choose Editor and Pause hydration in the review panel; open each page in the hydrate session.
2. Check that labels, gallery content and generated rows are readable before starting browser controls. Ordinary Notes, profile names and the page title use realistic text; deliberately hostile values are grouped under Safety samples.
3. Type in an input, focus it, select some text and scroll the page.
4. Start browser controls; check that the value, selection, focus and scroll remain and content does not flash or duplicate.
5. Before starting browser controls on a fresh gallery page, check Select page with no rows selected; start controls and check all 12 visible rows become selected. Repeat once with one row already selected. Restore automatic hydration in the review panel.
6. Text rows and Metadata rows have inline inputs and Up/Down/Remove icons with descriptive accessible names. Tab to the actions; use Enter/Space to add, move and remove, checking focus remains usable.
7. Tags are plain text chips with X remove buttons. Add a draft using Enter or Add; whitespace-only drafts stay untouched. Remove then add to change a tag; there is no chip editing or reordering. Backspace in an empty add input focuses the last X without deleting. Removing a chip focuses a neighbour or the add input; input composition must not append accidentally. Fill to the bound and inspect disabled Add behavior. Existing empty values remain visible as “(empty)” in the automated fixture.

## 00013-002 — keyboard dialogs

Path: `/ui-gallery/gallery`.

1. Use Tab and Enter to open the profile dialog; edit its input without the mouse.
2. Tab and Shift-Tab around it; check that focus stays inside and remains visible.
3. Open/close the nested dialog, then use Escape on the ordinary dialog; check focus returns to its opener.
4. Open the explicit-close dialog; verify Escape follows its stated policy and its Close button works.
5. Try Remove opener and Unmount dialog; verify focus reaches a usable fallback and the page still works.

## 00013-003 — tables and list state

Paths: `/ui-gallery/gallery`, `/generated-admin/task/search`, `/review`.

1. The gallery Paged people table has 48 rows, 12 columns and 12 rows per page. Read its caption/headings; filter, sort and move between pages. Headings use inline text and Lucide sort indicators; the page checkbox keeps its accessible Select page name.
   Scroll sideways to Reference and down inside the table: headings remain sticky, compact values stay on one line and Notes wrap. Selection, sorting, filtering and paging must still work.
2. Open the list's current-state link, refresh and use Back; check that the represented filter/sort/page agrees.
3. Choose Slow next read in the second tab and load again; check useful rows remain while loading.
4. Filter to no matches; check that the empty state explains what to do.
5. Try Failed next read and Denied caller separately on a generated-admin document URL. Expect styled HTML “Unable to load this page” or “Access denied”, with safe guidance/Return to list and the Review fixtures link. No raw JSON envelope or private row appears. Restore Editor.

## 00013-004 — generated record changes

Paths: `/generated-admin/profile/search`, `/generated-admin/profile/create`, `/review`.

1. As Editor, create a uniquely named record; submit an invalid field first and check its named error/focus and preserved draft.
2. Correct/save it, edit it and copy it using a new identity; check the original remains distinct.
3. Remove and restore the record; check each destructive confirmation and visible result.
4. Edit another draft, change to Reader in the second tab and submit; check denial preserves the draft, then restore Editor.
5. Arm Lose next write acknowledgement and save once; check the uncertain warning and disabled repeat submission.
6. Open the list in another tab to inspect the actual stored result; do not assume rollback or automatically retry.

## 00013-005 — related records and denied access

Paths: `/generated-admin/profile/search`, `/generated-admin/project/search`, `/generated-admin/task/search`, `/review`.

1. Open a seeded parent's related projects/tasks; check the parent context and related links remain visible.
2. Create a related record as Editor and check it appears only under the correct parent.
3. Switch to Reader; check read access remains and unavailable write actions disappear.
4. Paste the earlier Create URL directly; expect the styled Access denied page and access guidance. In browser developer tools, repeat a captured write request with a forged `x-admin-actor: editor`; check Reader still cannot write.
5. Switch to Denied and paste a saved detail/related URL; expect the same HTML denial and no private record. Restore Editor. API/write/export denials remain machine-readable JSON.

## 00013-006 — CSV preview, confirm and export

Paths: `/generated-admin/profile/import`, `/generated-admin/profile/export`, `/review`.

1. Download the panel's invalid sample CSV; select it for import and preview without saving.
2. Read the row errors, correct the file locally and select it again; preview and check every decoded row.
3. Cancel once and check confirmation is unavailable until a fresh preview; repeat with Slow next read to inspect pending/progress states.
4. Explicitly confirm the valid preview; check the stored records and try a denied confirmation as Reader.
5. Export a filtered set; inspect headers for public fields and open the formula-leading sample as inert text in a spreadsheet.
6. If reimporting an export, compare previews with Decode Stackpress export guards off/on before confirming.

## 00013-007 — language, formatting and dirty drafts

Paths: `/ui-preferences/preferences`, `/ui-preferences/fr/preferences`, `/ui-format/edit/en`, `/ui-format/edit/fr`, `/review`.

1. Type an unsaved preference-page draft and switch English/French; check the draft stays while text, samples, title, document language and Home link agree.
2. Refresh, follow Home and reopen the browser with the same persistent profile; check the saved language remains.
3. Arm Fail next preference save, change language and check the failure message and retained draft/current presentation.
4. Open the formatting editor; enter a partial decimal/date/phone, change locale while dirty, then commit/cancel and inspect raw text versus successful values.
5. Compare English/French deterministic display samples. Preference-page date/number samples are translated fixtures; they do not establish named-timezone conversion or OS IME support.

## 00013-008 — theme and public notice

Paths: `/ui-preferences/preferences`, `/review`.

1. Select Dark, refresh and watch the initial paint; repeat with Light. Only Light and Dark exist (G39); there is no System option.
   In a fresh private window (nothing saved), the first paint follows the OS light/dark setting; changing the OS setting while the page is open does not switch it. With JavaScript disabled and nothing saved, the page is Light (G40).
2. Check text, inputs, focus and errors remain readable; reopen with the same browser profile and inspect the saved theme.
3. Use the review panel to issue one public notice, then open the preferences page and dismiss it.
4. Interact and refresh; check it does not reappear in the same root. Concurrent-tab cookie delivery remains best effort.

## 00013-009 — inert rich content

Paths: `/ui-gallery/gallery`, `/ui-format/`.

1. In the gallery Safety samples section, read the deliberately hostile Resource, labelled Textarea escaping sample and sanitized HTML/Markdown. Markup must remain inert. The textarea retains its literal leading newline, quotes and Unicode after startup; ordinary Notes elsewhere contain normal text.
   The rejected encoded JavaScript, protocol-relative, data and relative URL examples have clear separate labels with visible spaces. Only Allowed HTTPS link is navigable.
2. Check no popup, hidden form, broken layout or unexpected resource load appears; inspect the Network tab as well as the page.
3. Refresh before/after hydration; check the safe content remains equivalent.
4. Do not activate external sample links when checking for unsolicited network requests.

## 00013-010 — narrow layout, zoom and motion

Paths: gallery, generated-admin search/create/import, preferences and formatting pages above.

1. At 1280 px, reach every action/error using the keyboard.
2. Repeat at 375 px and 200% zoom; check no action/error is clipped off the page.
3. Scroll a wide table inside its labelled region; check the whole page does not need horizontal scrolling.
4. Keep keyboard focus visible and enable reduced motion; inspect dialogs, notices, validation and pending states again.
5. Inspect collection input/action alignment, wrapped tag chips and labelled Add controls at both widths/zooms. Icons should be recognizably Lucide; focus is visible and actions remain reachable.
6. The interactive rating uses stars and native radio keyboard behavior. Choose 2 then use arrow keys/Space, choose the small 0 control, and Clear rating: 0 and Unrated are distinct. The role catalogue score also admits 0. Fields whose minimum excludes 0 must omit that control (the automated minimum-two fixture covers this case). Disabled controls cannot change values.
7. Typed nested rows is absent from the owner gallery; its four-state/raw-token demo is retained only in the automated collection proof fixture. A nullable-boolean product control is deferred.

## 00013-011 — installed iOS app

Paths: packaged `index.html` / Profile UI; no HTTP URL. The coordinator follows the exact build, launch, restart and stop commands in [Installed iOS review](../../../../examples/review-preview/README.md#installed-ios-review). Start the plugin installed session; the disabled session follows for 013.

1. Have the coordinator build and launch the plugin debug review app for 1800 seconds. Use the owned simulator's primary / LCD 1 display. The web preview can be stopped; this app uses bundled assets.
2. In Profile UI, search for `abc456`, open Grace Hopper, then return to the list. Check navigation/results remain usable.
3. Above Profile UI, choose Check fixed denial; expect “Denied: the fixed extra-field search request was refused.” Choose Start cancellation check, then Cancel review read during the four-second delay. Expect “Cancelled: native read returned cancelled.” If it completes first or shows uncertainty, repeat and record the actual outcome; neither is a cancellation success.
4. Select Français and Thème sombre. Have the coordinator issue `--control restart` for the same plugin session; check French/dark apply on the first render. Restart keeps the same disposable app container.
5. Start another delayed review read, background the app using the simulator's Home gesture, then reopen it. Check usable recovery and repeat the cancel check. Finish with `--control stop` after checking the plugin catalog in 013; the launcher must report owned device/display cleanup. Deadline and Ctrl-C also clean up.
6. Record iOS evidence separately, including restart/background behavior and cleanup. Android remains unverified by owner decision. These steps still require a real coordinator-run mobile build/launch; the implementer's Chromium proof is not installed acceptance.

## 00013-012 — progressive streaming and recovery

Paths: `/ui-stream/stream`, `/ui-stream/late`, `/ui-stream/pending`, `/review`.

1. Open the ordinary streamed page; read the useful shell before the deliberately delayed details appear.
2. Wait for the normal project summary and use the counter; check loading/data is not duplicated. The separate Safety samples section contains deliberately hostile text, displayed inertly.
3. Open Pending and immediately use browser Back, before its two-second idle deadline. Refresh the review panel and check that cleaned stream roots increased. Browser navigation alone is not resource-cleanup evidence.
4. Open Late failure and leave it open for at least six seconds. Expect a visible alert and Reload this page link: “This page didn't finish loading” if a failure marker/parsing end is observable, otherwise the honest “This page hasn't finished loading”. The shell remains readable, counter inactive, with no success or hydrated partial page.
5. Open Pending again and leave it open past its deadline and six-second guidance timer; expect the same safe recovery. Reload points at the same URL and does not retry automatically.
6. Use browser Back or `/review` → Delayed stream to return to the ordinary case and check it works again. Reload on the deliberately failing URL repeats that failure.

## 00013-013 — compiled plugin replacement/removal

Paths: `/generated-admin/roles`, `/generated-admin/profile/create`; packaged `admin-review.html` in the explicit debug `review-preview` bundle.

1. During the main session, choose Pause hydration and record the review-owned “Full name” label on the role catalog and Profile create form; start browser controls and compare it again. This disabled review bundle uses an isolated review-only schema label and is built after removing the plugin. Normal automated generation/removal proofs keep the hostile label and verify escaped output and restored files.
2. Stop the main session before rebuilding. Have the coordinator run the benchmark-recorded `review.py bundles` command described in the [preview README](../../../../examples/review-preview/README.md); it compiles with the plugin, removes it and rebuilds disabled artifacts through the existing disposable generation helpers.
3. Start the matching plugin gateway using the receipt's command. Use Pause hydration to read “Plugin field label” on the role catalog before JavaScript; hydrate and edit its field, then inspect the matching Profile create form.
4. In 011's plugin installed session, open Generated admin review above Profile UI. Expect Plugin field label, matching the plugin web session. Edit its local field, inspect Safety samples and leave Debug state (review only) collapsed. The ordinary catalogue must contain no raw JSON state dump; hostile Rows-style labels belong only to Safety samples. Return to Profile UI using the local link.
5. Stop that installed session and wait for cleanup. Have the coordinator set `IOS_REVIEW_VARIANT=disabled` and repeat the README's mobile build/launch commands. Open Generated admin review and check Full name returns, matching the main web session. Stop the disabled session and check cleanup. No third web session is required.
6. On roles, inspect the styled catalogue sections, fields and locally scrolling table. Empty list values read None and populated lists are comma-separated; draft values stay exact. The separate Safety samples section labels the nullable/default/hostile proofs.
7. Confirm this is a compiled selection, not the role selector. The packaged role catalog has local fixture drafts; it does not provide native admin CRUD.

## Optional extra checks

The automated proofs cover CSR and the mounted base. If useful, stop the assigned devmetrics port, relaunch with `--render csr` or `--base /demo`, and prepend `/demo` to the listed paths for the latter. Use a fresh browser profile for `/demo`. These are optional; the owner need not repeat the full walkthrough.
