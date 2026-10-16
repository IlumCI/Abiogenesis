"""Strip enwik8 to article prose, keeping case and punctuation (wikifil.pl without the
lowercase / letters-only step). Usage: strip_wiki.py IN OUT"""
import re
import sys

src = open(sys.argv[1], "rb").read().decode("utf-8", "replace")
out = []
for m in re.finditer(r"<text[^>]*>(.*?)</text>", src, re.S):
    t = m.group(1)
    if t.lstrip().upper().startswith("#REDIRECT"):
        continue
    for a, b in (("&lt;", "<"), ("&gt;", ">"), ("&quot;", '"'), ("&amp;", "&")):
        t = t.replace(a, b)
    t = re.sub(r"<!--.*?-->", "", t, flags=re.S)
    t = re.sub(r"<ref[^>]*/>", "", t)
    t = re.sub(r"<ref.*?</ref>", "", t, flags=re.S)
    t = re.sub(r"<math.*?</math>", "", t, flags=re.S)
    t = re.sub(r"<[^>]*>", "", t)
    for _ in range(6):  # nested templates and tables, innermost first
        t2 = re.sub(r"\{\{[^{}]*\}\}", "", t)
        t2 = re.sub(r"\{\|[^{}]*?\|\}", "", t2, flags=re.S)
        if t2 == t:
            break
        t = t2
    t = re.sub(r"\[\[(?:[a-z\-]{2,12}|Category|Image|File|Wikipedia):[^\]]*\]\]", "", t)
    t = re.sub(r"\[\[[^\]|]*\|([^\]]*)\]\]", r"\1", t)
    t = re.sub(r"\[\[([^\]]*)\]\]", r"\1", t)
    t = re.sub(r"\[https?://\S+ ([^\]]*)\]", r"\1", t)
    t = re.sub(r"\[?https?://\S+\]?", "", t)
    t = re.sub(r"&#(\d+);", lambda m: chr(int(m.group(1))) if int(m.group(1)) < 0x110000 else "", t)
    t = re.sub(r"&[a-z]+;", " ", t)
    t = re.sub(r"'{2,}", "", t)
    t = re.sub(r"^=+\s*(.*?)\s*=+\s*$", r"\1", t, flags=re.M)
    t = re.sub(r"^[*#:;]+\s*", "", t, flags=re.M)
    t = re.sub(r"[ \t]+", " ", t)
    t = re.sub(r"\n\s*\n+", "\n", t)
    t = t.strip()
    if t:
        out.append(t)
open(sys.argv[2], "wb").write("\n".join(out).encode("utf-8"))
