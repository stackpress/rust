# Slice 9 auth/profile inventory under G43

Status: Revised design only. **All source behaviours below are now in scope**, including SQL adapters and every account flow. `S/` is upstream packages/stackpress-session/src. [G43 decisions](slice-9-design-decisions.md) supersede the former security redesign. Reasons are source comments or call-flow interpretations; no presumed defect is fixed implicitly.

## Exported auth types

| Every exported source name | Why / source meaning | Rust mapping |
| --- | --- | --- |
| Auth | Persisted id,profileId,type,token,secret,verified,consumed,active,created,updated | Native storage DTO with original fields/defaults; token encryption and secret hashing remain source codec contracts. No replacement session-reference or pending-proof tables. |
| AuthInput | Writable payload with optional id/type/flags/times | Preserve the original payload surface for actions; public events/pages select/override fields as their source does, not a new globally restricted input. |
| AuthColumns | Column identity and validation/codecs | Map all ten columns through existing framework-schema/SQL machinery, preserving defaults/constraints. |
| AuthAssertInterfaceMap, AuthSerializeInterfaceMap, AuthUnserializeInterfaceMap | Per-column validation/serialization lookup | Typed maps/errors/codecs retain meanings; JS interface syntax is mechanics-only non-port. |
| AuthSchemaInterface | Schema shape/default/assert/filter/populate/serialize/unserialize | Native schema implementation with same observable coercions and issues. |
| AuthExtended | Auth plus joined profile | Preserve join and source response fields; secret stripping remains at specific source call sites. |
| AuthRelations | Profile store/local/foreign/cardinality metadata | Existing checked FK/join infrastructure; no new account ownership check beyond each consuming process. |
| AuthStoreInterface, AuthActionsInterface | Direct generic data operations available to the framework | Implement native Store/Actions interfaces and generated-event integration, including auth-search/create/update/remove consumed by pages. Do not replace events with private-only repository calls. |
| ActionOptions {seed?,password?} | Direct action construction separates DB codec seed and password rules | Seed defaults '' in constructor, database.seed fallback abc123 in make(ctx); same rule object, no native-secret-injection requirement. |
| AuthEmailConfig {name,address} | Sender for OTP/link email | Same fields consumed by email flows; missing sender silently stops sending at the original point. |
| AuthPasswordConfig {min,max,upper,lower,number,special} | Optional signup/change validation | Preserve absent rules, source JS UTF-16 length, branch precedence, truthiness and field messages. No 12-character/1024-byte/E.164 policy added. |
| AuthMenuConfig {type?,target?,name,icon?,path} | Signin/footers/navigation presentation | Same configured menu/targets/paths; Leptos and project Lucide ownership replace React/FontAwesome mechanics without adding redirect policy. |
| AuthConfig {base?,redirect?,2fa?,captcha?,email?,roles?,menu?,password?} | Enables routes and provides flow settings | Optional typed section plus extension objects; every source config consumer retained. Placeholder objects remain available to plugins, not an asserted provider implementation. |
| SignupInput {name,type?,username?,email?,phone?,secret,roles} | Can create one profile and several authentication methods | Preserve roles and request data precedence; no rejection of roles/protected-looking fields at a new framework boundary. |
| SigninInput {type?,username?,email?,phone?,secret}; SigninType username/email/phone | Reusable password or passwordless signin action | Retain source dispatch and String(input[type])/Boolean(password); event data flags are mapped in execution contract. |
| AuthConfigProps, AuthPageProps | Original view config/data/request/status | Leptos public snapshot model carries source-selected fields; no new default filter removing a source-intended result. Native implementation resources are not serialized automatically. |

## Exported profile types

