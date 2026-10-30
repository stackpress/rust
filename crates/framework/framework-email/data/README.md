# Versioned source transport data

These data tables come from the read-only original workspace's Nodemailer 7.0.13:

- `nodemailer-services.json`: verbatim `lib/well-known/services.json`.
- `nodemailer-mime.json`: `lib/mime-funcs/mime-types.js` maps, projected to JSON
  `{types:Object.fromEntries(mimeTypes),extensions:Object.fromEntries(extensions)}`.
- `nodemailer-LICENSE`: original package license, covering the copied data.

No original application/source file is modified. Tables remain source authority,
not a new allowlist. The resolver preserves unknown service/extension fallbacks.
