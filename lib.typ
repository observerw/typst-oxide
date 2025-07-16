#let meta(..args) = metadata(args)

#let wikilink-regex = regex("\[\[([^|\]]+?)(?::([^|\]]+?))?(?:\|([^\]]+))?\]\]")

#let note(
  body,
) = {
  set text(font: ("Source Han Sans SC", "Times New Roman"), size: 14pt, region: "cn")
  set par(justify: true)
  set page(height: auto, margin: 2em)
  show wikilink-regex: it => {
    let parsed = it.text.match(wikilink-regex)
    let captures = parsed.captures
    let path = captures.at(0)
    let label = captures.at(1)
    let alias = captures.at(2)

    set text(fill: blue)
    if alias != none {
      link(path)[#alias]
    } else if label != none {
      link(path)[#path > #label]
    } else {
      link(path)
    }
  }

  set math.equation(numbering: "(1)")

  show heading.where(level: 1): it => {
    block[
      #it
      #line(length: 100%, stroke: 1pt + gray)
      #h(0em)
    ]
  }

  body
}
