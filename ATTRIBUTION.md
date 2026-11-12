# Third-party assets and data

This inventory covers vendored assets, derived data and their compiled copies in
the Rust crates and examples. Licence texts remain in the linked existing files;
this index does not change the repository's or any crate's licence. Registry
dependencies are inventoried separately by the
[dependency licence check](tools/release/PACKAGING.md#dependency-licence-snapshot).

## Lucide and inherited Feather geometry

All three bundles pin **Lucide 1.62.0**, commit
`03f11e2d6adc319ea21e9a1d5820a99cd3767257`, from
[lucide-icons/lucide](https://github.com/lucide-icons/lucide/tree/03f11e2d6adc319ea21e9a1d5820a99cd3767257).
Lucide geometry uses **ISC**; the inherited Feather geometry identified in each
bundle's upstream LICENSE uses **MIT**. These are the SPDX identifiers for the
licences named in the retained texts; consult those texts for the per-icon split.

| Shipped paths | What it is | Existing notices and licence texts |
| --- | --- | --- |
| `crates/leptos/third-party/lucide/*.svg` (44 masters); compiled geometry in `crates/leptos/src/components/icons.rs` and embedded toolbar SVGs in `crates/leptos/src/components/text_editor/toolbar/icons.rs` | Row, tag, rating, source-role and text-editor icons | [NOTICE](crates/leptos/third-party/lucide/NOTICE), [LICENSE](crates/leptos/third-party/lucide/LICENSE), [FEATHER-LICENSE](crates/leptos/third-party/lucide/FEATHER-LICENSE) |
| `crates/framework/framework-session/third-party/lucide/*.svg` (16 masters); compiled geometry in `crates/framework/framework-session/src/views/icons.rs` | Account, authentication and preference icons | [NOTICE](crates/framework/framework-session/third-party/lucide/NOTICE), [LICENSE](crates/framework/framework-session/third-party/lucide/LICENSE), [FEATHER-LICENSE](crates/framework/framework-session/third-party/lucide/FEATHER-LICENSE) |
| `crates/framework/framework-api/third-party/lucide/*.svg` (19 masters); compiled geometry in `crates/framework/framework-api/src/views/icons.rs` | Consent and management icons | [NOTICE](crates/framework/framework-api/third-party/lucide/NOTICE), [LICENSE](crates/framework/framework-api/third-party/lucide/LICENSE), [FEATHER-LICENSE](crates/framework/framework-api/third-party/lucide/FEATHER-LICENSE) |

## Frui-derived presentation and data

Source project: Frui. The retained Frui licence
is **MIT**. A revision below is supplied only where existing provenance records it.

| Shipped paths | What it is and source | Existing notice and licence text |
| --- | --- | --- |
| `crates/leptos/src/components/core.css` (switch rules), `crates/leptos/src/components/controls.rs` (native switch) | Default switch appearance adapted from Frui `src/form/Switch.tsx` and `styles/form/switch.css`; commit `096fd14580f0f49b6e159fa0aa5ae2a5bce8fb0e` is recorded in the notice | [NOTICE](crates/leptos/third-party/frui/NOTICE), [LICENSE](crates/leptos/third-party/frui/LICENSE) |
| `crates/leptos/src/components/core.css` (editor rules), `crates/leptos/src/components/source/styles.css` | Native CSS implementing Frui-compatible editor/source controls, including Frui text-style classes and radio appearance. These adaptations have no separately recorded revision in the existing switch notice | [Retained Frui notice](crates/leptos/third-party/frui/NOTICE), [LICENSE](crates/leptos/third-party/frui/LICENSE) |
| `crates/leptos/src/components/source/data.rs`, consumed by `source/countries.rs` and `source/color_input.rs` | Country, currency and named-colour arrays ported from Frui `src/data/{countries,currencies,colors}.ts`, preserving source order and records. The Rust data file records each original file's SHA-256; the switch notice's commit is not a separate data revision claim | [Data provenance](crates/leptos/src/components/source/data.rs), [retained Frui notice](crates/leptos/third-party/frui/NOTICE), [LICENSE](crates/leptos/third-party/frui/LICENSE) |
| `crates/framework/framework-session/src/views/countries.rs` | Country/dial menu derived from Frui `src/data/countries.ts`, retaining source order, ISO2, names and telephone codes; no revision is recorded in this notice | [NOTICE](crates/framework/framework-session/third-party/frui/NOTICE), [LICENSE](crates/framework/framework-session/third-party/frui/LICENSE) |

The Frui directories contain the retained notice/licence files for these adapted
assets; they do not contain a second copy of the upstream source tree.

## Nodemailer data

Source project: [Nodemailer](https://github.com/nodemailer/nodemailer), version
**7.0.13**, as recorded in the existing data README. Licence: **MIT**, as stated by
the retained package licence text.

| Shipped path | What it is | Existing provenance and licence text |
| --- | --- | --- |
| `crates/framework/framework-email/data/nodemailer-services.json` | Verbatim `lib/well-known/services.json` service presets | [README](crates/framework/framework-email/data/README.md), [nodemailer-LICENSE](crates/framework/framework-email/data/nodemailer-LICENSE) |
| `crates/framework/framework-email/data/nodemailer-mime.json` | `lib/mime-funcs/mime-types.js` MIME type/extension maps projected to JSON | [README](crates/framework/framework-email/data/README.md), [nodemailer-LICENSE](crates/framework/framework-email/data/nodemailer-LICENSE) |

## Original repository assets

- Original: blog `brand.svg`, `global.css` and `article.css` under
  [examples/blog/plugins/assets/assets](examples/blog/plugins/assets/assets).
  The SVG is the authored SP badge; the CSS records that it is authored for the
  Rust template. No upstream generated public tree or font files are copied.
- Original: [API checkpoint application logo](examples/api-checkpoint/proofs/application-logo.svg),
  the plain [Profile native icon](examples/profile-ui/native/icons/icon.png),
  [generated-admin role CSS](examples/generated-admin/shared/src/roles.css) and
  [UI-preferences CSS](examples/ui-preferences/shared/style.css).
- Original: remaining native component/source-role CSS under
  `crates/leptos/src/components/` and framework session/API view CSS under
  `crates/framework/*/src/views/`, except the Frui adaptation identified above.
- Original: test-only TLS identities in `crates/server/http/tests/fixtures/`
  ([generation provenance](crates/server/http/tests/fixtures/README.md)) and
  `crates/framework/framework-email/tests/fixtures/tls-{cert,key}.pem`, plus the
  HTTP example's `crates/server/http/examples/assets/hello.txt`.

No vendored font files are present in the tracked crate/example assets. System
font names and Unicode/emoji characters select platform resources rather than
shipping font binaries. Generated application/model fixtures and source-oracle
vectors are repository development material, not additional vendored media.
The preserved upstream KB and installed agent-skill material under `.agents/`
retain their separate [source/provenance catalog](.agents/context/source-catalog.md).
