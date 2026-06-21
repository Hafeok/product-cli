# Øvelse 1 — Modellér jeres eget system

**Tid:** 45 minutter · **Form:** i grupper · **Værktøj:** Copilot CLI + `product`

> **Målet:** byg en **domænemodel** *og* en **event-model** for ét af jeres
> egne systemer — i én graf, schema-valid og klar til at bygge videre på.
> Pointen I skal mærke: hvor lidt I selv skriver, og at det er trygt — binæren
> afviser ugyldige noder, så agenten ikke kan bygge noget forkert.

---

## Setup (2 min)

I har et forberedt repo klonet på forhånd. Det indeholder en
`.github/copilot-instructions.md`, der allerede har lært Copilot
`product`-grammatikken.

```bash
cd <det-udleverede-repo>
product --version      # bekræft at binæren virker
copilot                # start Copilot CLI i mappen
```

I **skriver ikke kommandoer i hånden**. I *beskriver* systemet i almindeligt
sprog — Copilot oversætter til `product`-kommandoer og kører dem. Binæren
validerer hver node og afviser det, der bryder en regel.

---

## Trin

### 1. Vælg ét afgrænset område (5 min)
Ét system I kender, ét bounded context, 3–6 entiteter, 2–3 interessante flows.
**Dybde slår bredde.** Modellér *ikke* hele jeres platform.

### 2. Beskriv strukturen — domænemodellen (15 min)
Sig til Copilot, hvad systemet er, og lad den foreslå begreberne. Coach den:

> "Vores system håndterer X. De centrale begreber er A, B og C.
> Opret et bounded context og entiteterne — A er aggregate-root."

- **Afgør de svære ord.** Er en *User* en *Customer*? Bliv enige **én gang**,
  her i jeres ubiquitous language — ikke igen på hvert møde.
- Giv hver entitet en **forretningssproglig `--definition`**. Det er ikke pynt;
  det er aftalen.
- Tilføj de invarianter, der altid skal gælde ("en ordre har mindst én vare").

### 3. Beskriv adfærden — event-modellen (15 min)
Beskriv ét **helt flow** som en tidslinje:

> "En kunde afgiver en ordre. Det udløser hændelsen *OrderPlaced*, som ændrer
> *Order*. En *OrderSummary* viser resultatet."

- **Husk fejlstien.** En ordre kan *afvises* (udsolgt, ugyldig kunde) — det er
  også adfærd. Et flow uden fejlsti fortæller modellen, at det aldrig fejler.
- Rækkefølgen er vigtig: event før den command, der udløser den. Lad Copilot
  styre det; retter binæren den, så læs fejlen og lad Copilot prøve igen.

### 4. Valider til grønt (5 min)
Bed Copilot køre:

```bash
product domain validate     # → conformant — N node(s), 0 violations
```

Er der violations, så bed den om at rette dem (eller kør `product guide` for at
få næste skridt). Når den er grøn — **commit**.

```bash
git add .product && git commit -m "Øvelse 1: <område> What-model"
```

---

## Når noget bliver afvist

Det er meningen. Afvisningen er typen, der virker:

| Besked | Hvad den betyder | Fix |
|---|---|---|
| `Rejected … [label]` | Et påkrævet felt mangler | Tilføj `--label` (entitet kræver også `--context` + `--definition`) |
| `Rejected … [targets]/[emits]` | Command peger på en node, der ikke findes endnu | Opret entiteten og eventet **først** |
| `Rejected … [changes]/[inContext]` | Event mangler `--changes <entitet>` eller `--context` | Tilføj begge; de skal allerede findes |
| "Hvad nu?" | — | `product guide`. Altid. |

---

## Leverance (Definition of done)

- [ ] Ét bounded context med dets entiteter, `--definition` på hver.
- [ ] Ét helt flow modelleret som command → event → read-model — **inkl. fejlsti**.
- [ ] `product domain validate` siger **conformant**.
- [ ] Grafen er committet.

**Til opsamlingen, vær klar til at svare:** *Hvor lidt skulle I selv skrive?*

> Fallback hvis Copilot driller: I kan altid skrive kommandoerne i hånden —
> grammatikken står i `.github/copilot-instructions.md` og i
> `docs/guide/getting-started-framework.md`.
