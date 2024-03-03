#import "template.typ": *
#import "@preview/cetz:0.2.0"
#import "@preview/tablex:0.0.8": tablex, rowspanx, colspanx
#import "@preview/physica:0.9.2": hbar
#let note(body) = box(stroke: red, inset: 0.5cm, body)
#show: project.with(
  title: "Passiv lokalisering av wifiklienter",
  authors: (
    "Åke Amcoff",
    "Jarl Åkesson",
  ),
  sammanfattning: [
    //En sammanfattning av hela arbetet (syfte, metod, resultat och slutsatser) på högst en halv sida. En läsare ska kunna förstå vad ni har undersökt, hur ni gjort det och vilka resultat ni kommit fram till genom att bara läsa sammanfattningen.

    Hela tiden platsbestämns vi, och platsdata samlas in av appar som Google Maps för att till exempel mäta mängden trängsel på restauranger eller fordonstrafik. Denna bestämning av wifi-klienters position sker genom GPS-system inuti enheterna där deras position avgörs genom trilaterering. Problemet med denna metod är att avsändarna själva avgör när de vill skicka in information till en databas. Platsbestämning av enheter utan detta sjärrapporteringsbehov skulle möjliggöra mer omfattande statistik angående platsdata. Syftet med detta gymnasiearbete är att undersöka hur rimlig denna form av lokalisering av wifi-klienter är gällande ekonomi samt vilken precision som kan uppnås.

    För att undersöka detta omprogrammerade vi mjukvaran i en Asus RT-AC86U wifi-router i syfte att mäta fasförskjutningen av signaler från en wifi-klient mellan routerns tre externa antenner och på så sätt beräkna precisionen möjlig vid triangulering med denna router -- som är en tämligen typisk "off-the-shelf"-router, vilket är avgörande för om ett system baserat på denna rapport kan implementeras i existerande wifinätverksinstallationer.  

    Den resulterande precisionen (#sym.plus.minus~10#sym.degree) anses vara tillräckligt god för inomhusbruk, där avstånden mellan router och klient i regel är relativt korta, men inte utomhus.

    
  ],
  abstract: [
    // Sphinx of black quartz, judge my vow.

    #note([Vi kan inte engelska än men Jarl har en amerikansk kompis som kanske kan hjälpa oss. Vi håller tummarna.])
  ],
  acknowledgements: [
    //En kall novembereftermiddag följde vår käre vän Vidar Nordqvist med Åke Amcoff till Hökarängen för att köpa en begagnad router på Blocket. Tack.

    Genom Norrnässtiftelsen blev vi tilldelade pengar från Södra Latin av summan 1~000 kronor. Utan detta hade inköp av nödvändiga materiel varit omöjlig.

    Genom sitt överseende tillät Jenny Alpsten programmering av gymnasiearbetet under ett flertal mattelektioner. Utan detta är det oklart ifall ett genomförande av vårt experiment hade varit möjligt inom tilldelad tidsram. 

    Vidar Nordqvist var en kämpe och gav emotionellt stöd vid inköp av wifi-routern Asus RT-AC86U från Blocket. Med rädslan närvarande vid inköp genom Blocket i åtanke, var detta starkt och hjälpsamt. 
  ],
  date: "16 februari 2024",
)

#let format(number, precision: 2, decimal_delim: ",", thousands_delim: str(sym.space.nobreak)) = {
  let integer = str(calc.floor(number))
  if precision <= 0 {
    return integer
  }

  let value = str(calc.round(number, digits: precision))
  let from_dot = decimal_delim + if value == integer {
    precision * "0"
  } else {
    let precision_diff = integer.len() + precision + decimal_delim.len() - value.len()
    value.slice(integer.len() + 1) + precision_diff * "0"
  }

  let cursor = 3
  while integer.len() > cursor {
    integer = integer.slice(0, integer.len() - cursor) + thousands_delim + integer.slice(integer.len() - cursor, integer.len())
    cursor += thousands_delim.len() + 3
  }
  integer + from_dot
}

#let scientific(number, precision: 2) = {
  let exp = calc.floor(calc.log(number))
  let sig = format(number / calc.pow(10, exp), precision: precision)

  if exp == 0 {
    return sig
  }

  $sig dot 10^exp$
}

