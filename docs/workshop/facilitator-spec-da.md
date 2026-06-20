# Facilitator-spec — Specifikationsdrevet design (workshop hos Clever)

> **Hvad dette dokument er.** Dette er *What* for workshoppen: agendaen, intentionen bag hvert segment og talepunkterne. Slide-decket (`spec-drevet-design-workshop-da.pptx`) er én *How* — én realisering af denne spec. En anden facilitator kan afvikle sessionen ud fra dette dokument alene. **Hver eneste slide har en reference herinde** (sektion + intention).
>
> **Kilde-disciplin:** intentionen for hver slide bor i slidens egen talenote i decket. Dette dokument er *udtrukket* derfra — det er en projektion, ikke en parallel kilde. Skal en slides intention ændres, ændres talenoten i decket, og dette dokument genereres på ny. Sådan kan de to aldrig drive fra hinanden. Det er præcis What/How-disciplinen fra workshoppen anvendt på workshoppen selv.

| | |
|---|---|
| **Varighed** | 12:00–15:50 (~3 timer inkl. pauser) |
| **Format** | Teknisk session — devs og PO'er |
| **Serie** | Næste i serien om LLM'er i udvikling |
| **Gennemgående eksempel** | E-handels-ordresystem |
| **Deck (How)** | `spec-drevet-design-workshop-da.pptx` — 34 slides |

**Definition of done for sessionen:** deltagerne kan (1) forklare What/How-snittet og hvorfor det skaber tillid, (2) bygge en domæne- + event-model for et af deres egne systemer i product-cli, og (3) placere deres nuværende måde at bruge LLM'er på i de fem modes og forstå blast-radius-argumentet for vores tilgang.

---

## Run-of-show

| Tid | Segment | Varighed | Slides |
|---|---|---|---|
| 12:00 | **What-talk:** fra LLM til specifikation | 50 min | 1–21 |
| 12:50 | Pause | 10 min | — |
| 13:00 | **Øvelse 1:** modellér jeres eget system | 45 min | 22 |
| 13:45 | Pause | 10 min | — |
| 13:55 | **How-talk:** fem måder at bruge LLM'er | 50 min | 23–28 |
| 14:45 | **SPMC:** fra spec til eksekvering | 30 min | 29–31 |
| 15:15 | **Øvelse 2:** byg et SPMC-bundle | 25 min | 32 |
| 15:40 | Afrunding & adoption | 10 min | 33–34 |

> Pauser (12:50 og 13:45) har ingen slides og er bevidst udeladt af slide-mappingen.

---

## Slide-indeks (komplet)

Hurtig reference: hver slide → segment. Detaljerede intentioner følger under hvert segment.

