# Workshop runbook — capturing a product's What with `product`

A facilitator's guide to running a ~90-minute hands-on session where a team
captures the **What** of a real product as a conformant graph, then sees the
path onward to How and Delivery. Built on the framework graph; assumes no prior
knowledge from participants.

## Outcomes

By the end, participants will have:

- a shared, **validated** What model of one product area (contexts, entities,
  commands, events, a read model);
- felt the discipline the framework enforces (every event changes a real entity,
  every command emits one) — caught by the tool, not by review;
- a clear, self-service next step (`product guide`) they can keep using after
  the room empties.

## Before the day (facilitator prep)

1. **Install** `product` on every machine (see the README) and confirm
   `product --version` works. For the facilitated capture path, also confirm an
   agent CLI (`claude`) is installed and authenticated.
2. **Pick the product area.** One bounded context with 3–6 entities and 2–3
   interesting flows. Resist scope — depth beats breadth.
3. **Dry-run the tutorial** yourself end to end:
   [guide/getting-started-framework.md](guide/getting-started-framework.md). If
   it runs clean for you, it'll run clean in the room.
4. **Seed a demo** to show the finished shape first:
   `product init --demo` spins up the worked **bookstore** model in seconds —
   open `product status` and `product domain show Order` to set expectations.
5. **Print/share** [guide/framework-concepts.md](guide/framework-concepts.md) —
   the one-paragraph-per-term primer is the only reading participants need.

## Agenda (90 minutes)

| Time | Segment | What happens |
|---|---|---|
| 0:00–0:10 | **Frame it** | The one idea: What vs How, agreed before built, all checked. Show the demo bookstore graph. |
| 0:10–0:20 | **Setup** | Everyone runs `product init --name <area>` then `product guide`. The guide is the through-line — point at it now. |
| 0:20–0:40 | **Structure** | Author the bounded context and its entities (`product domain new context/entity`). Stress the business-language `--definition`. |
| 0:40–1:05 | **Behaviour** | The interesting flows: `domain new event` → `command` → `read-model`. This is where the "every event changes a real entity" rule bites — let it. |
| 1:05–1:15 | **Validate** | `product domain validate` until conformant. Use `product guide` to drive out the gaps. A green validate is the milestone — celebrate it. |
| 1:15–1:25 | **The path onward** | Show, don't do: `how init`, `slice new`, `deliverable new`, and where `decider`/`build` go. The journey checklist makes the rest concrete. |
| 1:25–1:30 | **Close** | Each table runs `product status` + `product guide`. Everyone leaves knowing their next command. |

## The through-line: `product guide`

The single most important habit to instill. Whenever anyone asks "what do I do
now?", the answer is `product guide`. It reads their graph and prints the exact
next command. Repeat it until it's reflex — it's what makes the framework
self-service after the workshop.

## Authoring cheat-sheet (hand this out)

```bash
# structure
product domain new context  <Ctx>  --label "<Ctx>" --purpose "..."
product domain new entity   <Ent>  --label "<Ent>" --context <Ctx> \
                                    --definition "..." --aggregate-root true
# behaviour  (author in this order — events before the commands that emit them)
product domain new event    <Evt>  --label "..." --context <Ctx> --changes <Ent>
product domain new command  <Cmd>  --label "..." --targets <Ent> --emits <Evt>
product domain new read-model <RM> --label "..." --projects <Evt>
# check & orient
product domain validate
product guide
```

## Troubleshooting (the papercuts to pre-empt)

| Symptom | Cause | Fix |
|---|---|---|
| `Rejected X — no change made: [label] …` | A required field was omitted | A context/entity needs `--label`; an entity also needs `--context` and `--definition`. Nothing was saved — re-run with the field. |
| `Rejected X … [targets]/[emits]` | The command references a node that doesn't exist yet | Author the entity and the event **first**, then the command. |
| `Rejected X … [changes]/[inContext]` | The event has no `--changes <entity>` or `--context` | Add both; the entity and context must already exist. |
| "I don't know what to do next" | — | `product guide`. Always. |
| `domain list` shows fewer nodes than expected | Some `new` calls were rejected | Re-run `product domain validate` to see what's missing; rejected nodes never persisted. |

## After the workshop

Participants keep the graph in `.product/` (commit it). The same commands run
every day; `product guide` keeps pointing at the next step as the model grows
from What into How and Delivery.
