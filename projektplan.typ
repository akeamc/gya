#set text(font: "Linux Libertine", lang: "sv")

#let sans-font = "Inria Sans"

#set page(header: [Åke Amcoff och Jarl Åkesson])

#show heading: set text(font: sans-font)

#text(2em, weight: 700, font: sans-font, [Projektplan -- passiv platsbestämning av wifiklienter])

#set par(justify: true)

= Bakgrund

Wifienheter (mobiltelefoner, datorer etc.) sänder hela tiden radiosignaler. Ett nätverk av minst tre mottagare borde därför kunna användas för platsbestämning av enheterna.

#figure(
  image("./phased_array.gif", width: 150pt),
  caption: [Fasstyrd antenn.]
)

= Problemställning

Frågan är vilken precision som kan uppnås med "normal" (lättillgänglig) hårdvara och huruvida ToF #footnote([time of flight; radiosignalens restid]), AoA #footnote([angle of arrival; radiosignalens infallsvinkel mot antennen]) eller en kombination ger bäst resultat.

= Genomförande

== Plan $alef$ (alltså före A)

Existerar. Hemlig.

== Plan A

+ Prototyp av central server; simulerad wifitrafik.
+ Design och inköp av sensorer.
+ Montering av sensorer.
+ Programmering:
   - *Sensorernas mjukvara.* Skicka uppsnappade wifiramar till en central server.
   - *Centrala servern.* Behandling och analys av wifiramar. Platsbestämning.
+ Mätning av precision.
+ Utkast till rapport.
+ Färdigställande av rapport.

= Tidsplan

_Vi ska sikta på att vara färdiga med själva laborationen innan jullovet. Plan $alef$ får ta lite längre tid. Laborationen kan oavsett vilken plan vi väljer genomföras samtidigt med rapportskrivningen._

- Stipendium för sensorer bör sökas så snart som möjligt.
- Steg 1 beräknas ta två veckor.
- Sensorerna ska vara oss tillhanda i början av oktober.
- Steg 3--5 ska tillsammans ta två veckor.
- Resten av gymnasiearbetestiden går åt till att skriva.
