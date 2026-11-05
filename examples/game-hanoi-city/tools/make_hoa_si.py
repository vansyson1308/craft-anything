"""Builds HOA_SI.md: everything the art agent needs in one file, gathered from the sources, so a
person hands over a single file. Edit the sources, then run this again; never edit HOA_SI.md.

Sources (in this folder's parent):
- AGENT_PROMPT_05.md: what to make now;
- AGENT_PROMPT_02.md: section A (standard technique) and section B (notes for remaking old pieces);
- AGENT_PROMPT_04.md: technique for pieces used many times;
- ASSETS.md: the full specification.

Usage: python3 -I tools/make_hoa_si.py   (from examples/game-hanoi-city)
Standard library only."""

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def read(name):
    return (ROOT / name).read_text(encoding="utf-8")


def section(text, start, end=None):
    """The lines from the heading that starts with `start` up to (not including) the one with `end`."""
    lines = text.splitlines()
    try:
        i = next(k for k, l in enumerate(lines) if l.startswith(start))
    except StopIteration:
        sys.exit(f"make_hoa_si: no section starting {start!r}")
    j = len(lines)
    if end is not None:
        j = next((k for k in range(i + 1, len(lines)) if lines[k].startswith(end)), len(lines))
    return "\n".join(lines[i:j]).strip("\n")


def demote(text, by=0):
    """Pushes every Markdown heading down `by` levels (none by default: the parts are top-level headings
    and the sources start at the second level)."""
    out = []
    for l in text.splitlines():
        if l.startswith("#"):
            l = "#" * by + l
        out.append(l)
    return "\n".join(out)


def must_replace(text, old, new):
    if old not in text:
        sys.exit(f"make_hoa_si: {old[:50]!r} not found; update this script")
    return text.replace(old, new)


def main():
    brief = read("AGENT_PROMPT_05.md")
    # The brief's body starts after its "---" line.
    body = brief.split("\n---\n", 1)[1].strip("\n")
    body = must_replace(body, "(mục A của\n   AGENT_PROMPT_02)", "(Phần 2)")
    body = must_replace(body, "(DESIGN.md, giai đoạn 3)", "(kể chuyện bằng môi trường)")

    p02 = read("AGENT_PROMPT_02.md")
    technique = section(p02, "## A.", "## B.")
    technique = technique.replace("## A. Nguyên nhân gốc và kỹ thuật chuẩn (áp dụng cho mọi món)", "## Kỹ thuật chuẩn (áp dụng cho mọi món)")
    remake = section(p02, "## B.", "## C.")
    remake = remake.replace("## B. Làm lại năm món của lô 01", "## Ghi chú khi làm lại các món cũ")
    remake = must_replace(remake, "theo mục A.5", "theo Phần 2, ý 5")
    repeat = section(read("AGENT_PROMPT_04.md"), "## Kỹ thuật cho đồ lặp nhiều lần")
    repeat = must_replace(repeat, "như mục A của AGENT_PROMPT_02", "như Phần 2")
    repeat = must_replace(repeat, "build như mục A.7 của AGENT_PROMPT_02", "build như Phần 2, ý 7")
    repeat = must_replace(repeat, "Đợt đầu nên là mục 1 và 2: chỉ riêng chúng đã đổi hẳn cảm giác của phố.", "Với đồ phố, đợt đầu nên là xe máy dựng và bàn ghế (Phần 1, mục C1–C2): chỉ riêng chúng đã đổi hẳn cảm giác của phố.")

    spec = read("ASSETS.md")
    # Drop the spec's own title line.
    spec = spec.split("\n", 1)[1].strip("\n")

    doc = f"""# Gửi họa sĩ: mọi thứ cần để làm tài nguyên cho Hanoi City: Giải Cứu Cún

Một file duy nhất: dán toàn bộ cho agent làm tài nguyên. Gồm bốn phần:

1. **Việc cần làm bây giờ** (lô 05): những gì còn thiếu để game hoàn chỉnh, theo thứ tự người chơi gặp.
2. **Kỹ thuật chuẩn**: cách làm để hình trông thật, không như đồ nhựa.
3. **Ghi chú khi làm lại các món cũ** và **kỹ thuật cho đồ lặp nhiều lần**.
4. **Đặc tả kỹ thuật đầy đủ**: tên file, kích thước, khung xương, cách nộp.

Repo: `vansyson1308/craft-anything`, thư mục `examples/game-hanoi-city/`. Game chạy thử:
https://vansyson1308.github.io/craft-anything/

<!-- Tệp này được ghép tự động bằng tools/make_hoa_si.py; sửa ở các file nguồn. -->

# Phần 1. Việc cần làm bây giờ (lô 05)

{demote(body)}

# Phần 2. {technique.lstrip('# ')}

{demote(technique.split(chr(10), 1)[1].strip(chr(10)))}

# Phần 3. Ghi chú khi làm lại và đồ lặp nhiều lần

{demote(remake)}

{demote(repeat)}

# Phần 4. Đặc tả kỹ thuật đầy đủ

Trong phần này, "mục N" là mục N của Phần 4.

{demote(spec)}
"""
    (ROOT / "HOA_SI.md").write_bytes(doc.encode("utf-8"))
    print(f"make_hoa_si: HOA_SI.md, {len(doc.splitlines())} lines")


if __name__ == "__main__":
    main()
