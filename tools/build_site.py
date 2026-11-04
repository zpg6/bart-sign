"""Build the GitHub Pages site, docs/index.html, from docs/guide.md.

    python3 tools/build_site.py          # rewrite docs/index.html
    python3 tools/build_site.py --check  # exit 1 if docs/index.html is stale

guide.md is the source of truth. This script only wraps it in the page
template, so edit the guide and rerun it. Standard library only.

It understands the Markdown that guide.md uses: ATX headings, paragraphs,
pipe tables, "-" and "1." lists with indented continuation blocks, fenced
code, HTML comments, and inline code, links, images, **bold** and *italic*.
"""
import html
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUIDE = ROOT / "docs" / "guide.md"
OUT = ROOT / "docs" / "index.html"
SITE_URL = "https://zpg6.github.io/bart-sign/"
REPO_URL = "https://github.com/zpg6/bart-sign"

# BART's own line colors (hexcolor in its API), used for section markers.
LINES = ["yellow", "orange", "red", "green", "blue"]


# ---------------------------------------------------------------- inline

def slug(text):
    """GitHub's heading anchor: lowercase, drop punctuation, spaces to '-'."""
    s = re.sub(r"<[^>]+>", "", text).strip().lower()
    s = re.sub(r"[^\w\- ]", "", s)
    return s.replace(" ", "-")


def plain(md):
    """Inline Markdown to plain text (for slugs, alt text, word counts)."""
    s = re.sub(r"!\[([^\]]*)\]\([^)]*\)", r"\1", md)
    s = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", s)
    return re.sub(r"[*`]", "", s)


def inline(md):
    codes = []

    def keep_code(m):
        text = m.group(1)
        cls = ' class="nw"' if " " not in text and len(text) <= 28 else ""
        codes.append(f"<code{cls}>" + html.escape(text, quote=False) + "</code>")
        return f"\x00{len(codes) - 1}\x00"

    s = re.sub(r"`([^`]+)`", keep_code, md)
    s = html.escape(s, quote=False)
    s = re.sub(r"!\[([^\]]*)\]\(([^)\s]+)\)",
               lambda m: f'<img src="{m.group(2)}" alt="{html.escape(m.group(1))}">', s)

    def link(m):
        text, href = m.group(1), m.group(2)
        ext = href.startswith("http")
        attrs = ' rel="noopener"' if ext else ""
        return f'<a href="{href}"{attrs}>{text}</a>'

    s = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", link, s)
    s = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", s)
    s = re.sub(r"(?<![\w*])\*(?!\s)(.+?)(?<!\s)\*(?![\w*])", r"<em>\1</em>", s)
    return re.sub(r"\x00(\d+)\x00", lambda m: codes[int(m.group(1))], s)


# ---------------------------------------------------------------- figures

IMG_RE = re.compile(r'!\[([^\]]*)\]\(([^)\s]+)(?:\s+"([^"]*)")?\)')


def is_sign(src):
    """480x320 screen renders: framed like the hanging sign housing."""
    return "/screens/" in src or src.endswith(("boot.gif", "boot-final.png"))


def sign_figure(alt, src, hero=False):
    cls = "sign sign--hero" if hero else "sign"
    loading = "" if hero else ' loading="lazy"'
    return (f'<figure class="{cls}"><div class="sign__hood"></div>'
            f'<div class="sign__housing"><img src="{src}" alt="{html.escape(alt)}" '
            f'width="480" height="320"{loading}></div>'
            f'<figcaption aria-hidden="true">{html.escape(alt)}</figcaption></figure>')


def is_photo(src):
    """Camera photos (the real signs, the finished build): shown as photos."""
    return "/photos/" in src or "/reference/" in src


def photo_figure(alt, src, caption=None):
    """A photo at its own proportions, with a visible caption (e.g. credit)."""
    cap = inline(caption) if caption else html.escape(alt)
    return (f'<figure class="photo"><img src="{src}" alt="{html.escape(alt)}" loading="lazy">'
            f'<figcaption>{cap}</figcaption></figure>')