#let qty = (scalar, unit, precision: 2) => {
  scientific(scalar, precision: precision) + sym.space.nobreak + unit
}

//#let que(body) = [#highlight(body)#super(emoji.quest)]

#let rtac86u_d = qty(0.088, "m", precision: 1)

= Inledning

== Presentation

Sedan början av 1970-talet har GPS (Global Positioning System) blivit en alltmer använd teknologi. Det fungerar genom att ett flertal satelliter skickar ut information cirka 20 kilometer från jorden med hjälp av transpondrar om sin exakta tid och position. GPS-mottagaren får då in information från flera satelliter och kan genom _trilaterering_ räkna ut exakt var den är. Trilaterering är ett sätt att fastställa en enhets position genom att mäta avståndet från enheten till tre eller fler andra enheter. Eftersom det finns minst fyra satelliter ovanför ett GPS system fungerar detta alltid. 

Idag platsbestäms vi hela tiden, och platsdata samlas in _en masse_ av appar som Google Maps för att till exempel uppskatta hur trafikerad en väg är eller hur trång en restaurang är -- i realtid. Denna funktion bygger på självrapportering. wifi-enheter vet genom GPS sin position och ger den informationen till Google som sedan drar slutsats om trafikering. Eftersom denna metod är beroroende av självrapportering avgör enheten själv när dess information bör sändas till en databas. Detta skiljer sig från _passiv lokalisering_ där en databas genom mottagare avgör vart en avsändare är. 

Fastställning av position är dock inte bara möjligt genom trilaterering. Istället för distans och tid kan det genom _triangulering_ platsbestämmas. Detta är en ny metod där vinkeln mellan avsändare och minst tre mottagare mäts. Ifall mottagarnas exakta positioner är kända kan avsändarens plats bestämmas. Eftersom tid inte är en variabel som beräknas är atomklockor inte nödvändiga för platsbestämning genom triangulering vilket gör denna metod billigare och enklare för undersökningar på en mindre skala än dylik trilatereringsmetod. 

// https://www.lantmateriet.se/en/geodata/gps-geodesi-och-swepos/gps-and-satellite-positioning/gps-and-other-gnss/gps/

== Syfte

Syftet med detta gymnasiearbete är att undersöka ifall passiv platsbestämning av wifienheter är träffsäkert och rimligt. Ifall detta är rimligt skulle det innebära möjligheter för större insamling av data att användas under effektivisering av stadsplanering samt egentid. 

Problemet med _aktiv platsbestämning_ är att den är beroende av självrapportering. Alltså avgör enheter själva när information om deras position är lämplig att sända till en databas. På grund av att enheterna själva avgör när platsbestämning är lämpligt ger inte detta lika omfattande statistik som platsbestämning utan aktiv tillåtelse. Ifall hög precision kan uppnås genom passiv platsbestämning skulle detta ha många tillämpningsområden inom effektivisering av stadsplanering. 

Den mer omfattande mängden data skulle innebära enklare identifiering av trafikering. Butiksägare och restaurangsägare kan då bättre optimera öppetider och marknadsföring samt sina fastigheters geografi för att minska konkurrens samt öka tillgänglighet. Det skulle också göra livet enklare för konsumenter, eftersom de alltid hade vetat väntetider för restauranger eller trängsel i butiker och mataffärer vilket skulle innebära en minskning i slöseri av tid. Trafikering avseende fordonstrafik skulle också vara ännu tydligare vilket hade möjliggjort  mer effektiv infrastruktur, eftersom problematiska områden och tidpunkter då är mer uppenbara. 

Problematiska områden avseende smittspridning vore också enklare att identifiera. Vid utbrott av aggressivt smittsamma sjukdomar skulle mer effektiv samt större mängd platsdata möjliggöra för områdesspecifika restriktioner och då minska behovet av omfattande restriktioner samt mängden drabbade av sjukdom. Privatpersoner hade då mer enkelt kunnat ta del av nödvändiga aktiviteter som inhandling av livsmedel samt mindre nödvändiga aktiviteter som restaurangbesök vilket hade lett till att färre företag måste stänga ner eller göra nedskärningar.  

== Frågeställning

- Vilken precision kan uppnås av passiv lokalisering av wifiklienter gällande platsbestämning?
- Hur rimlig är tillämpning av dylik metod i statistiksyfte avseende ekonomi? 

