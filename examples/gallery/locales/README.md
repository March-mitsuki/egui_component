# Gallery translations

The gallery supports `en`, `zh` (Simplified Chinese), and `ja`. Each JSON file is a
flat object. Keys use `namespace:camelCase`; dots may subdivide a namespace:

```json
{
  "common:livePreview": "Live preview",
  "demo.button:variants": "Variants"
}
```

- Edit values, keeping keys identical across all three languages.
- Keep named placeholders such as `{label}` or `{path}` unchanged. You may move
  them within a sentence. Values substituted into placeholders stay literal.
- Do not translate API names, URLs, route slugs, section IDs, or code snippets.
  Example names and raw technical diagnostics may also remain unchanged.
- New messages need a descriptive key and a value in every language. Duplicate
  keys, missing keys, empty values, and different placeholders fail validation.
- Resources are bundled at build time. Rebuild to see changes.
- Missing translations fall back to English; unknown keys are logged and shown
  verbatim to help identify the missing entry.

Run `cargo test --example gallery` from the repository root to validate resources,
routes, and gallery behavior.

Routes start with `/en`, `/zh`, or `/ja`, for example `/ja/components/button`.
Unprefixed links default to English and are replaced with canonical prefixed URLs.
Language changes preserve page and query parameters. No language preference is
persisted in this version. Web hosting must serve `index.html` for page requests
(Trunk's SPA fallback is already enabled).
