//! The instrument's stated limits, each with its measured incidence (CG-R-77, CG-R-94).
//!
//! A limit is something the reader or the walk cannot see by design or by a
//! gap not yet closed. Stating it without its incidence would be the
//! "this may miss X" form CG-R-77 refused; each entry carries what was
//! measured, where, and when — or says *unmeasured*.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Limit {
    pub what: &'static str,
    pub rule: &'static str,
    pub incidence: &'static str,
}

pub const LIMITS: &[Limit] = &[
    Limit { what: "Razor views are not in the inventory; every @inject in them is an unseen composition edge", rule: "R-1 / CG-R-78 (rule 1)", incidence: "A 45 directives in 71 files, B 342 in 1,610 (run 7, 2026-09-14); printed beside every headline" },
    Limit { what: "a registration on a builder held in a variable is not recorded (only chains within one statement are)", rule: "rule 11 / R-2", incidence: "B 66 IHtmlLocalizer<T> edges left boundary behind AddViewLocalization on a variable; A 0 (run 7, 2026-09-14)" },
    Limit { what: "parameter attributes are not emitted: [FromServices] method injection is invisible", rule: "rule 4", incidence: "B 7 occurrences by source grep, A 0 (2026-09-14); unmeasured by the instrument" },
    Limit { what: "GetServices<T> and GetService<T> are one resolution of T (service-locator collection injection)", rule: "rule 10", incidence: "unmeasured (2026-09-14)" },
    Limit { what: "middleware named by UseMiddleware<T> has no fact to root on", rule: "roots", incidence: "A 1 ground-truth edge missed (run 7, 2026-09-14)" },
    Limit { what: "a lifetime-named registration whose arguments are variables is parsed in shape, opaque in content", rule: "resolver", incidence: "reported per run as the table's opaque count (CG-R-94)" },
];
