#import "@preview/cetz:0.1.1"

#set page(
  header: align(right, [Stockholm 29 september 2023])
)

#set text(font: "Linux Libertine", lang: "sv")

#text(2em, weight: 700, [Stipendieansökan för gymnasiearbete])

#grid(
  columns: (1fr, 1fr),
  [
    *Åke Amcoff* \
    ake.amcoff\@elevmail.stockholm.se \
    072 298 71 96 \
  ],
  [
    *Jarl Åkesson* \
    jarl.akesson\@elevmail.stockholm.se \
    070 470 09 68 \
  ]
)

#v(2em)

#set par(justify: true)

Vi går i klass NA21B på Södra Latins gymnnasium och vårt gymnasiearbete går ut på att undersöka med vilken precision man kan platsbestämma wifienheter passivt.

Planen är att använda tre miniatyrdatorer (modell *Raspberry Pi 4*) utrustade med fasstyrda wifiantenner. Inkommande signaler kommer vara olika mycket fasförskjutna från antenn till antenn beroende på källans riktning, och utifrån fasförskjutningen går det att bestämma vinkeln $theta$ (@phased_array). Vinklarna från två eller tre (ju fler datorer desto mindre felmarginal) datorer kan sedan kombineras för att triangulera källans position (@triangulation).

#figure(
  image("./phased_array.gif", width: 150pt),
  caption: [Fasstyrd antenn. Antennerna, märkta med $phi.alt$, kommer ta emot signalen (röd) vid olika tidpunkter. Fördröjningen _(fasförskjutningen)_ från antenn till antenn beror på vinkeln $theta$.]
) <phased_array>

#figure(
  cetz.canvas(length: 0.6cm, {
    import cetz.draw: *
    import cetz.angle: angle

    set-style(
      content: (padding: 3pt),
      angle: (
        radius: 1.5,
        label-radius: 1,
        fill: green.lighten(80%),
        stroke: (paint: green.darken(50%)),
      ),
    )

    let (a, b, o) = ((0,0), (8,0), (5,5))

    line(a, o, mark: (end: ">"))
    line(b, o, mark: (end: ">"))

    content(a, $A$, anchor: "top-right")
    content(b, $B$, anchor: "top-left")
    content(o, $O$, anchor: "bottom")

    angle(a, o, b, label: text(green.darken(50%), $theta_A$))
    angle(b, a, o, label: text(green.darken(50%), $theta_B$))
  }),
  caption: [
    Triangulering av källan $O$ (vid pilarnas skärningspunkt) med hjälp av vinklarna $theta_A$ och $theta_B$.
  ],
) <triangulation>

Vi behöver följande utrustning för att kunna genomföra vårt gymnasiearbete:

#table(
  columns: (1fr, auto, auto, auto),
  align: (left, right, right, right),
  "Modell", "Antal", "Styckpris (kr)", "Summa (kr)",
  "Raspberry Pi 4 (1 GB)", "3", "499", [1~497],
  "Minneskort", "3", "99", str(99 * 3),
  "Strömadapter", "3", "99", str(99 * 3),
  "Wifikort", "3", "ca 500", [ca 1~500]
)

#align(bottom, [Tack på förhand!])
