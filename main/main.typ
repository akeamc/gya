#import "template.typ": *
#import "@preview/cetz:0.2.0"
#import "@preview/tablex:0.0.8": tablex, rowspanx, colspanx
#import "@preview/unify:0.5.0": num, qty, numrange, qtyrange
#import "@preview/fletcher:0.4.3" as fletcher: diagram, node, edge

#let note(body) = box(stroke: red, inset: 0.5cm, body)
#show: project.with(
  title: "Passiv lokalisering av wifiklienter",
  authors: (
    "Åke Amcoff",
    "Jarl Åkesson",
  ),
  sammanfattning: [
    //En sammanfattning av hela arbetet (syfte, metod, resultat och slutsatser) på högst en halv sida. En läsare ska kunna förstå vad ni har undersökt, hur ni gjort det och vilka resultat ni kommit fram till genom att bara läsa sammanfattningen.

    Hela tiden platsbestämns vi, och platsdata samlas in av appar som Google Maps för att till exempel mäta mängden trängsel på restauranger eller fordonstrafik. Denna platsbestämning sker genom GPS-system inuti enheterna där deras position avgörs genom trilaterering. Problemet med denna metod är att avsändarna själva avgör när de vill skicka in information till en databas. Platsbestämning av enheter utan detta sjärrapporteringsbehov skulle möjliggöra mer omfattande statistik angående platsdata. Syftet med detta gymnasiearbete är att undersöka hur rimlig denna form av lokalisering av wifiklienter är avseende vilken precision som kan uppnås.

    För att undersöka detta omprogrammerade vi mjukvaran i en Asus RT-AC86U wifirouter i syfte att mäta fasförskjutningen av signaler från en wifiklient mellan routerns tre externa antenner och på så sätt beräkna precisionen möjlig vid triangulering med denna router -- som är en tämligen typisk "off-the-shelf"-router, vilket är avgörande för om ett system baserat på denna rapport kan implementeras i existerande wifinätverksinstallationer.

    I experimentet uppnåddes en precision om #qty("+-10", "degree") för vinklar mellan #qty("-30", "degree") och #qty("30", "degree"). På grund av antennernas placering på routern kunde inte större vinklar mätas.
  ],
  abstract: [
    We are localized all the time, and location data is collected by applications such as Google Maps in order to measure crowdedness at restaurants, or traffic levels, for example. This localization is done using the built-in GPS systems of the devices, which in turn utilize satellites and trilateration to determine the location of the devices. The main issue with this method is that it builds upon _active_ localization, meaning that the devices being localized themselves need to report their location, as opposed to _passive_ localization where the position of the devices can be silently detected by a third party. Passive localization would thus enable more thorough location statistics. The purpose of this paper is to determine the level of precision achieveable with such a passive localization system.

    In order to study this, we reprogrammed the firmware of an Asus RT-AC86U wifi router to be able to measure the phase shift between the router's three antennas of wifi signals originating from a wifi client in order to determine the angle of arrival of the signal, and in turn the direction of the wifi device itself. A crucial property of the wifi router used in this study is that it is a very typical, off-the-shelf router, which means a system based on this paper could be deployed widely at low cost.

    A precision of #qty("+-10", "degree") for angles between #qty("-30", "degree") and #qty("30", "degree") was achieved in the experiment. The manufacturer's antenna placement prohibited the measuring of larger angles.
  ],
  acknowledgements: [
    //En kall novembereftermiddag följde vår käre vän Vidar Nordqvist med Åke Amcoff till Hökarängen för att köpa en begagnad router på Blocket. Tack.

    Genom Norrnässtiftelsen blev vi tilldelade pengar från Södra Latin av summan 1~000 kronor. Utan detta hade inköp av nödvändiga materiel varit omöjlig.

    Genom sitt överseende tillät Jenny Alpsten programmering av gymnasiearbetet under ett flertal mattelektioner. Utan detta är det oklart huruvida ett genomförande av vårt experiment hade varit möjligt inom den givna tidsramen. 

    Vidar Nordqvist var en kämpe och gav emotionellt stöd vid inköp av wifiroutern Asus RT-AC86U från Blocket. Med rädslan närvarande vid inköp genom Blocket i åtanke, var detta starkt och hjälpsamt. 
  ],
  date: "12 april 2024",
)

#let rtac86u_d = qty("0.09", "m")

= Inledning

== Presentation

Sedan början av 1970-talet har GPS (Global Positioning System) blivit en alltmer använd teknologi. Det fungerar genom att ett flertal satelliter skickar ut information cirka 20~000~kilometer från jorden med hjälp av transpondrar om sin exakta tid och position. GPS-mottagaren får då in information från flera satelliter och kan genom _trilaterering_ räkna ut exakt var den är. Trilaterering är ett sätt att fastställa en enhets position genom att mäta avståndet från enheten till tre eller fler andra enheter (se @tri). Eftersom det finns minst fyra satelliter ovanför ett GPS system fungerar detta alltid. @lantmateriet

Idag platsbestäms vi hela tiden, och platsdata samlas in _en masse_ av appar som Google Maps för att till exempel uppskatta hur trafikerad en väg är eller hur trång en restaurang är -- i realtid. Denna funktion bygger på självrapportering. Wifienheter vet genom GPS sin position och ger den informationen till Google som sedan drar slutsats om trafikering. Eftersom denna metod är beroroende av självrapportering avgör enheten själv när dess information bör sändas till en databas. Detta skiljer sig från _passiv lokalisering_ där en sensor avgör var en enhet är utan att någon åtgärd krävs från användarens enhets håll. 

Fastställning av position är dock inte bara möjligt genom trilaterering. Istället för distans och tid kan det genom _triangulering_ platsbestämmas. I denna metod mäts vinkeln mellan avsändare och minst två mottagare. Om mottagarnas exakta positioner är kända kan avsändarens plats bestämmas. Eftersom tid inte är en variabel som beräknas är atomklockor inte nödvändiga för platsbestämning genom triangulering vilket gör denna metod billigare och enklare för undersökningar på en mindre skala än motsvarande trilatereringsmetod, när det rör sig om radiosignaler, vars hastighet är så hög ($c$) att skillnaden i ankomsttid mellan mottagare kan vara bara några nanosekunder.