#pagebreak(weak: true)

= Bakgrund

== Wifi genom tiderna

OSI-modellen är ett försök att kategorisera standardena för kommunikation datorer sinsemellan, och tanken är att varje _lager_ ska vara helt oberoende av de andra lagren och att lagren ska kunna kombineras obehindrat. @iso7498 HTTPS (HTTP Secure), till exempel, heter _secure_ eftersom det är krypterat, men namnet är missvisande eftersom informationen inte krypteras i HTTP(S)-lagret (lager 7) utan i lager 6.

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
), caption: [OSI-modellens skikt.])

Lager 1 överför bitar genom den fysiska världen. IEEE 802.3 (Ethernet) och IEEE 802.11 (Wifi) hör till de mest kända samlingarna av standarder; IEEE 802.3 standardiserar bitöverföringen i kopparkablar och fiberoptik medan IEEE 802.11 standardiserar trådlös överföring.

#figure(kind: table, tablex(
  columns: 3,
  map-hlines: h => (..h, stroke: if h.y == 0 { 0pt } else { 0.5pt }),
  map-vlines: v => (..v, stroke: 0.5pt),
  auto-vlines: false,
  [*Wifigeneration*], [*IEEE-standard*], [*Antagen*],
  [Wifi 8], [802.11bn], [2028],
  [Wifi 7], [802.11be], [2024],
  [Wifi 6], [802.11ax], [2019],
  [Wifi 5], [802.11ac], [2014],
  [Wifi 4], [802.11n], [2008],
  [(Wifi 3) #footnote([Wifi 0 #sym.dots.c 3 är inofficiellt och retroaktivt namngivna.]) <wifiretroactive>], [802.11g], [2003],
  [(Wifi 2) @wifiretroactive], [802.11a], [1999],
  [(Wifi 1) @wifiretroactive], [802.11b], [1999],
  [(Wifi 0) @wifiretroactive], [802.11], [1997],
), caption: [IEEE 802.11-standarder och deras konsumentvänliga namn. @wifistandards])

IEEE 802.11 består av en uppsjö standarder från de senaste 30 åren. I vårt experiment studeras IEEE 802.11ac (den näst senast _lanserade_ standarden) av två huvudsakliga skäl: Dels är IEEE 802.11ac-kretsar i skrivande stund vanligt förekommande och relativt billiga jämfört med IEEE 802.11ax-kretsar, dels är verktygen för korrigering av firmware mer testade på de äldre IEEE 802.11ac-kretsarna @csi.

#note([Här ska wifikanaler förklaras. Tills vidare, #link("https://en.wikipedia.org/wiki/List_of_WLAN_channels#5_GHz_(802.11a/h/n/ac/ax")[se Wikipedia].])

== Ortogonal frekvensdelningsmultiplex <ofdm>

Ortogonal frekvensdelningsmultiplex (OFDM) ökar överföringshastigheten genom att signalöverföringen delas in i flera parallella dataströmmar, så kallade underbärare _(subcarriers)_, på varsin frekvens. OFDM gör signalöverföringen tålig mot störningar som drabbar vissa frekvenser, och dessutom kan de olika underbärarna reflekteras på olika sätt så att _majoriteten_ når mottagaren. @ofdmhistory

OFDM används i alla wifistandarder från och med IEEE 802.11a @wifistandards, och alltså även i IEEE 802.11ac, som granskas ingående i denna rapport.

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



/*
#figure(
  caption: [Underbärare som används i IEEE 802.11ac],
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
)
*/

== Fasförskjutning och CSI <csi_explanation>

Moderna wifikretsar använder flera antenner till samma uppkoppling. Tillsammans bildar antennerna ett antennsystem där varje antenn utgör ett så kallat element. Om antennerna är uniformt fördelade längs en linje bildar de ett uniformt linjärt antennsystem (_ULA; Uniform Linear Array_ på engelska); se @interference. Routern som används i experimentet har ett uniformt linjärt antennsystem med tre element (vilket framgår i @rt-ac86u).

En sändare kan införa en viss farförskjutning $Phi$ mellan intilliggande antennelement och utnyttja interferens för att rikta signalen åt ett visst håll (så kallad _beamforming_). En mottagare kan på motsatt sätt bestämma den infallande signalens vinkel $theta$ utifrån den uppmätta fasförskjutningen mellan angränsande element.

