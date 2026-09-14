//! Registration knowledge — what a framework registration call the resolver does not parse registers (CG-R-75).
//!
//! `UserManager<T>` is registered by `AddIdentity<,>()`, a call whose body
//! is a framework's, not the solution's. Calling such a type *boundary* would
//! relabel a reader gap as a legitimate category. This table names, per
//! registration method (matched by resolved symbol id, never by name), the
//! service types it is documented to register; an edge to one of them at a
//! reached call is reported `registration-not-read` with the call. A call
//! absent from this table leaves its types in `boundary` — which is why every
//! report prints the reached external calls the table does not know.

/// What the host builder (`WebApplication.CreateBuilder`, `Host.CreateDefaultBuilder`)
/// registers with no call in the solution's source (CG-R-87): in force once a
/// production entry point is reached, reported as *registration-not-read (host builder)*.
pub const HOST_PROVIDED: &[&str] = &[
    "T:Microsoft.Extensions.Logging.ILogger`1", "T:Microsoft.Extensions.Logging.ILoggerFactory",
    "T:Microsoft.Extensions.Configuration.IConfiguration", "T:Microsoft.Extensions.Hosting.IHostEnvironment",
    "T:Microsoft.AspNetCore.Hosting.IWebHostEnvironment", "T:Microsoft.Extensions.Hosting.IHostApplicationLifetime",
    "T:Microsoft.Extensions.Options.IOptions`1", "T:Microsoft.Extensions.Options.IOptionsSnapshot`1",
    "T:Microsoft.Extensions.Options.IOptionsMonitor`1", "T:Microsoft.Extensions.Options.IOptionsFactory`1",
    "T:Microsoft.Extensions.DependencyInjection.IServiceProviderIsService",
];

/// (method id up to its parameter list, the call's name, the types it registers).
pub const REGISTRATION_KNOWLEDGE: &[(&str, &str, &[&str])] = &[
    ("M:Microsoft.Extensions.DependencyInjection.IdentityServiceCollectionExtensions.AddIdentity``2", "AddIdentity", IDENTITY),
    ("M:Microsoft.Extensions.DependencyInjection.IdentityServiceCollectionExtensions.AddIdentityCore``1", "AddIdentityCore", IDENTITY),
    ("M:Microsoft.Extensions.DependencyInjection.IdentityEntityFrameworkBuilderExtensions.AddEntityFrameworkStores``1", "AddEntityFrameworkStores", &["T:Microsoft.AspNetCore.Identity.IUserStore`1", "T:Microsoft.AspNetCore.Identity.IRoleStore`1"]),
    ("M:Microsoft.Extensions.DependencyInjection.MemoryCacheServiceCollectionExtensions.AddMemoryCache", "AddMemoryCache", &["T:Microsoft.Extensions.Caching.Memory.IMemoryCache"]),
    ("M:Microsoft.Extensions.DependencyInjection.HttpServiceCollectionExtensions.AddHttpContextAccessor", "AddHttpContextAccessor", &["T:Microsoft.AspNetCore.Http.IHttpContextAccessor"]),
    ("M:Microsoft.Extensions.DependencyInjection.HttpClientFactoryServiceCollectionExtensions.AddHttpClient", "AddHttpClient", HTTP_CLIENT),
    ("M:Microsoft.Extensions.DependencyInjection.HttpClientFactoryServiceCollectionExtensions.ConfigureHttpClientDefaults", "ConfigureHttpClientDefaults", HTTP_CLIENT),
    ("M:Microsoft.Extensions.DependencyInjection.LoggingServiceCollectionExtensions.AddLogging", "AddLogging", &["T:Microsoft.Extensions.Logging.ILogger`1", "T:Microsoft.Extensions.Logging.ILoggerFactory"]),
    ("M:Microsoft.Extensions.DependencyInjection.OptionsServiceCollectionExtensions.AddOptions", "AddOptions", OPTIONS),
    ("M:Microsoft.Extensions.DependencyInjection.OptionsServiceCollectionExtensions.AddOptions``1", "AddOptions", OPTIONS),
    ("M:Microsoft.Extensions.DependencyInjection.OptionsServiceCollectionExtensions.Configure``1", "Configure", OPTIONS),
    ("M:Microsoft.Extensions.DependencyInjection.OptionsServiceCollectionExtensions.PostConfigure``1", "PostConfigure", OPTIONS),
    ("M:Microsoft.Extensions.DependencyInjection.OptionsConfigurationServiceCollectionExtensions.Configure``1", "Configure", OPTIONS),
    ("M:Microsoft.Extensions.DependencyInjection.AuthorizationServiceCollectionExtensions.AddAuthorizationCore", "AddAuthorizationCore", AUTHORIZATION),
    ("M:Microsoft.Extensions.DependencyInjection.PolicyServiceCollectionExtensions.AddAuthorization", "AddAuthorization", AUTHORIZATION),
    ("M:Microsoft.Extensions.DependencyInjection.AuthenticationServiceCollectionExtensions.AddAuthentication", "AddAuthentication", &["T:Microsoft.AspNetCore.Authentication.IAuthenticationService", "T:Microsoft.AspNetCore.Authentication.IAuthenticationSchemeProvider", "T:Microsoft.AspNetCore.Authentication.IAuthenticationHandlerProvider"]),
    ("M:Microsoft.Extensions.DependencyInjection.DataProtectionServiceCollectionExtensions.AddDataProtection", "AddDataProtection", &["T:Microsoft.AspNetCore.DataProtection.IDataProtectionProvider"]),
    ("M:Microsoft.Extensions.DependencyInjection.MvcServiceCollectionExtensions.AddMvc", "AddMvc", MVC),
    ("M:Microsoft.Extensions.DependencyInjection.MvcServiceCollectionExtensions.AddControllers", "AddControllers", MVC),
    ("M:Microsoft.Extensions.DependencyInjection.MvcServiceCollectionExtensions.AddControllersWithViews", "AddControllersWithViews", MVC),
    ("M:Microsoft.Extensions.DependencyInjection.MvcServiceCollectionExtensions.AddRazorPages", "AddRazorPages", MVC),
    ("M:Microsoft.Extensions.DependencyInjection.MvcCoreServiceCollectionExtensions.AddMvcCore", "AddMvcCore", MVC),
    ("M:Microsoft.Extensions.DependencyInjection.LocalizationServiceCollectionExtensions.AddLocalization", "AddLocalization", &["T:Microsoft.Extensions.Localization.IStringLocalizer`1", "T:Microsoft.Extensions.Localization.IStringLocalizerFactory"]),
    ("M:Microsoft.Extensions.DependencyInjection.MvcLocalizationMvcBuilderExtensions.AddViewLocalization", "AddViewLocalization", &["T:Microsoft.AspNetCore.Mvc.Localization.IHtmlLocalizer`1", "T:Microsoft.AspNetCore.Mvc.Localization.IViewLocalizer", "T:Microsoft.AspNetCore.Mvc.Localization.IHtmlLocalizerFactory"]),
    ("M:Microsoft.Extensions.DependencyInjection.SignalRDependencyInjectionExtensions.AddSignalR", "AddSignalR", &["T:Microsoft.AspNetCore.SignalR.IHubContext`1"]),
    ("M:Microsoft.Extensions.DependencyInjection.AntiforgeryServiceCollectionExtensions.AddAntiforgery", "AddAntiforgery", &["T:Microsoft.AspNetCore.Antiforgery.IAntiforgery"]),
    ("M:Microsoft.Extensions.DependencyInjection.RoutingServiceCollectionExtensions.AddRouting", "AddRouting", &["T:Microsoft.AspNetCore.Routing.LinkGenerator"]),
    ("M:Microsoft.Extensions.DependencyInjection.HealthCheckServiceCollectionExtensions.AddHealthChecks", "AddHealthChecks", &["T:Microsoft.Extensions.Diagnostics.HealthChecks.HealthCheckService"]),
    ("M:Microsoft.Extensions.DependencyInjection.ServiceCollectionExtensions.AddAutoMapper", "AddAutoMapper", &["T:AutoMapper.IMapper", "T:AutoMapper.IConfigurationProvider"]),
    ("M:NimblePros.Metronome.ServiceRegistrationExtensions.AddMetronome", "AddMetronome", &["T:DbCallCountingInterceptor"]),
    // Known to register nothing: a build, not a registration.
    ("M:Microsoft.Extensions.DependencyInjection.ServiceCollectionContainerBuilderExtensions.BuildServiceProvider", "BuildServiceProvider", &[]),
];

