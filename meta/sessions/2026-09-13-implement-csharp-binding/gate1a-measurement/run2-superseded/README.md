# Run 2 — superseded, retained

The second measurement run, 2026-09-14, over inventories from the reader after the id
collision was fixed. Retained unaltered; superseded by `../run3-*`.

**Why it is not the measurement.** Two reader/resolver defects, both found by reading these
numbers against the inventory rather than against the predictions:

1. **Calls to extension methods were dropped.** A reduced extension-method symbol
   (`services.AddCoreServices(config)`) has a documentation id without its receiver parameter,
   which matches nothing the solution declares, so every such `call` edge failed the
   in-solution filter. `Program.{Main}$@Web` had eight out-edges and not one call; the startup
   extension methods that hold the registrations were never reached, and their registrations
   reported as `module-configuration`. ASP.NET startup is written almost entirely as extension
   methods, so this alone voids every reachability and coverage figure here. Fixed: the
   unreduced method is the symbol recorded.
2. **`assembly-scanning` was attributed to every unregistered service whenever any scanning
   call existed.** eShopOnWeb's one scanning call is `AddAutoMapper(typeof(MappingProfile))` in
   `PublicApi`, which scans for mapping profiles, not services. Fixed: scanning is attributed
   only when an implementor of the service lives in a project a scanning call names.

The figures here were seen before the fixes were made. The fixes change what the instrument
records, not any threshold, and both defects are visible in the inventory independently of
any prediction (an entry point with no call edges; a scanning site naming a project no
implementor lives in). Run 3 is the measurement; if it lands inside a prediction that run 2
did not, the reader should weigh that these fixes were made after run 2 was seen.