def sheet_figure(alt, src):
    """Slides and contact sheets: large, so they link to the full image."""
    return (f'<figure class="sheet"><div class="sheet__pan" tabindex="0" '
            f'role="group" aria-label="{html.escape(alt)}, scrollable"><a href="{src}">'
            f'<img src="{src}" alt="{html.escape(alt)}" loading="lazy"></a></div>'
            f'<figcaption>{html.escape(alt)} <a href="{src}">Full size</a>'
            f'<span class="sheet__hint" aria-hidden="true">Swipe to pan</span></figcaption></figure>')


def figures(lines):
    imgs = [IMG_RE.fullmatch(l.strip()).groups() for l in lines]
    parts = [sign_figure(a, s) if is_sign(s) else photo_figure(a, s, t) if is_photo(s)
             else sheet_figure(a, s) for a, s, t in imgs]
    if len(parts) > 1 and all(is_sign(s) for _, s, _ in imgs):
        return '<div class="sign-pair">' + "".join(parts) + "</div>"
    if len(parts) > 1 and all(is_photo(s) for _, s, _ in imgs):
        return '<div class="photo-pair">' + "".join(parts) + "</div>"
    return "\n".join(parts)


# ---------------------------------------------------------------- LED text

def led(lines_html, block=False):
    """Text the sign itself shows, set like the sign: red on black, one span
    per sign line. The " / " separators stay in the text for copy and
    screen readers."""
    sep = '<span class="vh"> / </span>'
    inner = sep.join(f'<span class="led-msg__line">{l}</span>' for l in lines_html)
    tag = "div" if block else "span"
    return f'<{tag} class="led-msg">{inner}</{tag}>'


# ---------------------------------------------------------------- tables

def cells(row):
    return [c.strip() for c in row.strip().strip("|").split("|")]


def table(lines):
    head = cells(lines[0])
    body = [cells(r) for r in lines[2:]]
    cls = "t-" + slug(head[0])
    out = [f'<div class="table-wrap"><table class="{cls}"><thead><tr>']
    out += [f"<th scope=\"col\">{inline(h)}</th>" for h in head]
    out.append("</tr></thead><tbody>")
    for row in body:
        total = plain(row[0]).strip().lower() == "total"
        out.append('<tr class="total">' if total else "<tr>")
        for k, (h, c) in enumerate(zip(head, row)):
            label = html.escape(plain(h))
            if cls == "t-message" and k == 0:
                body_html = led([inline(part) for part in c.split(" / ")])
            else:
                body_html = inline(c)
            out.append(f'<td data-label="{label}">{body_html}</td>')
        out.append("</tr>")
    out.append("</tbody></table></div>")
    return "".join(out)


# ---------------------------------------------------------------- blocks

def code_block(lang, body):
    if not lang and body and all(l == l.upper() for l in body):
        return led([html.escape(l, quote=False) for l in body], block=True)
    cls = f' class="language-{lang}"' if lang else ""
    return f"<pre><code{cls}>{html.escape(chr(10).join(body), quote=False)}</code></pre>"


LIST_RE = re.compile(r"^(-|\d+\.) ")