// https://www.lantmateriet.se/en/geodata/gps-geodesi-och-swepos/gps-and-satellite-positioning/gps-and-other-gnss/gps/

== Syfte

Syftet med detta gymnasiearbete är att undersöka om passiv platsbestämning av wifienheter är träffsäkert och rimligt. Ifall detta är rimligt skulle det innebära möjligheter för större insamling av data att användas under effektivisering av stadsplanering samt egentid. 

Problemet med _aktiv platsbestämning_ är att den är beroende av självrapportering. Alltså avgör enheter själva när information om deras position är lämplig att sända till en databas. På grund av att enheterna själva avgör när platsbestämning är lämpligt ger inte detta lika omfattande statistik som platsbestämning utan aktiv tillåtelse. Ifall hög precision kan uppnås genom passiv platsbestämning skulle detta ha många tillämpningsområden inom effektivisering av stadsplanering. 

Den mer omfattande mängden data skulle innebära enklare identifiering av trafikering. Butiksägare och restaurangsägare kan då bättre optimera öppetider och marknadsföring för att minska konkurrens samt öka tillgänglighet. Det skulle också göra livet enklare för konsumenter, eftersom de alltid hade vetat väntetider för restauranger eller trängsel i butiker och mataffärer vilket skulle innebära en minskning i slöseri av tid. Uppskattningar av fordonstrafik skulle också vara mer korrekta vilket hade möjliggjort mer effektiv infrastruktur, eftersom problematiska områden och tidpunkter då är mer uppenbara.

Problematiska områden avseende smittspridning vore också enklare att identifiera. Vid utbrott av aggressivt smittsamma sjukdomar skulle mer effektiv samt större mängd platsdata möjliggöra för områdesspecifika restriktioner och då minska behovet av omfattande restriktioner samt mängden drabbade av sjukdom. Privatpersoner hade då mer enkelt kunnat ta del av nödvändiga aktiviteter som inhandling av livsmedel samt mindre nödvändiga aktiviteter som restaurangbesök vilket hade lett till att färre företag måste stänga ner eller göra nedskärningar.  

== Frågeställning

#note([*Vad tycks?*])

#strike([Vilken precision kan uppnås av passiv lokalisering av wifiklienter med en Asus RT-AC86U wifirouter gällande platsbestämning?])

Vilken precision kan uppnås vid bestämning av den infallande wifisignalens vinkel med en wifirouter märkt Asus RT-AC86U?

#pagebreak(weak: true)

= Bakgrund

== Triangulering och trilaterering <tri>

Triangulering och trilaterering är två olika sätt att platsbestämma något (här: $O$) genom att mäta vinklar respektive avstånd. Båda kräver flera mätningar från olika punkter vars positioner är kända (här: $A$, $B$, $C$).

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

    let (a, b) = ((0,0), (8,0))

    set-style(stroke: (dash: "dashed", paint: gray))

    intersections("i", {
      line(a, (8, 8))
      line(b, (5, 8))

      hide({
        circle(a, radius: 3)
        circle(b, radius: 3)
      })
    })
    
    set-style(stroke: 1pt)

    angle(a, "i.0", b, label: text(green.darken(50%), $theta_A$))
    angle(b, (rel: (1, 0)), "i.0", label: text(green.darken(50%), $theta_B$))

    line(a, "i.1", mark: (end: ">"))
    line(b, "i.2", mark: (end: ">"))

    set-style(circle: (radius: 0.1cm, stroke: none, fill: black))
    
    circle(a, name: "a_l")
    circle(b, name: "b_l")
    circle("i.0", name: "o_l")
  
    content("a_l", $A$, anchor: "north-east")
    content("b_l", $B$, anchor: "north-west")
    content("o_l.east", $O$, anchor: "west")
  }),
  caption: [
    Triangulering av $O$ från $A$ och $B$.
  ],
) <triangulation>

Med två kända punkter $A$ och $B$, samt vinkeln till $O$ från respektive punkt, går det att triangulera $O$. Varje vinkel ger en stråle (de sträckade linjerna i @triangulation) varpå $O$ kan ligga. Där strålarna från $A$ och $B$ korsar varandra finns $O$.

#figure(caption: [Trilaterering av $O$ från $A$, $B$ och $C$.], cetz.canvas(length: 0.75cm, {
  import cetz.draw: *

  let (a, b, c) = ((0,0), (4,0), (4,5))
  
  intersections("i", {
    circle(a, radius: 4, stroke: red)
    circle(b, radius: 3, stroke: green)
    circle(c, radius: 2.5, stroke: blue)
  })

  set-style(circle: (radius: 0.1cm, stroke: none, fill: black), content: (padding: 3pt))
  circle("i.3", name: "o")
  set-style(stroke: (paint: gray, dash: "dashed"))
  line(a, "o")
  line(b, "o")
  line(c, "o")

  circle(a, name: "a_l")
  circle(b, name: "b_l")
  circle(c, name: "c_l")

  content("a_l.west", anchor: "east", text(red, $A$))
  content("b_l.east", anchor: "west", text(green, $B$))
  content("c_l.north", anchor: "south", text(blue, $C$))

  content("o.north", anchor: "south", $O$)
})) <trilateration>

Trilaterering kräver avståndsmätningar till en okänd punkt $O$ från minst tre punkter ($A$, $B$ och $C$). Avståndsmätning till $O$ från $A$ ger en sträcka $|A O|$, och alla punkter som ligger på avståndet $|A O|$ från $A$ utgör kanten på en cirkel med radien $|A O|$; dessa punkter är alla möjliga positioner för $O$. När avståndet mäts på samma sätt från $B$ till $O$ fås en ytterligare cirkel, och de möjliga positionerna för $O$ begränsas till de två skärningspunkterna mellan cirklarna. För att begränsa antalet möjliga positioner till en enda krävs en tredje mätning, $C$. I den gemensamma skärningspunkterna för alla tre cirklar finns $O$.

== (Trådlös) kommunikation mellan nätverksenheter <osi>

