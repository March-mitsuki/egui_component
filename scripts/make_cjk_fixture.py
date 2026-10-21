"""Generate a tiny two-face TTC for font-manager tests.

Run from the repository root:
    uv run --with fonttools==4.60.1 python scripts/make_cjk_fixture.py

The full source font remains a repository/gallery download asset, not a crate asset.
"""
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTCollection, TTFont

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "examples/assets/NotoSansCJK-Regular.ttc"
OUTPUT = ROOT / "examples/assets/font-fixtures/GalleryCJK-Test.ttc"


def face(index, character, family):
    font = TTFont(SOURCE, fontNumber=index, recalcTimestamp=False)
    options = subset.Options()
    options.name_IDs = [0, 1, 2, 3, 4, 5, 6, 13, 14]
    options.name_languages = [0x409]
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=[ord(character)])
    subsetter.subset(font)
    # Give the modified font its own names, including CFF metadata.
    for record in list(font["name"].names):
        if record.nameID in (1, 3, 4, 6):
            name = family.replace(" ", "") if record.nameID == 6 else family
            font["name"].setName(name, record.nameID, record.platformID, record.platEncID, record.langID)
    if "CFF " in font:
        cff = font["CFF "].cff
        cff.fontNames = [family.replace(" ", "")]
        top = cff.topDictIndex[0]
        top.FamilyName = family
        top.FullName = family
    return font


def main():
    collection = TTCollection()
    # Index 0 deliberately lacks Chinese; index 1 provides the tested glyph.
    collection.fonts = [face(0, "A", "Gallery Fixture Latin"), face(1, "中", "Gallery Fixture CJK")]
    collection.save(OUTPUT)
    with TTFont(SOURCE, fontNumber=1) as source:
        copyrights = sorted({name.toUnicode() for name in source["name"].names if name.nameID == 0})
    # Reuse the standard OFL text, with the source font's actual copyright notice.
    standard = (OUTPUT.parent / "OFL.txt").read_text().split("This Font Software", 1)[1]
    (OUTPUT.parent / "GalleryCJK-OFL.txt").write_text(
        "\n".join(copyrights) + "\n\nThis Font Software" + standard,
        encoding="utf-8",
    )
    print(f"{OUTPUT.name}: {OUTPUT.stat().st_size} bytes")


if __name__ == "__main__":
    main()