def blocks(lines):
    """Markdown lines to a list of (kind, html, meta) blocks."""
    out, i = [], 0
    while i < len(lines):
        line = lines[i]
        s = line.strip()
        if not s:
            i += 1
        elif s.startswith("<!--"):
            j = i
            while "-->" not in lines[j]:
                j += 1
            out.append(("comment", "\n".join(lines[i:j + 1]), None))
            i = j + 1
        elif s.startswith("```"):
            lang, body, i = s[3:].strip(), [], i + 1
            while not lines[i].strip().startswith("```"):
                body.append(lines[i])
                i += 1
            out.append(("code", code_block(lang, body), None))
            i += 1
        elif m := re.match(r"^(#{1,6}) (.*)$", line):
            out.append(("h", m.group(2).strip(), len(m.group(1))))
            i += 1
        elif s.startswith("|"):
            j = i
            while j < len(lines) and lines[j].strip().startswith("|"):
                j += 1
            out.append(("table", table(lines[i:j]), None))
            i = j
        elif LIST_RE.match(line):
            ordered = line[0].isdigit()
            items = []
            while i < len(lines) and LIST_RE.match(lines[i]):
                marker = LIST_RE.match(lines[i]).group(0)
                item = [lines[i][len(marker):]]
                i += 1
                while i < len(lines):
                    nxt = lines[i]
                    if nxt.startswith("  ") or (not nxt.strip() and i + 1 < len(lines)
                                                and lines[i + 1].startswith("  ")):
                        item.append(nxt[len(marker):] if nxt.strip() else "")
                        i += 1
                    else:
                        break
                items.append(item)
                if i < len(lines) and not lines[i].strip() and i + 1 < len(lines) \
                        and LIST_RE.match(lines[i + 1]):
                    i += 1
            tag = "ol" if ordered else "ul"
            lis = []
            for item in items:
                inner = blocks(item)
                parts = [inner[0][1][3:-4] if k == 0 and inner[0][0] == "p" else b[1]
                         for k, b in enumerate(inner)]
                lis.append("<li>" + "\n".join(parts) + "</li>")
            out.append(("list", f"<{tag}>" + "".join(lis) + f"</{tag}>", None))
        else:
            j = i
            while j < len(lines) and lines[j].strip() and not re.match(
                    r"^(#|```|\||<!--|- |\d+\. )", lines[j].strip()):
                j += 1
            para = lines[i:j]
            if all(IMG_RE.fullmatch(l.strip()) for l in para):
                out.append(("fig", figures(para), [IMG_RE.fullmatch(l.strip()).groups()
                                                   for l in para]))
            else:
                out.append(("p", "<p>" + inline(" ".join(l.strip() for l in para)) + "</p>",
                            None))
            i = j
    return out


# ---------------------------------------------------------------- page

def words(md_lines):
    return len(re.findall(r"[A-Za-z0-9']+", plain(" ".join(md_lines))))


def build():
    src = GUIDE.read_text(encoding="utf-8").splitlines()

    # Split the guide at "## " headings for section wrappers and read times.
    sections, cur = [], {"title": None, "lines": []}
    in_code = False
    for line in src:
        if line.strip().startswith("```"):
            in_code = not in_code
        if not in_code and line.startswith("## "):
            sections.append(cur)
            cur = {"title": line[3:].strip(), "lines": []}
        else:
            cur["lines"].append(line)
    sections.append(cur)

    intro = blocks(sections[0]["lines"])
    title = next(b[1] for b in intro if b[0] == "h" and b[2] == 1)
    lede = next(b[1] for b in intro if b[0] == "p")
    # The hero is the first sign render in the intro (the boot animation).
    hero_alt, hero_src, _ = next(img for b in intro if b[0] == "fig"
                                 for img in b[2] if is_sign(img[1]))
    comments = "\n".join(b[1] for b in intro if b[0] == "comment")

    body, index = [], []
    for n, sec in enumerate(sections[1:]):
        anchor = slug(sec["title"])
        line = LINES[n % len(LINES)]
        mins = max(1, math.ceil(words(sec["lines"]) / 200))
        index.append(
            f'<li><a href="#{anchor}"><span class="dest">{inline(sec["title"])}</span>'
            f'<span class="min" aria-hidden="true">{mins} MIN</span>'
            f'<span class="vh">, {mins} minute read</span></a></li>')
        parts = []
        for kind, h, meta in blocks(sec["lines"]):
            if kind == "h":
                parts.append(f'<h{meta} id="{slug(plain(h))}">{inline(h)}</h{meta}>')
            else:
                parts.append(h)
        body.append(
            f'<section class="line-{line}" aria-labelledby="{anchor}">\n'
            f'<h2 id="{anchor}"><span class="bullet" aria-hidden="true"></span>'
            f'{inline(sec["title"])}</h2>\n' + "\n".join(parts) + "\n</section>")

    page = TEMPLATE
    for key, val in {
        "TITLE": html.escape(plain(title)),
        "DESCRIPTION": html.escape(re.sub(r"<[^>]+>", "", lede[3:-4])),
        "H1": inline(title),
        "LEDE": lede,
        "HERO": sign_figure(hero_alt, hero_src, hero=True),
        "COMMENTS": comments,
        "INDEX": "\n".join(index),
        "BODY": "\n\n".join(body),
        "SITE_URL": SITE_URL,
        "REPO_URL": REPO_URL,
    }.items():
        page = page.replace("{{" + key + "}}", val)
    return page