Det finns en rad olika standarder och protokoll som nätverksenheter #footnote([Med "nätverksenheter" avses mobiltelefoner, datorer, servrar, routrar med mera; allt som är uppkopplat till internet och lite till.]) använder för att kommunicera med varandra. De mer abstrakta (icke-fysiska) protokollen, som HTTP (för webbsurfning) och SMTP (för mejl), gör ingen skillnad på om de förs över wifi eller trådbundet. Likaså spelar det ingen roll för wifikretsen huruvida bitarna som sänds och tas emot är HTTP, SMTP, nonsens eller något annat. I grund och botten handlar de flesta internetprotokoll om att representera information i binär form (i bitar) på ett eller annat sätt. De mest grundläggande protokollens syfte är att överföra dessa bitar genom den fysiska världen, till exempel genom en kopparkabel eller i luften.

OSI-modellen, framtagen av #cite(<iso7498>, form: "prose"), är ett försök att kategorisera standardena för kommunikation nätverksenheter sinsemellan, och tanken är att varje _lager_ ska vara helt oberoende av de andra lagren och att lagren ska kunna kombineras obehindrat. HTTPS (HTTP Secure), till exempel, heter _secure_ eftersom det är krypterat, men namnet är missvisande eftersom informationen inte krypteras i HTTP(S)-lagret (lager 7) utan i lager 6, med TLS. HTTP, å andra sidan, är okrypterat och sålunda inte "inneslutet" av TLS i lager 6. På lager 7 är HTTP och HTTPS dock identiska, vilket gör att mycket HTTP-programkod (webbservrar, till exempel) kan nyttjas oavsett om kryptering används eller ej.

#figure(kind: table, tablex(
  columns: 4,
  map-hlines: h => (..h, stroke: if h.y == 0 { 0pt } else { 0.5pt }),
  map-vlines: v => (..v, stroke: 0.5pt),
  auto-vlines: false,
  colspanx(2, [*Lager*]), [*Används  till*], [*Exempel*],
  [7], [Applikation], [Applikationsprotokoll], [HTTP],
  [6], [Presentation], [Kompression, kryptering, teckenkodning], [TLS],
  [5], [Session], [Sessionshantering], [SOCKS],
  [4], [Transport], [Sändnings- och ankomstkontroll], [TCP],
  [3], [Nätverk], [Logisk adressering], [IP],
  [2], [Datalänk], [Fysisk adressering], [MAC],
  [1], [Fysisk], [Bitöverföring], [IEEE 802.3, IEEE 802.11],
), caption: [OSI-modellens lager.])

Lager 1 överför bitar genom den fysiska världen. IEEE 802.3 (Ethernet) och IEEE 802.11 (wifi) hör till de mest kända samlingarna av standarder; IEEE 802.3 standardiserar bitöverföringen i kopparkablar och fiberoptik medan IEEE 802.11 standardiserar trådlös överföring.

#figure(kind: table, tablex(
  columns: 3,
  map-hlines: h => (..h, stroke: if h.y == 0 { 0pt } else { 0.5pt }),
  map-vlines: v => (..v, stroke: 0.5pt),
  auto-vlines: false,
  [*Wifigeneration*], [*IEEE-standard*], [*Antagen*],
  [(Wifi 0) @wifiretroactive], [802.11], [1997],
  [(Wifi 1) @wifiretroactive], [802.11b], [1999],
  [(Wifi 2) @wifiretroactive], [802.11a], [1999],
  [(Wifi 3) #footnote([Wifi 0 #sym.dots.c 3 är inofficiellt och retroaktivt namngivna.]) <wifiretroactive>], [802.11g], [2003],
  [Wifi 4], [802.11n], [2008],
  [Wifi 5], [802.11ac], [2014],
  [Wifi 6], [802.11ax], [2019],
  [Wifi 7], [802.11be], [2024],
  [Wifi 8], [802.11bn], [2028],
), caption: [IEEE 802.11-standarder och motsvarande konsumentvänliga namn. @wifistandards])

IEEE 802.11 består av en uppsjö standarder från de senaste 30 åren. I vårt experiment studeras IEEE 802.11ac (den näst senast antagna standarden) av två huvudsakliga skäl: Dels är IEEE 802.11ac-kretsar i skrivande stund vanligt förekommande och relativt billiga jämfört med IEEE 802.11ax-kretsar, dels är verktyget för manipulering av wifikretsmaskinvara som utvecklades av #cite(<csi>, form: "prose") designat för IEEE 802.11ac.

IEEE 802.11 är en komplicerad standard, men det enda som läsaren behöver ta med sig är följande:

