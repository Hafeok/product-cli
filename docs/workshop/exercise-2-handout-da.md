# Øvelse 2 — Byg et SPMC-bundle

**Tid:** 25 minutter · **Form:** i grupper

> **Målet:** tag én komponent fra en How-spec og nedbryd den til **én
> eksekverbar worker** — ét konkret, afgrænset LLM-kald. Den vigtigste vane at
> mærke: hold **Kontekst** minimal.

Et SPMC-bundle har fire påkrævede elementer:

| Element | Hvad det er |
|---|---|
| **S**chema | Den form output skal overholde (felter, typer, constraints). |
| **P**rompt | Instruktionen — afledt af adfærden + acceptkriterierne. |
| **M**odel | Kapabiliteten, valgt ud fra den *resterende* kompleksitet. |
| **K**ontekst | Det workeren har brug for — og intet mere. Frosset ved eksekvering. |

---

## Trin

### 1. Vælg én komponent (3 min)
Fx **PlaceOrder-Decideren** fra ordresystemet. Én komponent, ét ansvar — hvis
ansvaret kræver et "og", så er den for stor; del den.

### 2. Skriv output-**Schema** (6 min)
Hvilken form *skal* outputtet have? Vær præcis — det er kontrakten outputtet
tjekkes mod.

```json
// eksempel: PlaceOrder-Decideren
{
  "decision": "accept | reject",
  "events":   [{ "type": "OrderPlaced", "orderId": "string" }],
  "reason":   "string (påkrævet når decision = reject)"
}
```

### 3. Udkast **Prompt** (6 min)
Aflys fra adfærden og acceptkriterierne — ikke fra hensigt. Inkludér
invarianterne (vare på lager, gyldig kunde) og **fejlstien** eksplicit:
"afvis med en reason hvis …".

### 4. Angiv **Kontekst** (5 min)
List præcis hvad workeren behøver: de relevante entiteter, invarianter, det ene
flow. **Intet mere.** Hver ting I tilføjer, skal kunne forsvares.

### 5. Navngiv **Model** (3 min)
Hvilken kapabilitet kræver det, der er tilbage at beslutte? Når Schema + Prompt
+ Kontekst har kollapset beslutningsrummet, kan en lille, fokuseret model klare
det. **Diagnostik:** kræver workeren stadig en stor model, var specifikationen
opstrøms ikke nedbrudt nok.

---

## Leverance (Definition of done)

- [ ] Ét komplet SPMC-bundle: Schema, Prompt, Model, Kontekst — alle fire udfyldt.
- [ ] **En note:** hvad ville I skære fra Kontekst for at kunne bruge en *mindre*
      model? (Det er hele pointen med tragten.)

> Vil I fastholde det i grafen, findes der et værktøj:
> `product work-unit init` opretter et SPMC-skelet (`work-unit.yaml`), og
> `product work-unit validate` tjekker det mod What-grafen + How-kontrakten.
