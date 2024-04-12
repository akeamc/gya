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

  set page(margin: 0pt)

  image("cover.jpg", width: 100%, height: 100%)

  let logo = read("sodralatin.svg").replace(
    "#000000",
    white.to-hex(),
  )
  
  place(top, box(inset: 2.5cm, height: 100%, [
    #set text(fill: white, font: sans-font, weight: "bold")
    #grid(
      columns: (1fr, 20%),
      text([
        Södra Latins gymnasium, Stockholm \
        #authors.join("\n") \
        #date \
        //(Datum för inlämning reviderad version 1) \
        //(Datum för inlämning reviderad version 2)
      ]),
      //image.decode(logo, width: 100%)
    )
    
    #v(1fr)

    #box(width: 70%, text(3.5em, weight: 700, font: sans-font, tracking: -0.02em, title))
  
    #v(8em)
  
    #text(1.1em, "Handledare: Rickard Fors")
  ]))

  pagebreak()

  set page(margin: auto)

  set page(
    header: [
      #set text(8pt)
      #smallcaps([Amcoff och Åkesson: #title])
      #h(1fr) #date
    ],
    footer: [
      #h(1fr)
      #counter(page).display("1")
    ],
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

  set text(lang: "en")
  heading(
    outlined: false,
    numbering: none,
    [Abstract],
  )
  abstract
  set text(lang: "sv")

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