- Wifisignalerna skickas på en viss _kanal_ (del av frekvensspektrumet #footnote(<freqband>)) som bestäms av wifiroutern som enheten är ansluten (eller försöker ansluta) till.
- Bitarna som lagret ovanför (lager 2) vill överföra paketeras i _wifiramar_ och skickas några (upp till 2 304 bytes) i taget, tillsammans med mottagaradress och en del ytterligare information som är irrelevant här. Ju fler bitar, desto fler wifiramar.
- I denna studie har wifisignalen den ungefärliga frekvensen #qty("5", "GHz").

//#note([Här ska wifikanaler förklaras. Tills vidare, #link("https://en.wikipedia.org/wiki/List_of_WLAN_channels#5_GHz_(802.11a/h/n/ac/ax")[se Wikipedia].])

== Ortogonal frekvensdelningsmultiplex <ofdm>

Ortogonal frekvensdelningsmultiplex (OFDM) ökar överföringshastigheten genom att signalöverföringen delas in i flera parallella dataströmmar, så kallade underbärare _(subcarriers)_, på var sin frekvens. OFDM gör signalöverföringen tålig mot störningar som drabbar vissa frekvenser, och dessutom kan de olika underbärarna reflekteras på olika sätt så att _tillräckligt många_ når mottagaren. @ofdmhistory

OFDM används i alla wifistandarder från och med IEEE 802.11a @wifistandards, och således även i IEEE 802.11ac som granskas ingående i denna rapport.

/*
#figure(
  caption: [Underbärare i IEEE 802.11ac. @survivalguide],
  kind: table,
  tablex(
    columns: 3,
    auto-vlines: false,
    map-hlines: h => (..h, stroke: if h.y == 0 { 0pt } else { 0.5pt }),
    align: (x, y) => if y == 0 or x == 1 { left } else { right },
    [*Bandbredd (MHz)*],
    [*Underbärarindex*],
    [*Antal använda underbärare*],
    "20",
    [-28 till -1, +1 till +28],
    "56",
    "40",
    [-58 till -2, +2 till +58],
    "114",
    "80",
    [-122 till -2, +2 till +122],
    "242",
    "160",
    [-250 till -130, -126 till -6, +6 till +126, +130 till +250],
    "484"
  )
) <subcarrier_indices>
*/

#figure(
  caption: [Underbärare som används i IEEE 802.11ac för varje bandbredd. Glappen (vid 0, till exempel) orsakas av oanvända underbärare. Varför vissa underbärare inte används är irrelevant här.],
  cetz.canvas({
    import cetz.draw: *
    import cetz.plot
    
    let subcarriers(offset: 0, label: "", points) = {
      plot.add(label: label, points.map(((x,y)) => (x,y - offset * 1.5)))
      //plot.add-hline(offset * -1.5, style: (stroke: gray))
    }
    
    plot.plot(size: (14,4), x-min: -256, x-max: 256, x-tick-step: 64, x-label: "Underbärare", y-min: -5, y-max: 2, y-tick-step: 1.5, y-label: none, x-grid: true, y-grid: true, y-format: v => "", legend: "legend.inner-north-west", {
      subcarriers(label: [20 MHz], offset: 0, ((-29,0), (-28,1), (-1,1), (0,0), (1,1), (28,1), (29,0)))
      subcarriers(label: [40 MHz], offset: 1, ((-59,0), (-58,1), (-2,1), (-1,0), (1,0), (2,1), (58,1), (59,0)))
      subcarriers(label: [80 MHz], offset: 2, ((-123,0), (-122,1), (-2,1), (-1,0), (1,0), (2,1), (122,1), (123,0)))
      subcarriers(label: [160 MHz], offset: 3, ((-251,0), (-250,1), (-130,1), (-129,0), (-127,0), (-126,1), (-6,1), (-5,0), (5,0), (6,1), (126,1), (127,0), (129,0), (130,1), (250,1), (251,0)))
    })
  })
) <subcarriers>

@subcarriers visar underbärarna för olika _bandbredder_ ($20$, $40$, $80$ respektive #qty("160", "MHz")), det vill säga olika stora "block" av frekvensspektrumet #footnote([I EU har frekvensbandet #qtyrange("5.150", "5.875", "GHz", delimiter: "\"till\"") allokerats till wifi @wifistandards.]) <freqband>. Underbärarna i IEEE 802.11ac är vanligtvis placerade #qty("312.5", "kHz") isär @wifistandards -- i @subcarriers har underbärare $k$ frekvensen $f_0 + k dot qty("312.5", "kHz")$ där $f_0$ är den mittersta underbärarens (underbärare $0$) frekvens; _centerfrekvensen_.

== Interferens och fasförskjutning

I moderna wifiroutrar används flera antenner. Tillsammans bildar antennerna ett antennsystem där varje antenn utgör ett så kallat element. Om antennerna är uniformt fördelade längs en linje bildar de ett uniformt linjärt antennsystem (_ULA; Uniform Linear Array_ på engelska); se @interference. Routern som används i experimentet har ett uniformt linjärt antennsystem med tre element (vilket framgår i @rt-ac86u).

En sändare kan införa en viss farförskjutning $phi$ mellan intilliggande antennelement och utnyttja interferens för att rikta signalen åt ett visst håll (så kallad _beamforming_). @beamforming illustrerar detta. En mottagare kan på motsatt sätt bestämma den infallande signalens vinkel $theta$ utifrån den uppmätta fasförskjutningen mellan angränsande element.

#figure(caption: [Interferensmönster för olika sorters linjära antennsystem. Våglängden $lambda$ är $1 "l.e."$ och färgningen visar den relativa (i respektive diagram) elongationen i varje punkt -- blått är negativt, vitt är noll och rött är positivt.], rect(
  inset: 12pt,
  stroke: 0.5pt,
  grid(
    columns: (auto, auto, auto),
    gutter: 12pt,
    ..(
      ("base.png", [En punktkälla. ]),
      ("far_pair.png", [Två punktkällor (i fas) $4lambda$ isär. Lägg märke till de tydliga nodlinjerna.]),
      ("pair.png", [Två punktkällor (i fas) på avståndet $lambda$ ifrån varandra.]),
      ("15.png", [15 punktkällor (i fas) på en rad $1/2lambda$ isär.]),
      ("15_30.png", [15 punktkällor på en rad $1/2lambda$ isär med $phi = 30 degree$.]),
      ("15_60.png", [15 punktkällor på en rad $1/2lambda$ isär med $phi = 60 degree$.]),
    ).map(((path, caption)) => figure(caption: caption, numbering: none, image("interferens/" + path, width: 90%))
    )
  )
)
) <beamforming>

För att $theta$ ska kunna bestämmas precist måste signalen i fråga träffa alla antennelement från samma vinkel, och källan måste alltså befinna sig på ett tillräckligt stort avstånd från antennsystemet så att vågfronterna blir någorlunda parallella och i fas.