| Slide | Label | Segment |
|---|---|---|
| 1 | Åbning | 1 |
| 2 | Agenda | 1 |
| 3 | Hvor vi slap | 1 |
| 4 | Hvad en LLM er | 1 |
| 5 | Målet — autonomi-stigen | 1 |
| 6 | Flaskehalsen er tillid | 1 |
| 7 | Tillid via integritet | 1 |
| 8 | Integritet = præcision + transparens | 1 |
| 9 | What-spec er svaret | 1 |
| 10 | Sektionsskift: så hvordan ser det ud | 1 |
| 11 | Hvem ejer What (for PO'erne) | 1 |
| 12 | What — struktur & adfærd | 1 |
| 13 | Rækkefølgen: domæne først, så events | 1 |
| 14 | Event-modellering — eksempel | 1 |
| 15 | Hvorfor event-modellering passer til flows | 1 |
| 16 | Den første disciplin: adskil What fra How | 1 |
| 17 | Sektionsskift: What som LLM-input | 1 |
| 18 | Hvad en spec skal være (fem egenskaber) | 1 |
| 19 | What-spec: ni sektioner | 1 |
| 20 | To regler folk bryder | 1 |
| 21 | Gør det med et værktøj (product-cli) | 1 |
| 22 | Øvelse 1 | 2 |
| 23 | Sektionsskift: How | 3 |
| 24 | De fem modes (oversigt) | 3 |
| 25 | Det afgørende skift — hvorfor mode 5 | 3 |
| 26 | Tragt-princippet | 3 |
| 27 | How-specifikationen | 3 |
| 28 | Afledningskontrakten | 3 |
| 29 | Sektionsskift: SPMC | 4 |
| 30 | SPMC — fire elementer | 4 |
| 31 | Hvorfor SPMC betaler sig | 4 |
| 32 | Øvelse 2 | 5 |
| 33 | Sådan samler det sig | 6 |
| 34 | Tak / afslutning | 6 |

---

## 1 · What-talk: fra LLM til specifikation
**12:00–12:50 · slides 1–21**

**Slide 1 · Åbning**
Velkommen tilbage — det her er næste session i serien om LLM'er i udvikling. Dagens bue: opsummér hvor vi slap, fortæl så historien fra hvad en LLM faktisk er, til målet om autonom udvikling, til hvorfor tillid er flaskehalsen, til hvordan tillid opnås — og land på What-specifikationen som den transparens, der skaber den. Derefter viser vi hvordan det ser ud: domænemodellering og event-modellering. Gennemgående eksempel: et e-handels-ordresystem.

**Slide 2 · Agenda**
Agenda-slide. Gå kort run-of-show igennem. Nævn de to øvelser og pauserne, så rummet kender rytmen.

**Slide 3 · Hvor vi slap**
Hold det let og generisk — en 2-minutters genorientering. Vi underviser ikke på ny; vi minder folk om trappen, vi har bestiget, og signalerer at dagens trin er overdragelses-artefakten.

**Slide 4 · Hvad en LLM er**
Den centrale omformulering: en LLM er ikke en database eller en regelmotor — den er en forudsiger, ligesom hjernen. Derfor er kontekst alt. En dygtig person med den forkerte brief bygger det forkerte, med selvtillid. Det samme gælder modellen. Det sætter resten af oplægget op: hvis input er det eneste greb, er det overdragelses-artefakten, der betyder noget.

**Slide 5 · Målet — autonomi-stigen**
Stigen er afledt af SAE J3016, tilpasset AI. Pointen er ikke at huske niveauer — den er, at gevinsten er N4/N5, hvor mennesker holder op med at inspicere hver handling. Sæt ord på, hvor rummet er i dag (sandsynligvis N2–N3). Hele oplægget handler om, hvad der skal til for at bestige næste trin sikkert.

**Slide 6 · Flaskehalsen er tillid**
Bro-slide. Kapabilitet løber forud; det, der begrænser autonomi, er tillid. På N3 er mennesket tillidsmekanismen. På N4+ er mennesket væk fra loopet pr. handling, så tilliden må komme et strukturelt sted fra. Lægger op til næste slide: tillid = integritet.

**Slide 7 · Tillid via integritet**
Omdrejningspunktet for hele indledningen. Hensigt vs. aftale. Hensigtsdrevet prompting føles produktivt, men giver modellen det spillerum, man ville give en betroet senioringeniør — præcis den tillid, vi ikke har etableret. Tillid kommer fra integritet: at gøre det aftalte. Næste slide udfolder integritetens to krav.

**Slide 8 · Integritet = præcision + transparens**
To krav. Præcision = aftalen er præcis (ingen huller for modellen at forudsige ind i). Transparens = vi kan se, at outputtet overholdt aftalen. Sammen er de integritet, og integritet er det, der skaber tillid. Land pointen: der findes én artefakt, der både er den præcise aftale og den transparente registrering af den — What-specifikationen.

**Slide 9 · What-spec er svaret**
Landings-slide for den indledende fortælling. What-spec'en er på én gang den præcise aftale og den transparente registrering — derfor er den mekanismen, tillid opnås igennem. Det er overgangen til resten af blok 1: 'så hvordan ser det ud?' → domænemodellering, så event-modellering. Herfra forbindes direkte til What-model-slides.

**Slide 10 · Sektionsskift: så hvordan ser det ud**
Sektionsskift ind i 'så hvordan ser What ud i praksis'. Ingen tale ud over overgangen — brug den til at skifte gear fra fortælling til konkret modellering.

**Slide 11 · Hvem ejer What (for PO'erne)**
Ramme for PO'erne i rummet. What er DEN artefakt, hver rolle læser og underskriver — og den tilhører produkt-og-design, ikke udvikling. Den er skrevet i forretningssprog og aftalt, før How findes. PO'en ejer, om What er korrekt; udvikling ejer, hvordan den realiseres. Ingen gætter den andens hensigt, fordi snittet er eksplicit. Det er sliden, der fortæller PO'erne 'det her er jeres, og her er formen, det tager.'

**Slide 12 · What — struktur & adfærd**
What har to halvdele i én graf: domænemodellen (struktur — bounded contexts, ubiquitous language, entiteter/relationer/value-objects, invarianter som maskinkontrollerbare constraints, RDF+SHACL) og event-modellen (adfærd — commands, events, read models, UI-steps, alle domæne-typede). Decideren er ikke en tredje sidestillet model; den er den eksekverbare form af adfærd for et aggregate — decide/evolve, signatur afledt af event-modellen, kun logikken forfattet, og den bliver det orakel, realiseret kode tjekkes mod. Der er en symmetrisk Projector for read models. Værktøj: product-cli understøtter hele Product Framework — struktur med 'product domain new context/entity', adfærd med 'product domain new event/command/read-model', og 'product domain validate' + 'product guide' driver én gennem hullerne.

**Slide 13 · Rækkefølgen: domæne først, så events**
Det her er kernen i PO-budskabet. Rækkefølgen betyder noget: lav domænemodellen først, fordi den sætter det fælles ordforråd — den klassiske 'er en User en Customer'-forvirring afgøres én gang, i ubiquitous language, i stedet for at blive taget op igen på hvert møde. SÅ event-modellen, som er domæne-typet, så den fysisk ikke kan referere et begreb, der ikke findes. Den forstærkende gevinst kommer af at holde begge i én graf: et event ændrer altid en entitet, så struktur og adfærd ikke kan drive; 'hvad sker der med dette begreb' er én forespørgsel; og domænemodellen er værnet, der holder flows bygbare. At lave den ene uden den anden mister koblingen; at lave begge, i rækkefølge, er det, der gør What til en levende, forespørgbar beskrivelse frem for to forældede dokumenter.

**Slide 14 · Event-modellering — eksempel**
Gå én række igennem fra ende til ende: PlaceOrder (command) → Decider tjekker invarianter (vare på lager, gyldig kunde) → OrderPlaced (event) → OrderSummary + FulfilmentQueue (read models). Bemærk fejlstien: PlaceOrder kan afvises — det er også en adfærd.

**Slide 15 · Hvorfor event-modellering passer til flows**
Det her er 'event-modellering er fantastisk til flows'-sliden. Fire grunde rettet mod PO'er: (1) læsbar — en tidslinje i forretningssprog, der kan godkendes uden at lære notation; (2) komplet — fejlstier og afvisninger er førsteklasses, og det er der, krav normalt lækker; (3) den bygger bro til skærme — et flow er en tidslinje af interface-steps, der hver viser en projektion og tilbyder gyldige commands, så samme artefakt spænder fra begreb til skærm; (4) den er eksekverbar — domæne-typede steps lader flowet simuleres mod Decideren før kode, og fanger huller som billige funktionskald. Slutlinjen er pointen: én tidslinje er samtidig produktets godkendelse, byggespec'en og test-oraklet.

**Slide 16 · Den første disciplin: adskil What fra How**
Den første disciplin: What og How skal være adskilte artefakter. En sammensmeltet PRD diskvalificerer — kunden kan ikke godkende, og hvert teknisk valg skjuler en uerklæret produktbeslutning. Pointen: snittet er ikke en stilpræference, det er det, der gør resten af metoden mulig.

**Slide 17 · Sektionsskift: What som LLM-input**
Sektionsskift: stadig What-talk, nu ind i What som LLM-input. Brug overgangen til at sige, at en spec ikke er dokumentation — den er struktureret input.

**Slide 18 · Hvad en spec skal være (fem egenskaber)**
De fem egenskaber en spec skal opfylde: utvetydig, komplet ved sin grænse, internt konsistent, maskinlæsbar, eksplicit afgrænset. Pointe: fejler én af dem, er det ikke en spec — det er en opfordring til modellen om at træffe uerklærede produktbeslutninger. Læs dem hurtigt; de er en tjekliste, ikke et foredrag.

**Slide 19 · What-spec: ni sektioner**
Fremhæv Adfærd (fejlstien er halvdelen af adfærden) og Grænser (tom out-of-scope = grænser ikke gennemtænkt). Åbne spørgsmål er det vigtigste output af en discovery-session — en tom liste på et første udkast er et advarselstegn.

**Slide 20 · To regler folk bryder**
To regler folk bryder. (1) Fejlstien er ikke valgfri — en adfærd kun beskrevet ved happy path fortæller modellen, at den aldrig fejler. (2) Et mindre projekt får samme minimum, ikke et lavere — samme struktur, mindre indhold. Disciplinen skalerer ned; strukturen gør ikke.

**Slide 21 · Gør det med et værktøj (product-cli)**
Pointen med hele øvelsen: ingen skriver kommandoer i hånden. De starter Copilot CLI i det udleverede repo (som har en copilot-instructions.md, der lærer den product-cli-grammatikken) og BESKRIVER systemet i almindeligt sprog. Copilot oversætter til product-kommandoer og kører dem; binæren validerer. Vigtig pointe at sige højt: agenten (Copilot) er IKKE en del af product-cli — og det behøver den ikke at være. product-binæren er autoriteten: foreslår Copilot en ugyldig node (event uden rigtig entitet, command uden target/emit), afvises den, og Copilot læser fejlen og retter. Mennesket leverer intention og domæneviden; binæren leverer korrekthed. Kontrasten til venstre (i hånden) er den gamle måde. Forbind til blast-radius: Copilot rører kun product-grafen i den mappe — snævert toolset, lille blast-radius. Den manuelle domain new-vej er kun fallback hvis Copilot driller.

---

## 2 · Øvelse 1: modellér jeres eget system
**13:00–13:45 · slide 22**

**Opgave (grupper):** byg en domænemodel OG en event-model for ét af jeres egne systemer — med product-cli.

**Trin:**
1. Vælg ét afgrænset område af et system, I kender.
2. `product author domain`: navngiv begreber, relationer, invarianter.
3. Afgør de svære ord — ét ubiquitous language pr. kontekst.
4. `product author event`: commands, events, read models, UI-steps.
5. Modellér ét helt flow — inkl. fejlstier — og commit.

**Leverance:** en domæne- og event-model for et rigtigt system — i én graf, schema-valid og klar til at bygge videre på.

**Facilitator-noter:**
45 minutter, deres eget system — ikke ordre-eksemplet. Del handout-arket ud (exercise-1-handout-da.md). Setup: de har et FORBEREDT repo klonet på forhånd med en copilot-instructions.md, der primer Copilot CLI med product-cli-grammatikken. De cd-er ind, starter copilot, og BESKRIVER systemet — Copilot kører kommandoerne. Coach dem til at (1) sige hvad systemet er, (2) lade Copilot foreslå begreber og rette den hvor den tager fejl — her bliver de enige om de svære ord, er-en-User-en-Customer, (3) beskrive ét helt flow inkl. fejlstier, (4) bede den køre product domain validate til grønt. Det vigtige de skal mærke: hvor lidt de selv skriver, og at det er trygt — binæren afviser ugyldige noder, så Copilot ikke kan bygge noget forkert. Sig pointen højt: agenten behøver ikke være en del af værktøjet — binæren er autoriteten, og det snævre toolset (kun product-grafen i mappen) er blast-radius-argumentet i praksis. Manuel domain new kun som fallback. Saml op: hvor lidt skulle I selv skrive?

---

## 3 · How-talk: fem måder at bruge LLM'er
**13:55–14:45 · slides 23–28**

**Slide 23 · Sektionsskift: How**
Sektionsskift ind i How-talken. Brug overgangen: vi har modelleret What — nu hvordan vi faktisk sætter en LLM til at bygge den, og hvilke måder at gøre det på der findes i dag.

**Slide 24 · De fem modes (oversigt)**
Centrum i How-talken. To akser bevæger sig sammen ned gennem listen: driveren skifter fra hensigt → specifikation, og toolsettet snævrer fra fuldt → meget specifikt. Søjlerne til højre viser autoritet = blast-radius: hvor meget kan gå galt, når ingen ser med. Mode 1–2 er fint til udforskning med et menneske i loopet. Mode 3 er den farlige: headless OG bred autoritet — det er her folk bliver brændt. Mode 4 begynder at nedbryde og afgrænse. Mode 5 — vores tilgang — kombinerer specifikation med et meget specifikt toolset, så hver worker kun kan røre det, dens opgave kræver. Pointen: vi køber ikke autonomi med tillid til modellen, vi køber den ved at gøre konsekvensen af en fejl lille. Det binder direkte til tragt-princippet senere.

**Slide 25 · Det afgørende skift — hvorfor mode 5**
Dette er argumentet for vores tilgang. De to greb er ikke det samme: specifikationen styrer HVAD der skal ske (præcision + transparens fra What-talken); det snævre toolset begrænser HVAD der overhovedet KAN ske (blast-radius). Mode 5 bruger begge. Det er sådan vi når N4/N5 fra autonomi-stigen uden at skulle 'stole blindt' på modellen — vi gør en fejl billig i stedet for usandsynlig. Forbind eksplicit tilbage til tillids-narrativet: integritet var at gøre det aftalte; her tilføjer vi, at selv et brud kun kan gøre begrænset skade. Og forbind fremad til How-spec'en: det er specifikationen, der definerer både opgaven og det toolset, en worker må bruge.

**Slide 26 · Tragt-princippet**
Tragten er de fem modes set fra siden. Bevægelsen er den samme: fra bred hensigt (What) mod en snæver, afgrænset opgave (SPMC) og til sidst eksekvering. To ting bevæger sig modsat: constraint-tætheden stiger (hvert lag tilføjer begrænsninger), mens den krævede modelkapabilitet falder (der er mindre tilbage at beslutte). Det er derfor mode 5 virker: når How har kollapset beslutningsrummet, kan eksekveringen klares af en lille, fokuseret model med et meget specifikt toolset. Den vigtige diagnostiske pointe: hvis en worker nedstrøms stadig kræver en stor, kraftfuld model, er det et signal om, at specifikationen opstrøms ikke var nedbrudt nok. Tragten er altså både en designregel og en kvalitetsmåler. Bind tilbage til funnel-panelet, der dukker op igen i SPMC-blokken — der bruger vi den til at diagnosticere fejl.

**Slide 27 · How-specifikationen**
Hver How-sektion skal kunne spores til What. 'Ingen komponents ansvar kræver et og' — hvis det gør, så del den. Ikke-funktionelt: ikke 'hurtig' men 'p95 under 200 ms ved 100 samtidige brugere.'

**Slide 28 · Afledningskontrakten**
Regel 5 blokerer ikke arbejde — den fremkalder en samtale. Nogle gange er svaret 'ren teknisk beslutning, ingen produktdimension' — gyldigt, men det skal erklæres, ikke antages.

---

## 4 · SPMC: fra spec til eksekvering
**14:45–15:15 · slides 29–31**

**Slide 29 · Sektionsskift: SPMC**
Sektionsskift ind i SPMC-blokken. Brug overgangen: vi har What og How — nu hvordan en enkelt arbejdsenhed faktisk overdrages til modellen.

**Slide 30 · SPMC — fire elementer**
SPMC: fire elementer gør en How-komponent til ét LLM-kald — Schema (formen output skal overholde), Prompt (instruktionen, afledt af adfærd + acceptkriterier), Model (kapabilitet valgt ud fra resterende kompleksitet), Kontekst (afgrænset og frosset ved eksekvering). Alle fire er påkrævede. Brug kortene som talepunkter.

**Slide 31 · Hvorfor SPMC betaler sig**
Tragt: constraint-tætheden stiger mod værdihandlingen; krævet modelkapabilitet falder. Stor model nødvendig nedstrøms = How blev ikke nedbrudt nok. Det forbinder SPMC's Model-element tilbage til spec-komplethed.

---

## 5 · Øvelse 2: byg et SPMC-bundle
**15:15–15:40 · slide 32**

**Opgave (grupper):** tag én komponent fra en How-spec for ordresystemet og nedbryd den til én eksekverbar worker.

**Trin:**
1. Vælg én komponent (fx PlaceOrder-Decideren).
2. Skriv det output-Schema, den skal overholde.
3. Udkast Prompten ud fra adfærden + acceptkriterierne.
4. Angiv den Kontekst, workeren har brug for — og intet mere.
5. Navngiv den modelkapabilitet, den resterende kompleksitet kræver.

**Leverance:** ét komplet SPMC-bundle — plus en note om, hvad I ville skære fra Kontekst for at bruge en mindre model.

**Facilitator-noter:**
Øvelse 2. Hold den kort — deltagerne har allerede modelleret i øvelse 1; her er fokus at mærke, hvordan en komponent bliver til ét konkret, afgrænset kald. Briefen står på sliden; gå rundt og skub grupper mod at holde Kontekst minimal — det er den vigtigste vane.

---

## 6 · Afrunding & adoption
**15:40–15:50 · slides 33–34**

**Slide 33 · Sådan samler det sig**
Saml hele kæden visuelt: What-model → What-spec → How-spec → SPMC → eksekvering. Tre takeaways: adskillelse først; komplethed er minimum; kapabilitet følger komplethed. Det er opsummeringen, ikke ny information.

**Slide 34 · Tak / afslutning**
Afslutningsslide. Sidste linje: Specificér What. Afled How. Lad modellen eksekvere. Åbn for spørgsmål og for, hvor man konkret starter på et rigtigt Clever-system.

---

## Vedligehold
Én kilde: intentionen for hver slide bor i slidens talenote i decket. Dette dokument genereres derfra og redigeres ikke i hånden — så undgår man drift mellem deck og spec. Slide-indekset skal altid have præcis én række pr. slide i decket; mangler en, eller peger en på et segment, der ikke findes, er noget ude af sync. Når en slide tilføjes eller ændres: skriv/ret dens talenote i decket, og kør generatoren igen.

### Mønster for fremtidige præsentationer
Behandl enhver præsentation som What/How: agenda + intentioner + talepunkter er den delelige *What* (dette dokument), decket er en *How*. Skriv intentionen ('hvorfor findes denne slide') ind i talenoten, mens du bygger sliden — så falder facilitator-spec'en næsten ud af sig selv. Hver slide skal have en reference; en slide uden er en uerklæret beslutning.
