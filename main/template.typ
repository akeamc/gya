// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(
  title: "",
  abstract: [],
  authors: (),
  date: none,
  logo: "sodralatin.svg",
  body,
) = {
  // Set the document's basic properties.
  set document(author: authors, title: title)

  let body-font = "Linux Libertine"
  let sans-font = "Inria Sans"

  set text(font: body-font, lang: "sv")
  show heading: set text(font: sans-font)
  set heading(numbering: "1.1")

  // Title page.
  // The page can contain a logo if you pass one with `logo: "logo.png"`.
  if logo != none {
    align(left, image(logo, width: 20%))
  }
  v(9.6fr)

  text(1.1em, date)
  v(1.2em, weak: true)
  text(2em, weight: 700, font: sans-font, tracking: -0.02em, title)

  // Author information.
  pad(
    top: 0.7em,
    right: 20%,
    grid(
      columns: 1fr,
      gutter: 1em,
      ..authors.map(author => align(start, strong(author))),
    ),
  )

  v(2.4fr)
  pagebreak()

  set page(numbering: "1", number-align: center)
  counter(page).update(1)

  // Abstract page.
  v(1fr)
align(center)[
    #heading(
      outlined: false,
      numbering: none,
      text(0.85em, smallcaps[Abstract]),
    )
    #abstract
  ]
  v(1.618fr)
  pagebreak()

  // Table of contents.
  outline(depth: none, indent: true)
  pagebreak()


  // Main body.
  set par(justify: true)

  // Decimal comma
  show math.equation: it => {
    show regex("\d+\.\d+"): it => { show ".": { "," + h(0pt) }
      it}
    it
  }

  show bibliography: set heading(numbering: "1.1")

  body
}