#figure(cetz.canvas(length: 0.9cm, {
    import cetz.draw: *

    set-style(stroke: (thickness: 0.5pt))
  
    let wave(amplitude: 1, fill: none, phases: 2, scale: 8, samples: 300, stroke: stroke) = {
      line(..(for x in range(0, samples + 1) {
        let x = x / samples
        let p = (2 * phases * calc.pi) * x
        let m = calc.clamp(2 * calc.sin(p) + 1.5, 0, 1)
        let y = m * calc.sin(p * 10)
        ((x * scale, y),)
      }), stroke: stroke)
    }
  
    hide(intersections("i", {
      line((0,1), (0,-10))
      rotate(z: -30deg)
      line((-11,0),(100,0), name: "a")
      line((-11,-4),(100,-4), name: "b")
      line((-11,-8),(100,-8), name: "c")
      rotate(z: 30deg)
    }))

    let ray(name, from, to) = {
      line(from, to)
      group(name: name, {
        set-origin(from)
        rotate(-30deg)
        translate(x: 1)
        anchor("top", (7.5, 1.5))
        anchor("bottom", (7.5, -1.5))
        wave(phases: 3, stroke: blue + 1.2pt)
      })
    }

    ray("a_line", "a.start", "i.0")
    ray("b_line", "b.start", "i.1")
    ray("c_line", "c.start", "i.2")
    
    /*line("b.start", "i.1")
    group({
      set-origin("b.start")
      rotate(-30deg)
      translate(x: 1)
      wave(phases: 3, stroke: blue)
    })
    line("c.start", "i.2")
    group(name: "c_line", {
      set-origin("c.start")
      rotate(-30deg)
      translate(x: 1)
      anchor("s", (7.5, -1.5))
      wave(phases: 3, stroke: blue)
    })*/
  
    line("a_line.top", "c_line.bottom", stroke: (paint: red, dash: "dashed"))
  
    let antenna(name: "", center, ray-origin, n) = group(name: name, {
      set-origin(center)
      let s = 0.5
      line((0,-s), (0,s), (calc.sqrt(4*s/3),0), close: true)
      line((0,0), (3,0))
      anchor("start", (0,0))
      anchor("mid", (2,0))
  
      line((0,0),(-1.5,0), stroke: (dash: "dotted"), name: "horizon")
      cetz.angle.angle((0,0), "horizon.end", ray-origin, label: $theta$, radius: 1, label-radius: 75%)
      content((3.1,0), $n=#n$, anchor: "west")
    })
  
    antenna(name: "a_ant", "i.0", "a.start", 0)
    antenna(name: "b_ant", "i.1", "b.start", 1)
    antenna(name: "c_ant", "i.2", "c.start", 2)
  
    set-style(mark: (symbol: (">")))
    line(name: "ab", "a_ant.mid", "b_ant.mid", start: (mark: ">"))
    content("ab.mid", [$d$], anchor: "west", padding: 3pt)
    line(name: "bc", "b_ant.mid", "c_ant.mid", start: (mark: ">"))
    content("bc.mid", [$d$], anchor: "west", padding: 3pt)
  }),
  caption: [Uniformt linjärt antennsystem med tre element. Den infallande signalen (som ser ut att vara tre separata signaler men som inte är det) har vinkeln $theta$ och källan antas befinna sig på ett så stort avstånd $D$ från antennerna att signalen träffar alla antennelement från ungefär samma vinkel ($D>>d$).],
) <interference>

Betrakta den rätvinkliga triangel vars hypotenusa (med längden $d$) går mellan två intilliggande antenner och vars ena katet är ortogonal mot den infallande signalens riktning. Triangelns andra katet kommer ha längden

$ x=d sin theta, $

vilket motsvarar signalens vägskillnad mellan intilliggande antenner. Således är den observerade fasförskjutningen mellan två intilliggande antenner

$
phi=(2pi d sin theta)/lambda.
$ <phaseshift>

Utifrån @phaseshift beräknas den infallande signalens vinkel enligt

$ theta=arcsin((phi lambda)/(2pi d)). $ <aoa>

== Channel state information

Fasförskjutningen kan härledas från data om signalförhållandena, så kallad _Channel State Information (CSI)_, som rapporteras kontinuerligt av wifikretsen för varje ansluten enhet. Från varje antenn fås en så kallad CSI-vektor med komplexa tal

$ A = vec(
  a_1 e^(i alpha_1),
  a_2 e^(i alpha_2),
  a_3 e^(i alpha_3),
  dots.v,
  a_n e^(i alpha_n),
), quad B = vec(
  b_1 e^(i beta_1),
  b_2 e^(i beta_2),
  b_3 e^(i beta_3),
  dots.v,
  b_n e^(i beta_n),
) quad "respektive" quad C = vec(
  c_1 e^(i gamma_1),
  c_2 e^(i gamma_2),
  c_3 e^(i gamma_3),
  dots.v,
  c_n e^(i gamma_n),
) $ <csivec>

där $a_k$, $b_k$ och $c_k$ är den $k$:te underbärarens signalstyrka och $alpha_k$, $beta_k$, $gamma_k$ dess fas, för första, andra respektive tredje antennen. I följande stycken kommer vektorn $A$ (som tillhör den första antennen) förklaras mer i detalj, men samma principer gäller även för de övriga antennernas CSI-vektorer.

Routerns maskinvara rapporterar de komplexa talen i rektangulär form @csi och således kan bara principalvärdet, som ligger i intervallet $(-pi, pi]$, erhållas för argumentet $alpha_k$.