const IDENTITY: &[&str] = &[
    "T:Microsoft.AspNetCore.Identity.UserManager`1", "T:Microsoft.AspNetCore.Identity.SignInManager`1",
    "T:Microsoft.AspNetCore.Identity.RoleManager`1", "T:Microsoft.AspNetCore.Identity.IUserClaimsPrincipalFactory`1",
    "T:Microsoft.AspNetCore.Identity.IPasswordHasher`1", "T:Microsoft.AspNetCore.Identity.ILookupNormalizer",
    "T:Microsoft.AspNetCore.Identity.IdentityErrorDescriber", "T:Microsoft.AspNetCore.Identity.IUserValidator`1",
    "T:Microsoft.AspNetCore.Identity.IPasswordValidator`1", "T:Microsoft.AspNetCore.Identity.IRoleValidator`1",
    "T:Microsoft.AspNetCore.Identity.ISecurityStampValidator", "T:Microsoft.AspNetCore.Identity.IUserConfirmation`1",
];
const HTTP_CLIENT: &[&str] = &["T:System.Net.Http.IHttpClientFactory", "T:System.Net.Http.IHttpMessageHandlerFactory"];
const OPTIONS: &[&str] = &[
    "T:Microsoft.Extensions.Options.IOptions`1", "T:Microsoft.Extensions.Options.IOptionsSnapshot`1",
    "T:Microsoft.Extensions.Options.IOptionsMonitor`1", "T:Microsoft.Extensions.Options.IOptionsFactory`1",
];
const AUTHORIZATION: &[&str] = &[
    "T:Microsoft.AspNetCore.Authorization.IAuthorizationService", "T:Microsoft.AspNetCore.Authorization.IAuthorizationPolicyProvider",
    "T:Microsoft.AspNetCore.Authorization.IAuthorizationHandlerProvider",
];
const MVC: &[&str] = &[
    "T:Microsoft.AspNetCore.Mvc.ViewFeatures.ITempDataProvider", "T:Microsoft.AspNetCore.Mvc.Infrastructure.IActionDescriptorCollectionProvider",
    "T:Microsoft.AspNetCore.Mvc.Razor.ITagHelperFactory", "T:Microsoft.AspNetCore.Mvc.Rendering.IHtmlHelper",
    "T:Microsoft.AspNetCore.Mvc.ViewFeatures.ModelExpressionProvider", "T:Microsoft.AspNetCore.Mvc.Routing.IUrlHelperFactory",
    "T:Microsoft.AspNetCore.Mvc.IViewComponentHelper", "T:Microsoft.AspNetCore.Mvc.Razor.IRazorViewEngine",
    "T:Microsoft.AspNetCore.Mvc.ModelBinding.IModelMetadataProvider", "T:Microsoft.AspNetCore.Antiforgery.IAntiforgery",
];
