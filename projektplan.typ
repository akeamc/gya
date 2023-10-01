#set text(font: "Linux Libertine", lang: "sv")

#let sans-font = "Inria Sans"

#set page(header: [Åke Amcoff och Jarl Åkesson])

#show heading: set text(font: sans-font)


#show regex("\d+\.\d+"): it => {
  show "." : ","
  it
}

#text(2em, weight: 700, font: sans-font, [Projektplan -- passiv platsbestämning av wifiklienter])

#set par(justify: true)

= Bakgrund

Wifienheter (mobiltelefoner, datorer etc.) sänder hela tiden signaler i mikrovågsspektrumet (2,4 och 5~GHz). Ett nätverk av minst tre mottagare borde därför kunna användas för platsbestämning av enheterna.

#figure(
  image("./phased_array.gif", width: 150pt),
  caption: [Fasstyrd antenn.]
)

Wifitrafik består av _ramar_ som bland annat innehåller avsändarens MAC-adress #footnote([Media Access Control-adressen är ett 6 byte långt nätverksenhets-id som diskriminerar wifienheter.]). Själva adressen är inte krypterad; dock duckar fler och fler enheter för integritetskränkande experiment som detta genom att då och då ändra sin MAC-adress. Om (samma) ramar med samma MAC-adress fångas upp av flera mottagare ungefär samtidigt kan avsändarens position beräknas.

Enheternas globala position är inte särskilt intressant -- vi söker den lokala positionen inom sensornätverket, där sensorerna är fixerade. Måttband bör räcka.

Många moderna wifikort har flera antenner och sålunda kan AoA mätas, men bara i planet. Till plan #sym.alef behövs två vinklar (i rummet). Antingen bygger vi ett eget wifikort från grunden (svårt), eller så monterar vi två kort vinkelrätt mot varandra (lätt).

ToF blir svårt om inte omöjligt att mäta med "vanlig" hårdvara eftersom signalen färdas med ljusets hastighet (3~dm/ns) och mottagarnas processorer och klockor är för långsamma och opålitliga.

Varje sensor består av en Raspberry Pi och en passande wifimottagare.

= Problemställning

Frågan är vilken precision som kan uppnås med "normal" (lättillgänglig) hårdvara och huruvida ToF #footnote([time of flight; radiosignalens restid]), AoA #footnote([angle of arrival; radiosignalens infallsvinkel mot antennen]) eller en kombination ger bäst resultat.

= Genomförande

== Plan #sym.alef (alltså före A)

Plan #sym.alef är
- ganska hemlig
- antagligen laglig
- etisk med rätt frågeställning.

#figure(
  image("./alef.png", width: 100%),
  caption: [*Plan #sym.alef.* Trafik från lärarrummet fångas upp av mottagaren på skolgården. Källan lokaliseras (tvådimensionellt på fasaden) med azimut och altitud.]
) <alef>

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

_Vi ska sikta på att vara färdiga med själva laborationen innan jullovet. Plan #sym.alef får ta lite längre tid. Laborationen kan oavsett vilken plan vi väljer genomföras samtidigt med rapportskrivningen._

- Stipendium för sensorer bör sökas så snart som möjligt.
- Steg 1 beräknas ta två veckor.
- Sensorerna ska vara oss tillhanda i början av oktober.
- Steg 3--5 ska tillsammans ta två veckor.
- Resten av gymnasiearbetestiden går åt till att skriva.

