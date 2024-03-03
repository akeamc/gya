#import "@preview/tablex:0.0.8": tablex

// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(
  title: "",
  sammanfattning: [],
  abstract: [],
  acknowledgements: [],
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
  set math.equation(numbering: "(1)")

  show link: underline

  // Title page.
  // The page can contain a logo if you pass one with `logo: "logo.png"`.
  grid(
    columns: (1fr, 20%),
    text([
      Södra Latins gymnasium, Stockholm \
      #authors.join("\n") \
      #date \
      (Datum för inlämning reviderad version 1) \
      (Datum för inlämning reviderad version 2)
    ]),
    image(logo, width: 100%)
  )
  
  v(9.6fr)

  v(1.2em, weak: true)
  text(2em, weight: 700, font: sans-font, tracking: -0.02em, title)

  // Author information.
  /* pad(
    top: 0.7em,
    right: 20%,
    grid(
      columns: 1fr,
      gutter: 1em,
      ..authors.map(author => align(start, strong(author))),
    ),
  ) */

  v(2.4fr)

  align(right, text(1.1em, "Handledare: Rickard Fors"))
  
  pagebreak()

  set page(
    header: [
      #set text(8pt)
      #smallcaps([Amcoff och Åkesson: #title])
      #h(1fr) #date
    ],
    footer: [
      #h(1fr)
      #counter(page).display("1")
    ]
  )
  counter(page).update(1)

  // Main body.
  set par(justify: true)

  // Abstract page.
  heading(
    outlined: false,
    numbering: none,
    [Sammanfattning],
  )
  sammanfattning

  // pagebreak(weak: true)

  heading(
    outlined: false,
    numbering: none,
    [Abstract],
  )
  abstract

  pagebreak(weak: true)

  heading(
    outlined: false,
    numbering: none,
    [Tillkännagivanden],
  )
  acknowledgements

  pagebreak(weak: true)

  // Table of contents.
  outline(depth: none, indent: true)
  pagebreak()


  // Decimal comma
  show math.equation: it => {
    show regex("\d+\.\d+"): it => { show ".": { "," + h(0pt) }
      it}
    it
  }

  show bibliography: set heading(numbering: "1.1")

  body
}