#figure({
  // show math.equation: block.with(fill: white, inset: 1pt)
  set math.equation(numbering: none)

  cetz.canvas(length: 2cm, {
  import cetz.draw: *

  set-style(
    mark: (fill: black, scale: 2),
    stroke: (thickness: 0.4pt, cap: "round"),
    content: (padding: 1pt)
  )

  grid((-1.5, -1.5), (1.5, 1.5), step: 0.25, stroke: gray + 0.2pt)

  circle((0,0), radius: 1)

  line((-1.5, 0), (1.5, 0), mark: (end: "stealth"))
  content((), $ x $, anchor: "west")
  line((0, -1.5), (0, 1.5), mark: (end: "stealth"))
  content((), $ y $, anchor: "south")

  let a = (calc.cos(150deg), calc.sin(150deg))
  let b = (calc.cos(-150deg), calc.sin(-150deg))
  let c = (calc.cos(110deg), calc.sin(110deg))
  let d = (calc.cos(-190deg), calc.sin(-190deg))

  set-style(stroke: (thickness: 1.2pt))

  set-style(stroke: (thickness: 0.4pt))
  
  set-style(
    angle: (
      radius: 0.4,
      label-radius: .22,
      fill: green.lighten(80%),
      stroke: (paint: green.darken(50%))
    ),
  )
  
  cetz.angle.angle((0,0), a, b,
    label: text(green, [$phi$]), inner: false)
  
  set-style(
    angle: (
      radius: 0.3,
      label-radius: .22,
      fill: blue.lighten(80%),
      stroke: (paint: blue.darken(50%))
    ),
  )
  
  cetz.angle.angle((0,0), c, d, inner: true,
    label: text(blue, [$phi'$]))

  line((0,0), a)
  line((0,0), b)
  line((0,0), c)
  line((0,0), d)

  circle(a, radius: 2pt, fill: green, stroke: none)
  circle(b, radius: 2pt, fill: green, stroke: none)
  circle(c, radius: 2pt, fill: blue, stroke: none)
  circle(d, radius: 2pt, fill: blue, stroke: none)
})
}, caption: [Två olika par av faser (paren har var sin färg), förskjutna lika mycket från varandra, men som ger olika observerade fasförskjutningar.]) <d_eq_lambda>

Fasförskjutningen $phi_k$ mellan två intilliggande antenner för den $k$:te underbäraren är differensen mellan motsvarande faser $alpha_k$ och $beta_k$, och ligger i intervallet $(-2pi, 2pi)$. Uttrycket i @aoa har definitionsmängden $[-(2pi d)/ lambda, (2pi d)/lambda]$ för $phi$ och för att kunna beräkna ett entydigt värde på $theta$ för alla vinklar mellan $-pi$ och $pi$ måste därför $d<=lambda$. Med $d=lambda$ är dock ett $phi_k$-värde nära ytterkanterna av värdemängden osannolikt, eftersom det förutsätter att de två faserna $alpha_k$ och $beta_k$ _också_ har värden nära ytterkanterna ($-3$ och $3$, till exempel; se @d_eq_lambda) -- egentligen måste

$ d <= lambda/2 $ <unambig_aoa>

för att fasförskjutningarna $phi_1 dots.c space phi_n$ ska vara korrekta för alla $theta in (-pi, pi)$ -- intervallet som är ekvivalent med bredast möjliga "synfält" för routern.

//#note([$n$ betecknar olika saker på samma ställe. Fixa.])

#pagebreak(weak: true)

= Metod

== Materiel

Huvudkomponenten i lokaliseringssystemet är wifiroutern Asus RT-AC86U, som används eftersom dess wifikrets, Broadcom BCM4366, stödjs av CSI-extraheringsverktyget Nexmon framtaget av #cite(<csi>, form: "prose"). Dessutom har samma router använts i tidigare liknande experiment @ubilocate @schafer2021human @meneghello2022wi. Dess antennavstånd $d$ (se @interference och @aoa) är #rtac86u_d. I experimentet är antennerna inte riktade åt olika håll, som @rt-ac86u visar, utan samtliga är riktade rakt upp (de är vridbara).

#figure(image("rt-ac86u.png", width: 40%), caption: [Wifirouter märkt Asus RT-AC86U. De tre externa antennerna sitter på ovansidan. Routern har en fjärde intern antenn som inte används i experimentet.]) <rt-ac86u>

/*
== Mätmetod

Experimentet utfördes (datumet och tiden på dygnet samt längd av experiment), i ett (storlek på rummet) på Maria Prästgårdsgata 20 av åke Amcoff och Jarl Åkesson.

Dessa var de stegen som utfördes samt skrivna i rätt ordning. (text)

*Exempel*

Vi har en router (exakta routern). Den har 3 externa antenner att mäta data med. Den mäter den platta vinkeln mellan wifienheten och antennerna och får genom detta mätdata avseende wifienhetens plats. 

Olika vinklar samt distanser mellan wifienhet och router mättes för utvinning av mer tydligt resultat angående pålitlighet. 

För att starta programmet gör vi (något), och 
*/
== Avgränsningar
/*
Vad som kan undersökas med de medel vi har och vad det kan ge för slutsats. Platsbestämning av platsbestämningen (IE skolan eller hemma hos Åke eller Åkesson). 
*/
De ekonomiska medeln tillgängliga för vårt förfogande kom från Norrnässtiftelsens stipendium vid summan ett tusen kronor. Nio hundra av dessa kronor användes vid inköp av en Asus RT-AC86U wifirouter genom Blocket. Wifiroutern Asus RT-AC86U kan bara mäta platta vinklar eftersom routerns antennsystem är endimensionellt samt ortogonalt mot horisontalplanet (se @interference). Detta begränsar vår möjlighet att anlända vid en fullkomlig slutsats eftersom en wifienhets relation i höjd till routern inte går att mäta. Till exempel går det inte att avgöra från vilken våning av en byggnad en viss signal kommer. 

Eftersom vi endast har en router att använda till experimentet är det inte heller möjligt att platsbestämma en avsändare genom dess vinklar mot en mängd andra mottagare eftersom triangulering kräver insamling av data från två eller fler mottagare. (Jämför @setup med @triangulation.) I stället mäts precisionen av vinkeln mellan avsändaren och routern. Precisionen som uppnås genom detta visar på precisionen platsbestämning genom triangulering med två eller fler av dessa wifiroutrar skulle vara. En större budget -- cirka två tusen kronor -- hade behövts för att kunna använda två eller till och med tre identiska routrar till att genomföra triangulering. 

På grund av ekonomiska begränsningar samt den stora mängd programmering nödvändig för ett fullständigt genomförande av detta experiment är det uppenbart att en mer omfattande investering krävts. Tid var i synnherhet begränsande.

Med $d=#rtac86u_d$ och $lambda = c/ #qty("5", "GHz") approx #qty("6e-2", "m")$ är olikheten i @unambig_aoa (som måste uppfyllas för att vinklar mellan $-90 degree$ och $90 degree$ ska kunna mätas) falsk. Den uppenbara lösningen är att bygga om antennsystemet med förlängningskablar och ett mindre elementavstånd $d$, som #cite(<ubilocate>, form: "prose") gjorde i sin studie med samma router, men vi saknade de ekonomiska medlen för att konstruera något liknande.

== Genomförande

Genom att modifiera routerns maskinvara kan CSI erhållas @csi: Programvaru-?modifikations-?ramverket Nexmon är testat med programvaruversion `10_10_122_20`, så routerns programvara behövde nedgraderas. Därefter fördes `tcpdump` #footnote([Enligt Nexmons anvisningar ska `tcpdump` användas till att samla in CSI. @csi]) och Nexmons modifierade kernelmodul över till routern med SSH #footnote([SSH är ett lager 7-protokoll (precis som HTTP som beskrivs i @osi) som används för att säkert ansluta sig till andra nätverksenheter.]).

Enheten placeras på ett bestämt avstånd från routern vid en bestämd vinkel enligt @setup.

#figure(caption: [Ritning (ovanifrån) över rummet som experimentet utfördes i med routern ($A$) och laptopen ($O$). Routern är vänd med framsidan mot den streckade mittlinjen. Positiv vinkel innebär att $O$ är till höger om $A$ (sett från routerns baksida); negativ vinkel betyder till vänster.], cetz.canvas({
  import cetz.draw: *

  set-style(stroke: 0.5pt)
  
  rect((-4,-3), (4,3))
  rect((-3, -0.5), (-2, 0.5), name: "router")
  line("router", (4,0), stroke: (dash: "dashed", paint: gray, thickness: 0.5pt), name: "horiz")
  content("router", $A$)
  
  rect((2,2), (2.5,2.5), name: "laptop")
  line("router", "laptop", name: "rl", stroke: red)
  content("laptop", $O$)
  
  cetz.angle.angle("router.center", "rl.end", "horiz.end", radius: 1.5, label: $#qty("-30", "degree")$, label-radius: 135%)
  content(("rl.start", 50%, "rl.end"), angle: "rl.end", padding: 0.1, anchor: "south", $#qty("3", "m")$)
})) <setup>