| Every exported source name | Why / source meaning | Rust mapping |
| --- | --- | --- |
| Profile; ProfileInput | id,image nullable,name,type,roles,tags,references nullable,active,created,updated | All fields and writable inputs now; retain generic profile purpose (person/company/etc.), not a new restricted user schema. |
| ProfileColumns | Scalar columns and auth relation | Existing schema/SQL support with original field defaults and role metadata. |
| ProfileAssertInterfaceMap, ProfileSerializeInterfaceMap, ProfileUnserializeInterfaceMap | Per-field schema contracts | Typed source-equivalent validation/codecs. |
| ProfileSchemaInterface | Schema operations | Native schema implementation; no empty facade without actual storage. |
| ProfileExtended (Profile alias), ProfileRelations (empty) | Generated interface compatibility | Rust aliases/empty metadata as needed for analogous generated store signatures; do not discard an exported type merely as redundant. |
| ProfileStoreInterface, ProfileActionsInterface | Full direct profile data operations | Implement complete Store/Actions surface, backed by SQL adapter now. |
| ProfileAuth | Profile plus auth map of partial rows | Same signup/account result purpose; retain exactly which fields individual source handlers remove. |

## AuthActions and ProfileActions

| Source methods/properties | Why / current source behaviour | Rust mapping / parity case |
| --- | --- | --- |
| AuthActions.make(ctx); constructor; engine/store/profile; _seed/_password | Shared framework resource lookup/direct use | Existing Engine service and database seed/password config. No eager startup failure when an unused action lacks a database; source resolves dependencies when invoked. |
| AuthActions.assert | Signup identity/rules validation with ordered errors | Name required; at least one identifier; source email regex; required secret; configured min/max/upper/lower/number/special in source order, source messages/case preserved. |
| signin(type,input,password=true), AuthActions.ts:385–425 | Reuses credential lookup for password and passwordless paths | Same 404 {[type]:User Not Found} versus 401 {secret:Invalid Password}; optional password comparison; update consumed then reload joined auth. No dummy hash, verification-policy additions or new revocation. |
| signup, AuthActions.ts:428–506 | One profile with any supplied auth methods | Profile first, then email→phone→username; preserve per-field uniqueness error remapping. Do not add an encompassing transaction, profile cleanup or changed partial-effect/error precedence. |
| batch(inputs,mode=upsert), both classes | Explicit transaction plus per-input status collection | Preserve create/update/upsert selection, id/token lookup, per-item statuses, rollback flag and caught batch failure; rows may describe work later rolled back. No new atomic success envelope. |
| count, find, findAll | Generic filtered queries and relation expansion | Reuse existing SQL query/selector machinery, original selected-column defaults and active-row filter; no new hidden field/tenant/role policy. |
| create | Filter/populate/serialize/unserialize/remove-empty/remove-undefined, validate, token uniqueness and insert | Preserve preflight and SQL unique constraint, returned row/fallback behaviour; global token uniqueness, not per-method index. |
| update | Validate, read selected rows, merge return values, global token uniqueness, execute store update | Preserve original order and input used for write, including no rows→empty and matching-row exception to uniqueness check. Do not silently requery/transactionalize. |
| upsert | id first, otherwise existing nonempty token, otherwise create | Same branch selection/results. |
| delete | Read rows then physical deletion only if rows exist | Preserve returned rows and no-op behaviour, not default denial/confirmation. |
| remove, restore | update active=false/true | Existing soft-delete conventions; no JWT invalidation or challenge generation increment. |
| install, upgrade, purge(cascade=false), uninstall | Explicit store setup/maintenance | Full operations now using SQL; source does not run them from session plugin startup. Keep SQL engine/adapter ownership and existing Rust migration execution limits visible. |
| ProfileActions constructor, engine/store/_seed and all methods above except signin/signup/assert | Generic profile storage/actions | Implement source-equivalent behaviour; no duplication of an unrelated generated-admin demo Profile. |

## Generated support and fields