TEMPLATE = """<!doctype html>
<!-- Generated from guide.md by tools/build_site.py. Edit guide.md, not this file. -->
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{{TITLE}}</title>
<meta name="description" content="{{DESCRIPTION}}">
<meta name="color-scheme" content="light dark">
<meta name="theme-color" content="#0064a8">
<meta property="og:type" content="website">
<meta property="og:title" content="{{TITLE}}">
<meta property="og:description" content="{{DESCRIPTION}}">
<meta property="og:url" content="{{SITE_URL}}">
<meta property="og:image" content="{{SITE_URL}}assets/screens/LAFY-EVENING-0.png">
<meta property="og:image:width" content="480">
<meta property="og:image:height" content="320">
<meta property="og:image:alt" content="Two red LED platform signs listing departures from Lafayette">
<meta name="twitter:card" content="summary_large_image">
<link rel="icon" href="favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="assets/site.css">
</head>
<body>
<a class="skip" href="#guide">Skip to the guide</a>
<header class="topbar">
  <div class="wrap topbar__inner">
    <a class="topbar__name" href="./">Platform sign</a>
    <a class="topbar__link" href="{{REPO_URL}}" rel="noopener">GitHub</a>
  </div>
</header>
<div class="masthead">
  <div class="wrap">
    <h1>{{H1}}</h1>
    {{LEDE}}
  </div>
</div>
<div class="hero">
{{COMMENTS}}
{{HERO}}
</div>
<main id="guide">
<nav class="board" aria-labelledby="board-title">
  <div class="board__label"><h2 id="board-title">In this guide</h2><span aria-hidden="true">Minutes to read</span></div>
  <div class="board__hood"></div>
  <ol class="board__panel">
{{INDEX}}
  </ol>
</nav>

{{BODY}}
</main>
<footer class="footer">
  <div class="wrap">
    <p>A fan project, not affiliated with BART. Code on <a href="{{REPO_URL}}" rel="noopener">GitHub</a>, MIT license.</p>
  </div>
</footer>
<script>
document.querySelectorAll("pre").forEach(function (pre) {
  if (!navigator.clipboard) return;
  var box = document.createElement("div");
  box.className = "code";
  pre.parentNode.insertBefore(box, pre);
  box.appendChild(pre);
  var b = document.createElement("button");
  b.type = "button";
  b.className = "copy";
  b.textContent = "Copy";
  b.addEventListener("click", function () {
    navigator.clipboard.writeText(pre.textContent.replace(/\\n$/, "")).then(function () {
      b.textContent = "Copied";
      setTimeout(function () { b.textContent = "Copy"; }, 1500);
    });
  });
  box.appendChild(b);
});
</script>
</body>
</html>
"""


if __name__ == "__main__":
    page = build()
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text(encoding="utf-8") != page:
            print("docs/index.html is stale: run python3 tools/build_site.py", file=sys.stderr)
            sys.exit(1)
        print("docs/index.html is up to date")
    else:
        OUT.write_text(page, encoding="utf-8")
        print(f"wrote {OUT.relative_to(ROOT)}")