CSI-insamlingen görs genom att routern ställs in på samma kanal som enheten som ska spåras -- i vårt fall styr vi över enheten och vilket wifinätverk den är ansluten till, och därmed känner vi till kanalen. Därefter genereras nätverks-?trafik, vars CSI rapporteras av wifikretsen, med nätverks-?hastighets-?mätverktyget Iperf #footnote([Iperf är ett program som används för att mäta nätverksprestanda genom att så mycket data som möjligt skickas mellan två nätverksenheter (i vårt fall mellan enheten och en stationär dator på samma lokala nätverk). I experimentet konfigureras Iperf så att dataströmmen delas upp i så många wifiramar som möjligt, eftersom varje wifiram ger upphov till CSI.]). På routern samlas _pcap-paket_ från wifikretsen in (med programmet `tcpdump`) och skickas till en dator för analys.

#figure(
  caption: [Informationen som finns i ett pcap-paket från wifikretsen. @csi],
  kind: table,
  tablex(
    columns: 2,
    auto-vlines: false,
    map-hlines: h => (..h, stroke: if h.y == 0 { 0pt } else { 0.5pt }),
    //align: (x, y) => if y == 0 or x == 1 { left } else { right },
    [*Fält*],
    [*Förklaring*],
    [Received signal strength indicator],
    [Arbiträr signalstyrka],
    [Avsändarens MAC-adress],
    [Serienummer för avsändarens nätverksutrustning],
    [Wifiramnummer],
    [Unikt nummer för wifiramen #footnote([Det vill säga wifiramens "kollinummer".]) som gav upphov till detta CSI-fragment],
    [Antennummer],
    [Antennummer i intervallet $[0,3]$ (se @rt-ac86u)],
    [CSI],
    [CSI-vektor (se @csivec)],
  )
) <inside_pcap>

Ett pcap-paket genereras per antenn och wifiram och innehåller bland annat CSI (fullständig innehållsförteckning finns i @inside_pcap), som filtreras beroende på MAC-adress (bara wifiramarna med enhetens MAC-adress sparas) och grupperas (i datorn) med andra pcap-paket utifrån ramnummer för att faserna från vektorerna $A$, $B$ och $C$ (@csivec) ska kunna jämföras. Överensstämmande

Bestämningen av den infallande signalens vinkel _(Angle of Arrival, AoA)_ görs enligt @aoa utifrån respektive fasförskjutning; dels mellan den mellersta och den högra antennen, dels mellan den vänstra och den högra antennen.

Samplingsfrekvensen för CSI är ungefär #qty("230", "Hz") och AoA beräknas lika ofta. Experimentet genomförs med vinklarna $qty("-30", "degree"), qty("-20", "degree"), qty("-10", "degree"), dots.c, qty("30", "degree")$. AoA beräknas i cirka #qty("10", "s") per vinkel.

// #raw(read("snippets/aoa.rs"), lang: "rust")

//#note([Koden i sin helhet finns på #link("https://github.com/akeamc/gya")[github.com/akeamc/gya]. (Vi ska lista ut ett elegantare sätt att hänvisa till den och kanske bifoga mer av den i rapporten.)])

#pagebreak(weak: true)

= Resultat och analys

// #note([*Till handledaren och opponenterna:* Vår mätdata är konstig och vi ämnar upprepa experimentet för olika vinklar och avstånd, i ett större rum (Åke har ett klaustrofobiskt litet rum). Vi har inte testat systemet utomhus heller, men det planerar vi eventuellt att göra.])