| Source surface | Why / source behaviour | Rust mapping |
| --- | --- | --- |
| AuthSchema/ProfileSchema: name,columns,shape,defaults,constructor,assert,filter,populate,serialize,unserialize | Generated schemas centralize field validation/defaults | Reuse existing framework-schema with explicit compatibility codecs, not a new ORM or changed validation policy. |
| AuthStore/ProfileStore: table,relations,constructor; alter,create,delete,insert,select,selectors,joins,paths,where,whereEquals,whereLike,whereNotEquals,whereGreaterEquals,whereLessEquals,whereArrayContains,whereArrayNotContains,update,scalarize,toSqlValue,unscalarize | Dialect-aware generated stores expose all required operations | Implement/reuse all capabilities now, preserve selectors/relations/JSON paths and source serialization toggles. |
| Every column: name,shape,defaults,constructor,assert,serialize,unserialize | Field-specific conversion/constraints | Typed column adapters, including String/Date/array/hash coercions; maintain unset versus null and native Date semantics. |
| Auth id=cuid; profileId required; type=username; token required/min5/unique/encrypted; secret required/hashed/secret; verified=false; consumed=now; active=true; created/updated=now | Persist credential/lookup/status information | Same storage/semantic constraints. AuthStore.where defaults active=true, explicit active=-1 removes that filter; do not add a separate profile-active requirement to signin beyond its actual query/join. |
| Profile id=cuid; image optional; name required/searchable; type=person; roles=[GUEST]; tags=[]; references={}/nullable; active=true; created/updated=now | Generic entity profile and role grants | Same fields/defaults, trusted/application or request roles as source actions select. |
| schema.idea Profile.auth/Auth.profile; Profile.applications/Profile.sessions | Relationship composition with the API package | Auth/Profile and SQL adapter now; task 00021 owns Application/API Session definitions and its SQL adapter now in that parallel slice. Reconcile references without inventing JWT persistence. |
| @encrypted TokenColumn and @hashed SecretColumn | Deterministic lookup/encrypted identifier and password comparison | Preserve source SHAKE256(32-byte hex) and seed-derived AES-256-CBC contract from framework-schema/helpers.ts:64–108. Approved Argon2 does not authorize changing these bytes/verification paths. Provider fit is an M0 mechanics gate, not a default format migration. |
| @labels/display/icon, field/filter/list/view/span attributes, descriptions/examples | Generated UI metadata | Full field presentation intent now via current schema/Leptos system. Do not alter generic action exposure because a field looks sensitive; preserve source @secret handling and specific page stripping. |

## Helpers, events and extension hooks

| Every remaining relevant source item | Why / mapping now |
| --- | --- |
| isSpecialChars, isEmail | Preserve server regex predicates/messages; source special includes whitespace. Browser passwordChecklist uses a different predicate and weighted score; retain it as presentation behaviour, not a unified new rule. JS regex compatibility is a mechanics proof gate. |
| normalizePhone | Trim; retain one leading +; strip other nondigits; empty/no digit→undefined. No country inference/E.164 validation or global email/username canonicalization. Optional-input mismatch is Q-DEF1, not fixed. |
| base32Decode, generateSecret, generateTOTP, verifyTOTP | Original base32 upper/padding/truncated-bit handling, 16-character alphabet secret, SHA1/six digits/30s/±1 window, ordinary source comparison and no replay table. Use approved library where functionally equivalent; do not import a stricter default validator or step-consumption policy. |
| SigninEmailTemplateInput, makeOTPTemplate, makeMagicLinkTemplate | Preserve template text/subject/greeting fallback and HTML-escaping inputs before shared template expansion, including source plain-text escaped values. Native engine/Leptos mechanics may replace the JS evaluator, not the output contract. |
| auth-signup | Config roles spread first, request data overrides; normalize phone; call signup; fire email-email-send/auth-phone-verify without awaiting; strip each existing email/phone/username secret, return results. Keep hook names/order and nonblocking intent; supervise detached Rust work mechanically, not await/rename it. |
| auth-signin | Source flags, 2FA search/challenge redirect or token creation; remove results.secret at source point, profile roles fallback only when nonarray/empty. No mandatory current password outside source page/action choices. |
| auth-signout | Literal res.session.delete('session'), status 200. Do not change to configured key or add DB revocation. |
| auth-* and profile-* generated events used by pages | Available shared capabilities now; preserve resolve/emit and response/status composition. Do not hide them as private-only calls to enforce new authority. |
| email-email-send, auth-phone-verify, captcha/2fa extension objects | Existing named extension points now, with exact calls/config. No SMS/CAPTCHA provider code exists to port; provide hook contracts and tests, not a new mandatory provider or later deferral of current flows. |
