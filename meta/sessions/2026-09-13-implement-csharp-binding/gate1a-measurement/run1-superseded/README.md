# Run 1 — superseded, retained

The first measurement run, 2026-09-14, over inventories produced by the reader before it
disambiguated colliding type ids. Retained unaltered; superseded by `../run2-*`.

**Why it is not the measurement.** Roslyn documentation-comment ids are per compilation. Two
projects that each declare a top-level `Program` (every top-level-statements web project does)
share the id `T:Program`, and the reader kept the first it met: on eShopOnWeb `BlazorAdmin`'s,
on Orchard Core `tools/OpenApiClientGenerator`'s. The `member:M:Program.{Main}$…` root in these
files therefore names the wrong application, which is why `A-reach-web-main.txt` reaches four
types and `B-reach-cms-main.txt` three. Caught by reading the reached-type count against the
namespace table, then confirming which file the surviving `T:Program` came from.

The defect is the reader's, fixed by a first pass that finds ids declared by more than one
project and suffixes them (and their members) with `@<assembly>`. Every figure in this
directory is void; nothing in it is compared against either prediction.
