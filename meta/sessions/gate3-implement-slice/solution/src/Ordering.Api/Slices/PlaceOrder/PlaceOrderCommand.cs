using System.ComponentModel.DataAnnotations;

namespace Ordering.Api.Slices.PlaceOrder;

/// <summary>
/// The command message <c>PlaceOrder</c> takes.
/// </summary>
/// <remarks>
/// D-09 — INVENTED. <c>PlaceOrder</c> is a name in the act vocabulary. No input declares
/// a command payload, its fields, or its relation to the HTTP request body. The single
/// field below is this session's minimum: the handler reads <c>Cart</c>, so it must be
/// told which cart, and <c>ActorIdentity</c> is external and arrives by a different
/// route (DSC-0003), so it is deliberately NOT a payload field.
///
/// D-10 — The payload doubles as the HTTP request body; there is no separate transport
/// DTO. Nothing settles whether the command and the request body are one type.
///
/// DSC-0002 — "Every command slice validates its payload against the domain model's
/// declared types before deciding", allocation <c>checked</c>, closure
/// <c>operational</c>, <c>runnable_by: "roslyn-analyzer:PayloadTypeConformance"</c>.
/// That analyser does not exist, so per CG-R-127 this rule is read-enforced for this
/// run. The DataAnnotations below are this session's realisation of the check. They
/// validate the payload against types THIS SESSION declared, because the domain model
/// declares none — see Facts.cs, D-04. Q-07.
///
/// D-11 — DSC-0002's acceptance <c>does_not_cover</c> lists
/// <c>cross-field-consistency</c>. There is only one field, so the exclusion is
/// vacuous here and nothing is done about it.
/// </remarks>
public sealed record PlaceOrderCommand
{
    [Required(AllowEmptyStrings = false)]
    [StringLength(64, MinimumLength = 1)]
    public string CartId { get; init; } = string.Empty;
}