För att $theta$ ska kunna bestämmas precist måste signalen i fråga träffa alla antennelement från samma vinkel, och källan måste alltså befinna sig på ett tillräckligt stort avstånd från antennsystemet så att strålarna blir någorlunda parallella.

#figure(cetz.canvas(length: 1cm, {
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
      content((3,0), [$n=#n$], anchor: "west", padding: 3pt)
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
  caption: [Uniformt linjärt antennsystem med tre element. Den infallande signalen har vinkeln $theta$ och källan antas befinna sig på ett så stort avstånd $D$ från antennerna att signalen träffar alla antennelement från ungefär samma vinkel ($D>>d$).],
) <interference>

Betrakta den rätvinkliga triangel vars hypotenusa (med längden $d$) går mellan två intilliggande antenner och vars ena katet är ortogonal mot den infallande signalen. Triangelns andra katet kommer ha längden

$ x=d sin theta, $

vilket motsvarar signalens vägskillnad. Således är den observerade fasförskjutningen mellan två intilliggande antenner

$
Phi=(2pi d sin theta)/lambda.
$ <phaseshift>

Utifrån @phaseshift beräknas den infallande signalens vinkel enligt

$ theta=arcsin((Phi lambda)/(2pi d)). $ <aoa>

Fasförskjutningen kan härledas från data om signalförhållandena, så kallad _Channel State Information (CSI)_, som rapporteras kontinuerligt av wifikretsen för varje ansluten enhet. Från varje antenn fås en vektor med komplexa tal

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
) quad upright("respektive") quad C = vec(
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
    label: text(green, [$Phi$]), inner: false)
  
  set-style(
    angle: (
      radius: 0.3,
      label-radius: .22,
      fill: blue.lighten(80%),
      stroke: (paint: blue.darken(50%))
    ),
  )
  
  cetz.angle.angle((0,0), c, d, inner: true,
    label: text(blue, [$Phi'$]))

  line((0,0), a)
  line((0,0), b)
  line((0,0), c)
  line((0,0), d)

  circle(a, radius: 0.1cm, fill: green, stroke: none)
  circle(b, radius: 0.1cm, fill: green, stroke: none)
  circle(c, radius: 0.1cm, fill: blue, stroke: none)
  circle(d, radius: 0.1cm, fill: blue, stroke: none)
})
}, caption: [Två olika par av faser, förskjutna lika mycket från varandra, men som ger olika observerade fasförskjutningar. Notera att $arg(z) in underbrace((-pi, pi], upright("Inte") [0, 2pi)) forall z in bb("C")$.]) <d_eq_lambda>

Fasförskjutningen $Phi_k$ mellan två intilliggande antenner för den $k$:te underbäraren är differensen mellan motsvarande faser $alpha_k$ och $beta_k$, och ligger i intervallet $(-2pi, 2pi)$. Uttrycket i @aoa har definitionsmängden $[-(2pi d)/ lambda, (2pi d)/lambda]$ för $Phi$ och för att kunna beräkna ett entydigt värde på $theta$ för alla vinklar mellan $-90 degree$ och $90 degree$ måste därför $d<=lambda$. Med $d=lambda$ är dock ett $Phi_k$-värde nära ytterkanterna av värdemängden osannolikt, eftersom det förutsätter att de två faserna $alpha_k$ och $beta_k$ _också_ har värden nära ytterkanterna ($-3$ och $3$, till exempel; se @d_eq_lambda) -- egentligen måste

$ d <= lambda/2 $ <unambig_aoa>

för att fasförskjutningarna $Phi_1 dots.c space Phi_n$ ska vara korrekta för alla $theta in (-pi, pi)$.

#pagebreak(weak: true)

= Metod

Genom att modifiera routerns maskinvara kan CSI erhållas @csi: Programvaru-?modifikations-?ramverket Nexmon är testat med programvaruversion `10_10_122_20`, så routerns programvara nedgraderades. Därefter fördes `tcpdump` och Nexmons modifierade kernelmodul över till routern.

CSI-insamlingen görs genom att radion ställs in på samma kanal som enheten som ska spåras. Därefter genereras nätverks-?trafik, vars CSI rapporteras av wifikretsen, med nätverkshastighetsmätverktyget iPerf #footnote([iPerf3 körs med ett lågt _Maximum Segment Size_-värde för att generera så många wifiramar som möjligt, eftersom varje wifiram ger upphov till CSI.]). På routern samlas _pcap-paket_ in med programmet `tcpdump` in och skickas till en dator för analys.

