# Slice 8 owner review checklist

Status: walkthrough for the [approved review preview](00013-review-preview-design.md). Owner acceptance is recorded separately. Installed steps (011 and the installed half of 013) wait for Part B.

Use two sessions only: first the main root / hydrate / disabled session for 001–010, 012 and the disabled half of 013, then the compiled plugin session for the plugin half of 013. Open `http://127.0.0.1:<assigned-port>/review`, replacing the port with the one reported by devmetrics; this host uses that exact origin. Keep `/review` in a second tab for its labelled caller/failure fixtures; select Editor to begin. These fixtures are not a login system. Record pass/fail and observations per control.

## 00013-001 — useful content before JavaScript

Paths: `/review`, `/ui-gallery/gallery`, `/generated-admin/profile/search`.

1. Choose Editor and Pause hydration in the review panel; open each page in the hydrate session.
2. Check that labels, gallery content and generated rows are readable before starting browser controls.
3. Type in an input, focus it, select some text and scroll the page.
4. Start browser controls; check that the value, selection, focus and scroll remain and content does not flash or duplicate.
5. Restore automatic hydration in the review panel.

## 00013-002 — keyboard dialogs

Path: `/ui-gallery/gallery`.

1. Use Tab and Enter to open the profile dialog; edit its input without the mouse.
2. Tab and Shift-Tab around it; check that focus stays inside and remains visible.
3. Open/close the nested dialog, then use Escape on the ordinary dialog; check focus returns to its opener.
4. Open the explicit-close dialog; verify Escape follows its stated policy and its Close button works.
5. Try Remove opener and Unmount dialog; verify focus reaches a usable fallback and the page still works.

## 00013-003 — tables and list state

Paths: `/ui-gallery/gallery`, `/generated-admin/task/search`, `/review`.

1. Read the table caption/headings; filter, sort and move between pages.
2. Open the list's current-state link, refresh and use Back; check that the represented filter/sort/page agrees.
3. Choose Slow next read in the second tab and load again; check useful rows remain while loading.
4. Filter to no matches; check that the empty state explains what to do.
5. Try Failed next read and Denied caller separately; check safe errors/actions, then restore Editor.

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
4. Paste the earlier Create URL directly; check denial. In browser developer tools, repeat a captured write request with a forged `x-admin-actor: editor`; check Reader still cannot write.
5. Switch to Denied and paste a saved detail/related URL; check no private record appears, then restore Editor.

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

1. Select Dark, refresh and watch the initial paint; repeat with Light.
   Ignore the System option: by owner decision G39 it will be removed in a later task, and the OS preference will only choose the initial theme.
2. Check text, inputs, focus and errors remain readable; reopen with the same browser profile and inspect the saved theme.
3. Use the review panel to issue one public notice, then open the preferences page and dismiss it.
4. Interact and refresh; check it does not reappear in the same root. Concurrent-tab cookie delivery remains best effort.

## 00013-009 — inert rich content

Paths: `/ui-gallery/gallery`, `/ui-format/`.

1. Read the malicious-looking HTML/Markdown examples and their safe fallback/code text.
2. Check no popup, hidden form, broken layout or unexpected resource load appears; inspect the Network tab as well as the page.
3. Refresh before/after hydration; check the safe content remains equivalent.
4. Do not activate external sample links when checking for unsolicited network requests.

## 00013-010 — narrow layout, zoom and motion

Paths: gallery, generated-admin search/create/import, preferences and formatting pages above.

1. At 1280 px, reach every action/error using the keyboard.
2. Repeat at 375 px and 200% zoom; check no action/error is clipped off the page.
3. Scroll a wide table inside its labelled region; check the whole page does not need horizontal scrolling.
4. Keep keyboard focus visible and enable reduced motion; inspect dialogs, notices, validation and pending states again.

## 00013-011 — installed iOS app

Paths: packaged `index.html` / Profile UI; no HTTP URL. Coordinator uses `examples/profile-ui/native/proofs/build-ios.py` and the proposed manual mode of `launch-ios.py`.

1. Ask the coordinator to launch its bounded manual review session on the owned simulator with bundled assets; the development preview may be stopped.
2. Navigate, search and cancel using permitted actions; check displayed results and cancellation behavior.
3. Use the labelled debug denial check; inspect the refusal without gaining an extra permission.
4. Save language/theme, restart the app and check the settings persist.
5. Background/resume during pending work and check usable recovery; finish the session so its owned app/device/display are cleaned up.
6. Record iOS evidence separately. Android remains unverified by owner decision.

## 00013-012 — progressive streaming and recovery

Paths: `/ui-stream/stream`, `/ui-stream/late`, `/ui-stream/pending`, `/review`.

1. Open the ordinary streamed page; read the useful shell before the deliberately delayed details appear.
2. Wait for details and use the counter; check loading/data is not duplicated and hostile-looking text stays inert.
3. Open Pending and immediately use browser Back, before its two-second idle deadline. Refresh the review panel and check that cleaned stream roots increased. Browser navigation alone is not resource-cleanup evidence.
4. Open Late failure; wait for its bounded deadline and check safe recovery with no false success or hydrated partial page.
5. Use browser Back or `/review` → Delayed stream to return to the ordinary case and check it works again. Reload on the deliberately failing URL repeats that failure.

## 00013-013 — compiled plugin replacement/removal

Paths: `/generated-admin/roles`, `/generated-admin/profile/create`; packaged `admin-review.html` in the proposed debug review bundle.

1. During the main session, choose Pause hydration and record the restored “Name </script> 🦀” label on the role catalog and Profile create form; start browser controls and compare it again. This disabled bundle was built after removing the plugin; the automated removal proof also checks the restored generated files.
2. Stop the main session before rebuilding. Have the coordinator run `python3 examples/review-preview/review.py bundles --directory /private/tmp/s8-review`; it compiles with the plugin, removes it and rebuilds disabled artifacts through the existing disposable generation helpers.
3. Start the matching plugin gateway using the receipt's command. Use Pause hydration to read “Plugin field label” on the role catalog before JavaScript; hydrate and edit its field, then inspect the matching Profile create form.
4. Rebuild/install the matching debug iOS review bundle; open its local Generated admin review link and compare the same compiled plugin label. This installed step waits for Part B.
5. Rebuild/install the disabled iOS variant and check the original label returns, matching the main session's recorded web label. No third web session is required.
6. Confirm this is a compiled selection, not the role selector. The packaged role catalog has local fixture drafts; it does not provide native admin CRUD.

## Optional extra checks

The automated proofs cover CSR and the mounted base. If useful, stop the assigned devmetrics port, relaunch with `--render csr` or `--base /demo`, and prepend `/demo` to the listed paths for the latter. Use a fresh browser profile for `/demo`. These are optional; the owner need not repeat the full walkthrough.
