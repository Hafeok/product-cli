using Ordering.Api.Slices.PlaceOrder;
using Ordering.Api.Unroled;

// ---------------------------------------------------------------------------
// Composition root.
//
// NO [Slice] ATTRIBUTE APPLIES HERE. The profile constrains three roles and says
// nothing about how one role reaches another, who registers them, or what lifetimes
// they take. D-32 — every line below is invented.
//
// D-33 — DECIDED, AND IT IS THE LOAD-BEARING ONE. `IPlaceOrderHandler` resolves to
// `EventAppendingPlaceOrderHandler`, not to `PlaceOrderHandler`. The controller calls
// the interface, so its source text calls "exactly one type declaring the handler role
// for the same act instance" — while at run time the first type it reaches declares no
// role at all and performs the persistence the profile forbids every role it names.
// The profile is satisfied statically and bypassed dynamically by one DI registration.
// See Unroled/README.md and Q-16.
// ---------------------------------------------------------------------------

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddHttpContextAccessor();

// Profile roles.
builder.Services.AddScoped<CartProvider>();
builder.Services.AddScoped<ActorIdentityProvider>();
builder.Services.AddScoped<PlaceOrderHandler>();

// Unroled — outside every profile rule.
builder.Services.AddScoped<IPlaceOrderHandler, EventAppendingPlaceOrderHandler>();
builder.Services.AddScoped<IClaimSource, HttpContextClaimSource>();
builder.Services.AddSingleton<ICartStore, InMemoryCartStore>();
builder.Services.AddSingleton<IOrderPlacedSink, InMemoryOrderPlacedSink>();
builder.Services.AddSingleton<IOrderIdentityMint, GuidOrderIdentityMint>();
builder.Services.AddSingleton(TimeProvider.System);

var app = builder.Build();

// D-31 — see Unroled/Adapters.cs.
app.UseMiddleware<ReadPositionUnavailableMiddleware>();

app.MapControllers();

app.Run();

// D-34 — DECIDED. Exposed so the integration tests can drive the slice through HTTP.
// Nothing in the specification requires or forbids a test, which is itself Q-21.
public partial class Program;
