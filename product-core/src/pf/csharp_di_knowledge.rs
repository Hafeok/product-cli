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
    "T:Microsoft.AspNetCore.Hosting.Server.IServer",
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
    // --- Grown by measured frequency (CG-R-95): every call run 7 reached in A (14) or B (43)
    // that neither the resolver parsed nor the table knew, ranked in the run-7 reports. Each
    // entry lists what the call is documented to register that a constructor can ask for, or is
    // empty when it registers options, handlers or nothing injectable. Collection and LINQ
    // operations on the IServiceCollection are not registrations.
    ("M:Microsoft.Extensions.DependencyInjection.OpenTelemetryServicesExtensions.AddOpenTelemetry", "AddOpenTelemetry", &["T:OpenTelemetry.Trace.TracerProvider", "T:OpenTelemetry.Metrics.MeterProvider"]),
    ("M:OpenTelemetry.OpenTelemetryBuilder.WithMetrics", "WithMetrics", &[]),
    ("M:OpenTelemetry.OpenTelemetryBuilder.WithTracing", "WithTracing", &[]),
    ("M:OpenTelemetry.OpenTelemetryBuilderOtlpExporterExtensions.UseOtlpExporter", "UseOtlpExporter", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.ServiceDiscoveryServiceCollectionExtensions.AddServiceDiscovery", "AddServiceDiscovery", &["T:Microsoft.Extensions.ServiceDiscovery.ServiceEndpointResolver"]),
    ("M:Microsoft.Extensions.DependencyInjection.OAuthExtensions.AddOAuth", "AddOAuth", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.CookieExtensions.AddCookie", "AddCookie", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.IdentityServiceCollectionExtensions.ConfigureApplicationCookie", "ConfigureApplicationCookie", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.HealthChecksBuilderDelegateExtensions.AddCheck", "AddCheck", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.DatabaseDeveloperPageExceptionFilterServiceExtensions.AddDatabaseDeveloperPageExceptionFilter", "AddDatabaseDeveloperPageExceptionFilter", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.ComponentServiceCollectionExtensions.AddServerSideBlazor", "AddServerSideBlazor", &["T:Microsoft.JSInterop.IJSRuntime", "T:Microsoft.AspNetCore.Components.NavigationManager", "T:Microsoft.AspNetCore.Components.Authorization.AuthenticationStateProvider"]),
    ("M:Microsoft.AspNetCore.Identity.IdentityBuilderUIExtensions.AddDefaultUI", "AddDefaultUI", &["T:Microsoft.AspNetCore.Identity.UI.Services.IEmailSender"]),
    ("M:Microsoft.AspNetCore.Identity.IdentityBuilderExtensions.AddDefaultTokenProviders", "AddDefaultTokenProviders", &[]),
    ("M:Microsoft.AspNetCore.Identity.IdentityBuilder.AddTokenProvider``1", "AddTokenProvider", &[]),
    ("M:Blazored.LocalStorage.ServiceCollectionExtensions.AddBlazoredLocalStorage", "AddBlazoredLocalStorage", &["T:Blazored.LocalStorage.ILocalStorageService", "T:Blazored.LocalStorage.ISyncLocalStorageService"]),
    ("M:Microsoft.Extensions.Options.OptionsBuilder`1.Configure``1", "OptionsBuilder.Configure", OPTIONS),
    ("M:Microsoft.Extensions.DependencyInjection.ResilienceHttpClientBuilderExtensions.AddResilienceHandler", "AddResilienceHandler", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.ResilienceHttpClientBuilderExtensions.AddStandardResilienceHandler", "AddStandardResilienceHandler", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.OpenIddictExtensions.AddOpenIddict", "AddOpenIddict", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.OpenIddictCoreExtensions.AddCore", "OpenIddict.AddCore", OPENIDDICT),
    ("M:Microsoft.Extensions.DependencyInjection.OpenIddictServerExtensions.AddServer", "OpenIddict.AddServer", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.OpenIddictValidationExtensions.AddValidation", "OpenIddict.AddValidation", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.ProblemDetailsServiceCollectionExtensions.AddProblemDetails", "AddProblemDetails", &["T:Microsoft.AspNetCore.Http.IProblemDetailsService"]),
    ("M:Microsoft.Extensions.DependencyInjection.MetricsServiceExtensions.AddMetrics", "AddMetrics", &["T:System.Diagnostics.Metrics.IMeterFactory"]),
    ("M:Microsoft.Extensions.DependencyInjection.SwaggerGenServiceCollectionExtensions.AddSwaggerGen", "AddSwaggerGen", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.StackExchangeRedisDependencyInjectionExtensions.AddStackExchangeRedis", "AddStackExchangeRedis", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.OpenApiServiceCollectionExtensions.AddOpenApi", "AddOpenApi", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.MvcExtensions.AddMiniProfiler", "AddMiniProfiler", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.JsonProtocolDependencyInjectionExtensions.AddJsonProtocol``1", "AddJsonProtocol", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.HttpClientBuilderExtensions.AddHttpMessageHandler``1", "AddHttpMessageHandler", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.EndpointMetadataApiExplorerServiceCollectionExtensions.AddEndpointsApiExplorer", "AddEndpointsApiExplorer", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.EncoderServiceCollectionExtensions.AddWebEncoders", "AddWebEncoders", ENCODERS),
    ("M:Microsoft.Extensions.DependencyInjection.AzureSignalRDependencyInjectionExtensions.AddAzureSignalR", "AddAzureSignalR", &[]),
    ("M:Microsoft.Extensions.Azure.AzureClientServiceCollectionExtensions.AddAzureClientsCore", "AddAzureClientsCore", &[]),
    ("M:Microsoft.AspNetCore.DataProtection.DataProtectionBuilderExtensions.SetApplicationName", "SetApplicationName", &[]),
    ("M:Microsoft.AspNetCore.DataProtection.DataProtectionBuilderExtensions.PersistKeysToFileSystem", "PersistKeysToFileSystem", &[]),
    ("M:Microsoft.AspNetCore.DataProtection.DataProtectionBuilderExtensions.AddKeyManagementOptions", "AddKeyManagementOptions", &[]),
    ("M:Microsoft.AspNetCore.DataProtection.AzureStorageBlobDataProtectionBuilderExtensions.PersistKeysToAzureBlobStorage", "PersistKeysToAzureBlobStorage", &[]),
    ("M:Microsoft.AspNetCore.Builder.ResponseCompressionServicesExtensions.AddResponseCompression", "AddResponseCompression", &["T:Microsoft.AspNetCore.ResponseCompression.IResponseCompressionProvider"]),
    ("M:Microsoft.AspNetCore.Builder.RateLimiterServiceCollectionExtensions.AddRateLimiter", "AddRateLimiter", &[]),
    ("M:Microsoft.AspNetCore.Builder.HstsServicesExtensions.AddHsts", "AddHsts", &[]),
    ("M:GraphQL.MicrosoftDIGraphQLBuilderExtensions.AddGraphQL", "AddGraphQL", &["T:GraphQL.IGraphQLSerializer", "T:GraphQL.IGraphQLTextSerializer", "T:GraphQL.IDocumentExecuter", "T:GraphQL.Types.ISchema"]),
    ("M:Microsoft.Extensions.DependencyInjection.ServiceCollectionExtensions.AddAWSService``1", "AddAWSService", &["T:Amazon.S3.IAmazonS3"]),
    // Collection and LINQ operations on the IServiceCollection: not registrations.
    ("M:System.Collections.Generic.ICollection`1.Add", "ICollection.Add", &[]),
    ("M:System.Collections.Generic.ICollection`1.Remove", "ICollection.Remove", &[]),
    ("M:System.Collections.Generic.IList`1.RemoveAt", "IList.RemoveAt", &[]),
    ("M:System.Linq.Enumerable.Where``1", "Enumerable.Where", &[]),
    ("M:System.Linq.Enumerable.Select``2", "Enumerable.Select", &[]),
    ("M:System.Linq.Enumerable.ToArray``1", "Enumerable.ToArray", &[]),
    ("M:System.Linq.Enumerable.Except``1", "Enumerable.Except", &[]),
    ("M:Microsoft.Extensions.DependencyInjection.Extensions.ServiceCollectionDescriptorExtensions.RemoveAll``1", "RemoveAll", &[]),
];

const ENCODERS: &[&str] = &["T:System.Text.Encodings.Web.HtmlEncoder", "T:System.Text.Encodings.Web.JavaScriptEncoder", "T:System.Text.Encodings.Web.UrlEncoder"];
const OPENIDDICT: &[&str] = &[
    "T:OpenIddict.Abstractions.IOpenIddictApplicationManager", "T:OpenIddict.Abstractions.IOpenIddictAuthorizationManager",
    "T:OpenIddict.Abstractions.IOpenIddictScopeManager", "T:OpenIddict.Abstractions.IOpenIddictTokenManager",
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
    // AddMvcCore calls AddWebEncoders.
    "T:System.Text.Encodings.Web.HtmlEncoder", "T:System.Text.Encodings.Web.JavaScriptEncoder", "T:System.Text.Encodings.Web.UrlEncoder",
];
