namespace Ordering.Api.Slices.PlaceOrder;

using Microsoft.AspNetCore.Mvc;
using Ordering.Api.Profile;

/// <summary>
/// The transport entry to the <c>PlaceOrder</c> act.
/// </summary>
/// <remarks>
/// PROFILE / controller — required: true.
///   must     "declares [Slice(&lt;instance&gt;, \"controller\")]"                   — below.
///   must     "calls exactly one type declaring the handler role for the same act
///             instance"                                                            — IPlaceOrderHandler, one call.
///   must     "returns a transport result derived from the handler's Accepted or
///             Rejected"                                                            — SEE D-21; "derived" is undefined.
///   must_not "contains a conditional on domain state"                              — SEE D-22.
///   must_not "references a provider role directly"                                 — holds.
///   must_not "references a persistence type"                                       — holds.
///
/// D-21 — INVENTED, AND THIS IS THE LARGEST SINGLE HOLE THIS RUN FOUND IN THE PROFILE.
/// "Returns a transport result DERIVED FROM the handler's Accepted or Rejected" names a
/// dependency and not a function. Nothing anywhere in the four inputs states the HTTP
/// status for an accepted command, the status for a rejected one, whether a Location
/// header is owed, or what the response body is. Every line of the mapping below is
/// this session's invention:
///     Accepted            -> 201 Created, Location: /orders/{OrderId}, body = the event
///     Rejected            -> 422 Unprocessable Entity, ProblemDetails citing the invariant
///     invalid payload     -> 400 Bad Request  (DSC-0002; see D-23)
///     read position gone  -> mapped by unroled middleware (see Unroled/), not here
/// 200 / 202 for Accepted and 400 / 409 for Rejected are all equally consistent with
/// the specification. Two readers produce two incompatible APIs from one profile and
/// both conform. Q-12, Q-13.
///
/// D-22 — DECIDED. Rejections map to ONE status regardless of which invariant was
/// cited. Branching per invariant would be "a conditional on domain state", which the
/// profile forbids the controller. So the invariant travels in the body and never in
/// the status line. That is this session's reading of a must_not whose boundary — does
/// switching on Accepted vs Rejected already count? — the profile does not draw. The
/// profile requires exactly that switch, so it cannot; where the line falls past it is
/// undrawn.
///
/// D-23 — DECIDED. DSC-0002 ("every command slice validates its payload against the
/// domain model's declared types before deciding") is realised HERE and not in the
/// handler, because the handler may reject "only for invariants the fact vocabulary
/// declares" and a payload type fault is not one. But that puts a transport result on a
/// path that is NOT "derived from the handler's Accepted or Rejected", contradicting
/// the controller's own must. One of the two rules gives; this session chose which.
/// Q-09.
///
/// D-24 — INVENTED. Route, verb, the plural noun, the absence of versioning in the path,
/// and the fact that the resource is /orders rather than /place-order. The profile
/// fixes the stack and names the role; it fixes no URL.
/// </remarks>
[ApiController]
[Route("orders")]
[Slice("PlaceOrder", SliceRole.Controller)]
public sealed class PlaceOrderController : ControllerBase
{
    private readonly IPlaceOrderHandler _handler;

    public PlaceOrderController(IPlaceOrderHandler handler) => _handler = handler;

    [HttpPost]
    [ProducesResponseType(StatusCodes.Status201Created)]
    [ProducesResponseType(StatusCodes.Status400BadRequest)]
    [ProducesResponseType(StatusCodes.Status422UnprocessableEntity)]
    public IActionResult Post([FromBody] PlaceOrderCommand command)
    {
        // DSC-0002. [ApiController] already short-circuits an invalid ModelState to 400
        // before this method is entered; the explicit guard states the determination at
        // its site rather than leaving it to framework behaviour. D-23.
        if (!ModelState.IsValid)
        {
            return ValidationProblem(ModelState);
        }

        var outcome = _handler.Handle(command);

        return outcome switch
        {
            PlaceOrderOutcome.Accepted accepted => Created(
                $"/orders/{accepted.Event.OrderId}",
                accepted.Event),

            PlaceOrderOutcome.Rejected rejected => UnprocessableEntity(new ProblemDetails
            {
                Status = StatusCodes.Status422UnprocessableEntity,
                Title = "The command was rejected.",
                Detail = rejected.Reason,
                Extensions = { ["invariant"] = rejected.Invariant },
            }),

            // D-25 — INVENTED. PlaceOrderOutcome is a closed hierarchy, so this arm is
            // unreachable; C# cannot prove it. The profile's "Accepted or Rejected" says
            // the pair is exhaustive and says nothing about how to say so in the stack.
            _ => throw new InvalidOperationException(
                $"Unhandled outcome '{outcome.GetType().Name}'."),
        };
    }
}