#figure(
  caption: [Informationen som finns i ett Pcap-paket från routern. @csi],
  kind: table,
  tablex(
    columns: 2,
    auto-vlines: false,
    map-hlines: h => (..h, stroke: if h.y == 0 { 0pt } else { 0.5pt }),
    //align: (x, y) => if y == 0 or x == 1 { left } else { right },
    [*Fält*],
    [*Förklaring*],
    [RSSI (dBi)],
    [Signalstyrka],
    [Avsändarens MAC-adress],
    [Serienummer för avsändarens nätverksutrustning],
    [Wifiramnummer],
    [Unikt nummer för wifiramen som gav upphov till detta CSI-fragment],
    [Antennummer],
    [Antennummer i intervallet $[0,3]$],
    [CSI],
    [CSI-vektor (se @csivec)],
  )
) <inside_pcap>

Ett pcap-paket genereras per antenn och wifiram och innehåller bland annat CSI (fullständig innehållsförteckning finns i @inside_pcap), som grupperas med andra pcap-paket utifrån ramnummer för att faserna från vektorerna $A$, $B$ och $C$ (@csivec) ska kunna jämföras.

Bestämningen av den infallande signalens vinkel _(Angle of Arrival, AoA)_ görs enligt @aoa utifrån respektive fasförskjutning; dels mellan den mellersta och den högra antennen, dels mellan den vänstra och den högra antennen.

Samplingsfrekvensen för CSI är ungefär #qty(230, "Hz", precision: 1) och AoA beräknas lika ofta.

#raw(read("snippets/aoa.rs"), lang: "rust")

#note([
  Koden i sin helhet finns på #link("https://github.com/akeamc/gya")[github.com/akeamc/gya]. (Vi ska lista ut ett elegantare sätt att hänvisa till den och kanske bifoga mer av den i rapporten.)
])


== Materiel

Huvudkomponenten i lokaliseringssystemet är wifi-routern Asus RT-AC86U, som används eftersom dess wifi-krets, Broadcom BCM4366, stödjs av CSI-extraheringsverktyget Nexmon @csi. Dessutom har samma router använts i tidigare liknande experiment @ubilocate @schafer2021human @meneghello2022wi. Dess antennavstånd $d$ (se @interference och @aoa) är #rtac86u_d. Antennerna är inte riktade åt olika håll, som @rt-ac86u visar, utan åt samma håll (de är vridbara).

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
De ekonomiska medeln tillgängliga för vårt förfogande kom från Norrnässtiftelsens stipendium vid summan ett tusen kronor. Nio hundra av dessa kronor användes vid inköp av en Asus RT-AC86U wifi-router genom Blocket. wifi-routern Asus RT-AC86U kan bara mäta platta vinklar eftersom routerns antennsystem är endimensionellt samt ortogonalt mot horisontalplanet (se @interference). Detta begränsar vår möjlighet att anlända vid en fullkomlig slutsats eftersom en wifi-enhets relation i höjd till routern inte går att mäta. Till exempel går det inte att avgöra på vilken våning av en byggnad en viss signal kommer ifrån. 

Eftersom vi endast har en router att använda till experimentet är det inte heller möjligt att platsbestämma en avsändare genom dess vinklar mot en mängd andra mottagare eftersom triangulering kräver insamling av data från tre eller fler mottagare. I stället mäts precisionen av vinkeln mellan avsändare och routern. Precisionen som uppnås genom detta visar på precisionen platsbestämning genom triangulering med tre eller fler av dessa wifi-routrar skulle vara. En större budget – cirka tre tusen kronor – hade behövts för att kunna genomföra triangulering. 

På grund av ekonomiska begränsningar samt den stora mängd programmering nödvändig för ett fullständigt genomförande av detta experiment är det uppenbart att en mer omfattande investering krävts. Tid var i synnherhet begränsande.

