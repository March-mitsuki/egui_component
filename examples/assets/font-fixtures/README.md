# Font test fixture

`BlackOpsOne-latin.woff2` is an unmodified Black Ops One Latin font downloaded from
<https://fonts.gstatic.com/s/blackopsone/v21/qWcsB6-ypo7xBdr6Xshe96H3aDvbtw.woff2>.
It is used to test the Gallery's WOFF2 download conversion without requiring network access.
See `OFL.txt` for the SIL Open Font License supplied by the Google Fonts repository:
<https://github.com/google/fonts/tree/main/ofl/blackopsone>.

## CJK collection fixture

`GalleryCJK-Test.ttc` is a renamed subset of `../NotoSansCJK-Regular.ttc`.
Face 0 contains `A` (U+0041) and no Chinese glyph; face 1 contains `中` (U+4E2D).
It preserves offline coverage of late fallback loading, removal, and nonzero TTC
face selection without packaging the full 18.6 MiB font. The full font is only a
source/download asset for the gallery and is excluded from the published crate.
See `GalleryCJK-OFL.txt` for the source font's copyright and OFL license.

Regenerate from the repository root:

```sh
uv run --with fonttools==4.60.1 python scripts/make_cjk_fixture.py
```
