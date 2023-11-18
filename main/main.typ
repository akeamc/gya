#import "template.typ": *

#show: project.with(
  title: "Passiv lokalisering av wifiklienter",
  authors: (
    "Åke Amcoff",
    "Jarl Åkesson",
  ),
  sammanfattning: [
    En sammanfattning av hela arbetet (syfte, metod, resultat och slutsatser) på högst en halv sida. En läsare ska kunna förstå vad ni har undersökt, hur ni gjort det och vilka resultat ni kommit fram till genom att bara läsa sammanfattningen.
    
    I gymnasiearbetet ska sammanfattningen skrivas på både svenska och engelska (abstract).
  ],
  abstract: [
    The quick brown fox jumps over the lazy dog.
  ],
  date: "24 november 2023",
)

= Inledning

Hela tiden platsbestäms vi av (etc) bland annat. Detta genom att våra mobiler själva rapporterar sin position gentemot (något fancy ord). (Etc etc lite bakgrund och historia). Med hjälp av detta vet appar som Google Maps (etc) vart vi är och kan hjälpa lösa diverse problem. Men, tänk ifall Google Maps redan visste vart alla var? Skulle inte det göra (någonting) enklare och mer effektivt? Är inte allas positioner värdefull data som kan användas för förbättring av (etc) och (nånting nånting miljö)? Det skulle bli enklare att se när och var det är mycket trafik vilket kan förbättra våra vägar (etc etc) vilket skulle ha någon positiv effekt för miljön samt göra det enklare att hantera pandemier? 

== Syfte

Att undersöka ifall platsbestämning med hjälp av wifienheter är träffsäkert och rimligt. (bla bla bla)

== Frågeställning

Vilken precision kan uppnås vad gäller platsbestämningen?

== Bakgrund

= Metod

== Materiel

Routern Asus RT-AC86U används i gymnasiearbetet eftersom dess wifi-chip, Broadcom BCM4366, stödjs av CSI-extraheringsverktyget Nexmon @nexmon. Dessutom har samma router använts i liknande föregående experiment @ubilocate.

== Mätmetod

Vi gjorde detta under dessa dagarna och testar dessa olika faktorer av dessa anledningar. 

=== Avgränsningar

Budget. Vad som kan undersökas med de medel vi har och vad det kan ge för slutsats. Platsbestämning av platsbestämningen (IE skolen eller hemma hos Åke eller Åkesson). 

= Resultat och analys

Vi fick detta resultat i denna fina och tydliga tabell.

Vi analyserar på vilket sett våra faktorer påverkade våra resultat. 

= Diskussion

== Tolkning

== Validitet, reliabilitet och felkällor

== Tidigare forskning

= Slutsats

= Tillkännagivanden

Vidar 2 Nordqvist kom på att samma antenn kan användas i flera arrays en kall novemberdag i Hökarängen. Tack.

Hej. @indoorblwifi @ubilocate

#bibliography(style: "apa", "bibliography.bib")