På kanal 52, med bandbredden 40 MHz (vilket innebär att frekvenserna #qtyrange("5250", "5290", "MHz", delimiter: "\"till\"") används), och med signalkällan ("enheten", en MacBook Air (2020)) #qty("3.0", "m") från routern, avvek typvärdet för $theta$ från det faktiska värdet, #qty("0", "degree"), med som mest #qty("10", "degree") under experimentet. Medelvärdet för $theta$ under hela experimentet var #qty("0.244", "degree") och standardavvikelsen var $#qty("11.9", "degree")$.

#figure(image("results/2d.png"), caption: [Histogram över beräknad AoA för varje underbärare över tid. Vita områden har lägst frekvens #footnote([Frekvens som i förekomst -- *inte* fysikalisk frekvens.]) <statfreq> (noll) och blå har högst.]) <2d_histogram>

#figure(image("results/0deg.png"), caption: [Histogram över hur ofta varje $theta$ beräknades, det vill säga summan av frekvenserna @statfreq i @2d_histogram tidvis (horisontellt i diagrammet).])

Diagrammen ovan visar detaljerad data om beräknad AoA då den verkliga vinkeln var #qty("0", "degree"). Mätningarna upprepades för flera vinklar; resultatet återfinns i @results_tab.

#figure(caption: [Genomsnittliga uppmätta AoA ($overline(theta)$) och standardavvikelser ($sigma$) för olika vinklar mellan routern och enheten.], table(
  columns: 4,
  stroke: (x, y) => if y == 0 { none } else { (x: none, y: 0.5pt) },
  align: (x, y) => if y == 0 { center } else { right },
  table.header(
    [*Faktisk vinkel*],
    [*Avstånd till routern*],
    [$overline(theta)$],
    [$sigma$]
  ),
  [#qty("-30", "degree")],
  table.cell(rowspan: 7, align: center + horizon)[#qty("3,0", "m")],
  [#qty("-31.2", "degree")],
  [#qty("13.2", "degree")],
  
  [#qty("-20", "degree")],
  [#qty("-22.0", "degree")],
  [#qty("10,3", "degree")],
  
  [#qty("-10", "degree")],
  [#qty("-9.4", "degree")],
  [#qty("12.5", "degree")],
  
  [#qty("0", "degree")],
  [#qty("0.2", "degree")],
  [#qty("11.9", "degree")],

  [#qty("10", "degree")],
  [#qty("9.4", "degree")],
  [#qty("11.5", "degree")],
  
  [#qty("20", "degree")],
  [#qty("20.2", "degree")],
  [#qty("10.9", "degree")],
  
  [#qty("30", "degree")],
  [#qty("30.6", "degree")],
  [#qty("12.7", "degree")],
)) <results_tab>

#pagebreak(weak: true)

= Diskussion

//== Tolkning

#note([*Jalle, kolla igenom detta avsnitt.*])

#strike([Den precision som gick att uppnås med metoden var alltid inom tio grader av vinkeln mellan avsändare och mottagare.]) 

Ju närmare en wifi-enhet är till en router, desto högre precision uppnås vid lokalisering av enheten (se @ideal_setup). Eftersom platsbestämningen oftast kommer ske inomhus i närhet av en wifirouter innebär en felbedömning på (det värde Åke tycker vi borde skriva här) endast ett fel på ett par meter. Många rum har redan wifirouter och det skulle därför inte krävas en stor investering i hårdvara för att omprogrammera dem i statistiksyfte. Det mest effektiva sätt att samla in platsdata på detta vis vore ifall de från början var programmerade att avläsa avsändares position. Detta anser vi inte är helt orimligt ekonomiskt, eftersom det gick att utföras av två obetalda gymnasieelever på ett par månader. 

Precisionen som gick att uppnås med wifiroutern Asus RT-AC86U är dock alldeles för låg för att denna metod ska kunna användas vid platsbestämning av wifi-klienter utomhus. För att kompensera för en precision på (det värde Åke anser vi borde ha) krävs många wifi-routrar. Eftersom ytterst få wifiroutrar redan finns utomhus innebär detta att en mycket stor investering i hårdvara är nödvändig för att utvinna användbara resultat. Å andra sidan är sikten utomhus vanligen friare, vilket innebär färre störningselement för underbärarna och på så sätt att fler underbärare träffar antennsystemet direkt, utan att först reflekteras, och på så sätt en högre precision.

#figure(caption: [Platsbestämning av $O$ med hjälp av två routrar $A$ och $B$ i samma utförande som i detta experiment. Felmarginalen om #qty("10", "degree") begränsar $O$:s position till en fyrhörning (ljusgrön i figuren).], cetz.canvas({
  import cetz.draw: *

  let (a, b) = ((-3,0), (3, 0))

  set-style(circle: (radius: 2pt, fill: black, stroke: none), stroke: 0.5pt)
  
  circle(a, name: "a")
  circle(b, name: "b")
  content("a.west", anchor: "east", padding: 3pt, $A$)
  content("b.east", anchor: "west", padding: 3pt, $B$)

  let cone = (from, to) => {
    line(from, to, stroke: (dash: "dashed", paint: green, thickness: 1pt))
    group({
      rotate(10deg, origin: from)
      line(from, to)
    })
    group({
      rotate(-10deg, origin: from)
      line(from, to)
    })
  }

  intersections("i", {
    cone(a, (3,4))
    cone(b, (-3,4))
  })

  line("i.8", "i.7", "i.10", "i.11", close: true, fill: green.transparentize(80%))
  content("i.2", $O$)
})) <ideal_setup>

// == Validitet, reliabilitet och felkällor

Vid en första anblick är det inte konstigt att förvänta sig att alla underbärare ska nå antennerna från samma håll och därmed ge samma värde för $theta$. OFDM _förutsätter_ i själva verket att de olika underbärarna tar olika väg, eftersom olika frekvenser fortplantas på olika vis vid reflektion och refraktion mot olika material, och så vidare, (se @ofdm) vilket visserligen går tvärt emot undersökningens syfte, men å andra sidan kan det användas till att analysera fysiska förhållanden i rummet. #cite(<schafer2021human>, form: "prose") och #cite(<wifivision>, form: "prose") använde till exempel CSI för att känna igen rörelsemönster hos människor enbart baserat på hur en wifisignals karaktär ändras när människorna i rummet rör sig.

Routern stod nära en vägg, i stället för fritt i rummet, vilket medför höga risker för att signalen reflekteras mot väggen och _ytterligare_ ökar antalet "felriktade" underbärare.

//== Tidigare forskning

#pagebreak(weak: true)

= Slutsats

#note([*... och detta.*])

 Med wifiroutern Asus RT-AC86U går det att uppnå en precision där den genomsnittliga uppmätta AoA mellan router och wifi-enhet skiljer sig #qty("0.244", "degree") från den faktiska vinkeln samt har en standardavvikelse på $#qty("11.9", "degree")$.




#pagebreak(weak: true)

#bibliography(style: "apa", "bibliography.bib")


/*
Jarl:

Abstract

Tillkännagivanden

1.1 Presentation

1.2 Syfte 

1.3 Frågeställning (Kräver asvstämning med Åke)

3.2 Mätmetod

3.3 Avgränsningar (kräver avstämning med Åke)

4 Resultat och analys (Kräver utförande av experiment) 

5.1 Tolkning (kräver utförande av experiment)

5.2 Validitet, reliabilitet och felkällor: (Utförande önskas)
Nämn problem med den plats vi är på (tjocka väggar som ökar reflektionen och gör problemet värre)

5.3 Tidigare forskning

6 Slutsats








*/