Med $d=#qty(0.088, $upright("m")$, precision: 1)$ och $lambda = c/ #qty(5e9, "Hz", precision: 0) approx #qty(0.06, $upright("m")$, precision: 0)$ är olikheten i @unambig_aoa (som måste uppfyllas för att vinklar mellan $-90 degree$ och $90 degree$ ska kunna mätas) falsk. Bättre mätdata hade kunnat samlas in ifall antennerna var ännu närmare varandra än originellt på routern. Planer om avskruvning av antennerna samt byggnation av ställningar för dessa behövde dock läggas ner efter insikt om tidsbrist och prioritering av rapport och genomförande. 

#pagebreak(weak: true)

= Resultat och analys

#note([*Till handledaren och opponenterna:* Vår mätdata är konstig och vi ämnar upprepa experimentet för olika vinklar och avstånd, i ett större rum (Åke har ett klaustrofobiskt litet rum). Vi har inte testat systemet utomhus heller, men det planerar vi eventuellt att göra.])

På kanal 52, med bandbredden 40 MHz (vilket innebär att frekvenserna 5250–5290 MHz används), och med 3,0~m mellan signalkällan och routern, avvek $theta$ från det faktiska värdet med som mest 10 grader.

#figure(image("aoa.png"), caption: [Histogram över beräknad AoA för varje underbärare. Typvärdet är markerat med ljusblått.])

#pagebreak(weak: true)

= Diskussion

//== Tolkning

Den precision som gick att uppnås med metoden var alltid inom tio grader av vinkeln mellan avsändare och mottagare. Eftersom platsbestämningen oftast kommer ske inomhus i närhet av en wifi-router innebär en missbedömning på tio grader endast ett fel på ett par meter. Detta tolkar vi som en tillräckligt hög precision för den skala vi utförde experimentet på, särskilt med alla begränsningar i åtanke. Eftersom de flesta rum är försedda med en wifi-router skulle knappt någon investering i hårdvara krävas för att omprogrammera dem i statistiksyfte. Det mest effektiva sätt att samla in platsdata på detta sätt vore ifall de från början var programmerade att avläsa avsändares position. Detta anser vi rimligt ekonomiskt, eftersom det gick att utföras av två obetalda gymnasieelever på ett par månader. 

Detta visar alltså att det går att uppnå en rimlig precision med passiv lokalisering av wifi-klienter genom triangulering samt att det är rimligt ekonomiskt inomhus. 

Den precisionen som gick att uppnås är dock alldeles för låg för att användas vid platsbestämning utomhus. Eftersom ytterst få wifi-routrar finns utomhus blir felmarginalen på 10 grader stor. Alltså skulle väldigt många wifi-routrar behöva placeras utomhus vilket i sin tur hade innebärt en stor investering i hårdvara. Å andra sidan är sikten utomhus vanligen friare, vilket innebär färre störningselement för underbärarna och på så sätt att fler underbärare träffar antennsystemet direkt, utan att först reflekteras, och på så sätt en högre precision.

// == Validitet, reliabilitet och felkällor

Vid en första anblick är det inte konstigt att förvänta sig att alla underbärare ska nå antennerna från samma håll och därmed ge samma värde $theta$. OFDM _förutsätter_ i själva verket att de olika underbärarna tar olika väg (se @ofdm) vilket visserligen går tvärt emot undersökningens syfte, men å andra sidan kan det användas till att analysera fysiska förhållanden i rummet. #cite(<schafer2021human>, form: "prose") använder till exempel CSI för att känna igen rörelsemönster hos människor enbart baserat på hur en wifisignals karaktär ändras beroende på hur människorna i rummet rör sig.

Routern stod nära en vägg, i stället för fritt i rummet, vilket medför höga risker för att signalen reflekteras mot väggen och _ytterligare_ ökar antalet "felriktade" underbärare.

//== Tidigare forskning

#pagebreak(weak: true)

= Slutsats

Vår slutsats är att passiv lokalisering av wifi-klienter är rimligt inomhus. Det finns nog med wifi-routrar för att platsbestämning med en felmarginal på tio grader ger pålitlig platsdata. Mängden routrar innebär också att det inte krävs stor investering i hårdvara för att insamling av platsdata ska leda till omfattande statistik. Dock är passiv lokalisering av wifi-klienter orimligt utomhus. Det skulle krävas ett stort antal wifi-routrar för att kompensera för en felmarginal på tio grader vilket innebär enorm kostnad eftersom det knappt finns några wifi-routrar utomhus. 

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