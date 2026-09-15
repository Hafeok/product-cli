# Gate A — the candidate set over A, held for ratification

**Status: `[PROPOSED]`, 2026-09-15.** Nothing here is ratified by the session. Every figure is
**transport-derived** (CG-R-105). No candidate carries an act name, what it settles or who
answers (CG-R-106); the actor and scale slots are unfilled on every one (CG-R-108); the
supported-throughput field is carried empty on every one (CG-R-109). The full print is
`measurement/gateA-A-candidates.txt`; every field, every path type and every overlap pair is in
`measurement/gateA-A-candidates.json`.

## 1. Instrument record

`measurement/gateA-instrument.txt`: commit `ca58cd9`, working tree 0 uncommitted paths, binary
`target/release/product` sha256 `86225ad84edaf43f`, inventory A v6 sha256 `3197d7e07c5a0b46` —
**the run-8 artefact; the reader was not rebuilt or re-run** (CG-R-103). Before deriving
anything, the run-8 primary reach output was regenerated with this binary and diffed:
**byte-identical**. The existing measurement did not move.

The derivation ran twice. The first run (07:46Z) used a binary one edit behind the commit —
the cross-type split of the overlap pairs was added after the release build had started — and
its outputs were discarded unread of their figures and the script re-run from the committed
tree. Recorded because the order was wrong once already this week (`gate1a-run8.md` §4); the
committed outputs are from the second run only.

## 2. The candidate set — 68, transport-derived

| Kind | Count | Transport name | Path |
|---|---|---|---|
| controller-action (E-1) | 25 | `Type.Method` | from `[Route]` templates, tokens substituted |
| razor-page-handler (E-2) | 19 | `PageModel.Handler` (one `(render)`) | from the code-behind's path (P-EP-2) |
| view-component (E-3) | 1 | `Basket.InvokeAsync` | none — view-side (CG-R-78) |
| fastendpoints (E-4) | 23 | `Endpoint` type | **not read** (L-EP-2); verb read |
| hosted-service (E-5) | 0 | — | A has none |

Hosts (registration sites a candidate resolves through, each host's own closure): Web 352
members, PublicApi 50, BlazorAdmin 111 (a client; no candidate lives in it), AppHost 1.

**Recall against the hand enumeration** (`ground-truth-A-entry-points.yaml`, committed before
the run): **68 of 71 found (95.8%)**, precision **68 of 68 (100%)**. The three misses are
the three the criterion predicted (§9 of the criterion): two `MapHealthChecks` endpoints and
the `MapFallbackToFile` endpoint, invisible under L-EP-1. One correction to the enumeration
was made before the run and is noted in the file: the two `UpdateRoleEndpoint` keys carried an
abbreviated namespace tail; corrected to the full tail, no entry added or removed.

### The set

Columns: id · method · path · observed authorisation (`af` = anti-forgery attribute) · identity
checks on the path · path (production types / composition edges / unscored) · facts read
(P-EP-4) · facts written (P-EP-4). Ground slots and the invited determination are omitted from
the table because they are the same on every row: *not stated · not stated · not stated* and
*— · — · — · —*.

| id | method | path | observed authorisation | id checks | path | read | written |
|---|---|---|---|---|---|---|---|
| `ManageController@Controllers.ChangePassword#GET` | GET | `/Manage/ChangePassword` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.ChangePassword#POST` | POST | `/Manage/ChangePassword` | Authorize@type · af | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.Disable2fa#POST` | POST | `/Manage/Disable2fa` | Authorize@type · af | 1 | 7 / 7 / 0 | — | — |
| `ManageController@Controllers.Disable2faWarning#GET` | GET | `/Manage/Disable2faWarning` | Authorize@type | 1 | 7 / 7 / 0 | — | — |
| `ManageController@Controllers.EnableAuthenticator#GET` | GET | `/Manage/EnableAuthenticator` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.EnableAuthenticator#POST` | POST | `/Manage/EnableAuthenticator` | Authorize@type · af | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.ExternalLogins#GET` | GET | `/Manage/ExternalLogins` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.GenerateRecoveryCodes#POST` | POST | `/Manage/GenerateRecoveryCodes` | Authorize@type · af | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.GenerateRecoveryCodesWarning#GET` | GET | `/Manage/GenerateRecoveryCodesWarning` | Authorize@type | 1 | 7 / 7 / 0 | — | — |
| `ManageController@Controllers.LinkLogin#POST` | POST | `/Manage/LinkLogin` | Authorize@type · af | 1 | 6 / 7 / 0 | — | — |
| `ManageController@Controllers.LinkLoginCallback#GET` | GET | `/Manage/LinkLoginCallback` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.MyAccount#GET` | GET | `/Manage/MyAccount` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.MyAccount#POST` | POST | `/Manage/MyAccount` | Authorize@type · af | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.RemoveLogin#POST` | POST | `/Manage/RemoveLogin` | Authorize@type · af | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.ResetAuthenticator#POST` | POST | `/Manage/ResetAuthenticator` | Authorize@type · af | 1 | 7 / 7 / 0 | — | — |
| `ManageController@Controllers.ResetAuthenticatorWarning#GET` | GET | `/Manage/ResetAuthenticatorWarning` | Authorize@type | 0 | 6 / 7 / 0 | — | — |
| `ManageController@Controllers.SendVerificationEmail#POST` | POST | `/Manage/SendVerificationEmail` | Authorize@type · af | 1 | 10 / 7 / 0 | — | — |
| `ManageController@Controllers.SetPassword#GET` | GET | `/Manage/SetPassword` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.SetPassword#POST` | POST | `/Manage/SetPassword` | Authorize@type · af | 1 | 8 / 7 / 0 | — | — |
| `ManageController@Controllers.ShowRecoveryCodes#GET` | GET | `/Manage/ShowRecoveryCodes` | Authorize@type | 0 | 7 / 7 / 0 | — | — |
| `ManageController@Controllers.TwoFactorAuthentication#GET` | GET | `/Manage/TwoFactorAuthentication` | Authorize@type | 1 | 8 / 7 / 0 | — | — |
| `OrderController@Controllers.Detail#GET` | GET | `/Order/Detail/{orderId}` | Authorize@type | 2 | 24 / 14 / 4 | Order | — |
| `OrderController@Controllers.MyOrders#GET` | GET | `/Order/MyOrders` | Authorize@type | 2 | 24 / 14 / 4 | Order | — |
| `UserController@Controllers.GetCurrentUser#GET` | GET | `/User` | Authorize@member; AllowAnonymous@member | 3 | 8 / 5 / 0 | — | — |
| `UserController@Controllers.Logout#POST` | POST | `/User/Logout` | Authorize@member; AllowAnonymous@member | 1 | 7 / 5 / 0 | — | — |
| `CheckoutModel@Basket.OnGet#GET` | GET | `/Basket/Checkout` | Authorize@type | 3 | 49 / 29 / 4 | Basket, CatalogItem, Order | Basket |
| `CheckoutModel@Basket.OnPost#POST` | POST | `/Basket/Checkout` | Authorize@type | 3 | 49 / 29 / 4 | Basket, CatalogItem, Order | Basket |
| `ConfirmEmailModel@Account.OnGetAsync#GET` | GET | `/Identity/Account/ConfirmEmail` | AllowAnonymous@type | 0 | 2 / 1 / 0 | — | — |
| `EditCatalogItemModel@Admin.OnGet#GET` | GET | `/Admin/EditCatalogItem` | Authorize@type (Roles=Administrators) | 0 | 9 / 3 / 0 | CatalogItem | CatalogItem |
| `EditCatalogItemModel@Admin.OnPostAsync#POST` | POST | `/Admin/EditCatalogItem` | Authorize@type (Roles=Administrators) | 0 | 9 / 3 / 0 | CatalogItem | CatalogItem |
| `ErrorModel@Pages.OnGet#GET` | GET | `/Error` | *(none found)* | 0 | 1 / 0 / 0 | — | — |
| `IndexModel@Admin.(render)#GET` | GET | `/Admin/Index` | Authorize@type (Roles=Administrators) | 0 | 1 / 0 / 0 | — | — |
| `IndexModel@Basket.OnGet#GET` | GET | `/Basket/Index` | *(none found)* | 3 | 24 / 12 / 0 | Basket, CatalogItem | Basket |
| `IndexModel@Basket.OnPost#POST` | POST | `/Basket/Index` | *(none found)* | 3 | 25 / 12 / 0 | Basket, CatalogItem | Basket |
| `IndexModel@Basket.OnPostUpdate#POST` | POST | `/Basket/Index?handler=Update` | *(none found)* | 3 | 24 / 12 / 0 | Basket, CatalogItem | Basket |
| `IndexModel@Pages.OnGet#GET` | GET | `/Index` | *(none found)* | 0 | 21 / 8 / 0 | CatalogBrand, CatalogItem, CatalogType | — |
| `LoginModel@Account.OnGetAsync#GET` | GET | `/Identity/Account/Login` | AllowAnonymous@type | 0 | 13 / 7 / 0 | Basket | Basket |
| `LoginModel@Account.OnPostAsync#POST` | POST | `/Identity/Account/Login` | AllowAnonymous@type | 0 | 15 / 7 / 0 | Basket | Basket |
| `LogoutModel@Account.OnGet#GET` | GET | `/Identity/Account/Logout` | *(none found)* | 0 | 2 / 3 / 0 | — | — |
| `LogoutModel@Account.OnPost#POST` | POST | `/Identity/Account/Logout` | *(none found)* | 1 | 3 / 3 / 0 | — | — |
| `PrivacyModel@Pages.OnGet#GET` | GET | `/Privacy` | *(none found)* | 0 | 1 / 0 / 0 | — | — |
| `RegisterModel@Account.OnGet#GET` | GET | `/Identity/Account/Register` | AllowAnonymous@type | 0 | 2 / 4 / 0 | — | — |
| `RegisterModel@Account.OnPostAsync#POST` | POST | `/Identity/Account/Register` | AllowAnonymous@type | 0 | 3 / 4 / 0 | — | — |
| `SuccessModel@Basket.OnGet#GET` | GET | `/Basket/Success` | Authorize@type | 0 | 1 / 0 / 0 | — | — |
| `Basket@BasketComponent.InvokeAsync#view-component` | view-component | `(invoked from a Razor view the reader does not read, CG-R-78)` | *(none found)* | 3 | 22 / 8 / 0 | Basket, CatalogItem | — |
| `AuthenticateEndpoint@AuthEndpoints.(type)#POST` | POST | `not read (L-EP-2)` | AllowAnonymous(..) in AuthenticateEndpoint.#ctor, AuthenticateEndpoint.Configure | 0 | 9 / 3 / 0 | — | — |
| `CatalogBrandListEndpoint@CatalogBrandEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AllowAnonymous(..) in CatalogBrandListEndpoint.#ctor, CatalogBrandListEndpoint.Configure | 0 | 7 / 3 / 0 | CatalogBrand | — |
| `CatalogItemGetByIdEndpoint@CatalogItemEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AllowAnonymous(..) in CatalogItemGetByIdEndpoint.#ctor, CatalogItemGetByIdEndpoint.Configure | 0 | 13 / 4 / 0 | CatalogItem | — |
| `CatalogItemListPagedEndpoint@CatalogItemEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AllowAnonymous(..) in CatalogItemListPagedEndpoint.#ctor, CatalogItemListPagedEndpoint.Configure | 0 | 14 / 5 / 0 | CatalogItem | — |
| `CatalogTypeListEndpoint@CatalogTypeEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AllowAnonymous(..) in CatalogTypeListEndpoint.#ctor, CatalogTypeListEndpoint.Configure | 0 | 7 / 3 / 0 | CatalogType | — |
| `CreateCatalogItemEndpoint@CatalogItemEndpoints.(type)#POST` | POST | `not read (L-EP-2)` | AuthSchemes(..) in CreateCatalogItemEndpoint.#ctor, CreateCatalogItemEndpoint.Configure; Roles(..) in CreateCatalogItemEndpoint.#ctor, CreateCatalogItemEndpoint.Configure | 0 | 18 / 4 / 0 | CatalogItem | CatalogItem |
| `CreateRoleEndpoint@RoleManagementEndpoints.(type)#POST` | POST | `not read (L-EP-2)` | AuthSchemes(..) in CreateRoleEndpoint.#ctor, CreateRoleEndpoint.Configure; Roles(..) in CreateRoleEndpoint.#ctor, CreateRoleEndpoint.Configure | 0 | 8 / 1 / 0 | — | — |
| `CreateUserEndpoint@UserManagementEndpoints.(type)#POST` | POST | `not read (L-EP-2)` | AuthSchemes(..) in CreateUserEndpoint.#ctor, CreateUserEndpoint.Configure; Roles(..) in CreateUserEndpoint.#ctor, CreateUserEndpoint.Configure | 0 | 12 / 1 / 0 | — | — |
| `DeleteCatalogItemEndpoint@CatalogItemEndpoints.(type)#DELETE` | DELETE | `not read (L-EP-2)` | AuthSchemes(..) in DeleteCatalogItemEndpoint.#ctor, DeleteCatalogItemEndpoint.Configure; Roles(..) in DeleteCatalogItemEndpoint.#ctor, DeleteCatalogItemEndpoint.Configure | 0 | 8 / 2 / 0 | CatalogItem | CatalogItem |
| `DeleteRoleEndpoint@RoleManagementEndpoints.(type)#DELETE` | DELETE | `not read (L-EP-2)` | AuthSchemes(..) in DeleteRoleEndpoint.#ctor, DeleteRoleEndpoint.Configure; Roles(..) in DeleteRoleEndpoint.#ctor, DeleteRoleEndpoint.Configure | 0 | 6 / 2 / 0 | — | — |
| `DeleteUserEndpoint@UserManagementEndpoints.(type)#DELETE` | DELETE | `not read (L-EP-2)` | AuthSchemes(..) in DeleteUserEndpoint.#ctor, DeleteUserEndpoint.Configure; Roles(..) in DeleteUserEndpoint.#ctor, DeleteUserEndpoint.Configure | 0 | 5 / 1 / 0 | — | — |
| `DeleteUserFromRoleEndpoint@RoleMembershipEndpoints.(type)#DELETE` | DELETE | `not read (L-EP-2)` | AuthSchemes(..) in DeleteUserFromRoleEndpoint.#ctor, DeleteUserFromRoleEndpoint.Configure; Roles(..) in DeleteUserFromRoleEndpoint.#ctor, DeleteUserFromRoleEndpoint.Configure | 0 | 5 / 2 / 0 | — | — |
| `RoleGetByIdEndpoint@RoleManagementEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in RoleGetByIdEndpoint.#ctor, RoleGetByIdEndpoint.Configure; Roles(..) in RoleGetByIdEndpoint.#ctor, RoleGetByIdEndpoint.Configure | 0 | 6 / 1 / 0 | — | — |
| `RoleListEndpoint@RoleManagementEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in RoleListEndpoint.#ctor, RoleListEndpoint.Configure; Roles(..) in RoleListEndpoint.#ctor, RoleListEndpoint.Configure | 0 | 4 / 1 / 0 | — | — |
| `RoleMembershipGetByNameEndpoint@RoleMembershipEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in RoleMembershipGetByNameEndpoint.#ctor, RoleMembershipGetByNameEndpoint.Configure; Roles(..) in RoleMembershipGetByNameEndpoint.#ctor, RoleMembershipGetByNameEndpoint.Configure | 0 | 7 / 1 / 0 | — | — |
| `SaveRolesForUserEndpoint@UserManagementEndpoints.(type)#PUT` | PUT | `not read (L-EP-2)` | AuthSchemes(..) in SaveRolesForUserEndpoint.#ctor, SaveRolesForUserEndpoint.Configure; Roles(..) in SaveRolesForUserEndpoint.#ctor, SaveRolesForUserEndpoint.Configure | 0 | 5 / 1 / 0 | — | — |
| `UpdateCatalogItemEndpoint@CatalogItemEndpoints.(type)#PUT` | PUT | `not read (L-EP-2)` | AuthSchemes(..) in UpdateCatalogItemEndpoint.#ctor, UpdateCatalogItemEndpoint.Configure; Roles(..) in UpdateCatalogItemEndpoint.#ctor, UpdateCatalogItemEndpoint.Configure | 0 | 16 / 4 / 0 | CatalogItem | CatalogItem |
| `UpdateRoleEndpoint@RoleManagementEndpoints.(type)#PUT` | PUT | `not read (L-EP-2)` | AuthSchemes(..) in UpdateRoleEndpoint.#ctor, UpdateRoleEndpoint.Configure; Roles(..) in UpdateRoleEndpoint.#ctor, UpdateRoleEndpoint.Configure | 0 | 6 / 1 / 0 | — | — |
| `UpdateRoleEndpoint@UserManagementEndpoints.(type)#PUT` | PUT | `not read (L-EP-2)` | AuthSchemes(..) in UpdateRoleEndpoint.#ctor, UpdateRoleEndpoint.Configure; Roles(..) in UpdateRoleEndpoint.#ctor, UpdateRoleEndpoint.Configure | 0 | 9 / 1 / 0 | — | — |
| `UserGetByIdEndpoint@UserManagementEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in UserGetByIdEndpoint.#ctor, UserGetByIdEndpoint.Configure; Roles(..) in UserGetByIdEndpoint.#ctor, UserGetByIdEndpoint.Configure | 0 | 9 / 1 / 0 | — | — |
| `UserGetByUserNameEndpoint@UserManagementEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in UserGetByUserNameEndpoint.#ctor, UserGetByUserNameEndpoint.Configure; Roles(..) in UserGetByUserNameEndpoint.#ctor, UserGetByUserNameEndpoint.Configure | 0 | 9 / 1 / 0 | — | — |
| `UserGetRolesByIdEndpoint@UserManagementEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in UserGetRolesByIdEndpoint.#ctor, UserGetRolesByIdEndpoint.Configure; Roles(..) in UserGetRolesByIdEndpoint.#ctor, UserGetRolesByIdEndpoint.Configure | 0 | 8 / 1 / 0 | — | — |
| `UserListEndpoint@UserManagementEndpoints.(type)#GET` | GET | `not read (L-EP-2)` | AuthSchemes(..) in UserListEndpoint.#ctor, UserListEndpoint.Configure; Roles(..) in UserListEndpoint.#ctor, UserListEndpoint.Configure | 0 | 7 / 1 / 0 | — | — |

For the 23 FastEndpoints rows the authorisation column is the configuring calls by member id;
the `Roles(..)`/`AuthSchemes(..)` **arguments are not read** (L-EP-2) — the constant fields the
configuring member references (`Constants.Roles.ADMINISTRATORS`, `Roles.PRODUCT_MANAGERS`,
`JwtBearerDefaults.AuthenticationScheme`) are printed in the full output as *argument
symbols*, with the binding of symbol to call unread.

**Nine candidates have no authorisation attribute or call found**: eight page handlers
(`Error`, `Index`, `Privacy`, `Basket/Index` ×3, `Logout` ×2) and the ViewComponent. *Not
found* is not *anonymous*: the `AuthorizePage("/Basket/Checkout")` convention in `Program.cs`
and any fallback policy live in call arguments the reader does not emit (L-EP-1). On A the one
convention read by hand names `/Basket/Checkout`, which carries `[Authorize]` anyway.

## 3. The paths

Every candidate walked from its handler(s) plus constructors (or its type) through its host's
sites, O-17 off. Composition edges are counted **per candidate** — paths overlap, so the sum
is not the run-8 denominator and is not comparable to it.

| Kind | Edges | resolved | unresolved | registration-not-read | partial | boundary | excluded | unscored | path types min / median / max |
|---|---|---|---|---|---|---|---|---|---|
| controller-action | 185 | 83 | 2 | 92 | 4 | 2 | 2 | 8 | 6 / 8 / 24 |
| razor-page-handler | 137 | 78 | 21 | 30 | 4 | 2 | 2 | 8 | 1 / 9 / 49 |
| view-component | 8 | 4 | 3 | 1 | 0 | 0 | 0 | 0 | 22 |
| fastendpoints | 45 | 12 | 11 | 22 | 0 | 0 | 0 | 0 | 4 / 8 / 18 |
| **all** | **375** | 177 | 37 | 145 | 8 | 4 | 4 | **16 (4.3%)** | |

The **error bound** on the per-candidate paths is the unscored fraction, 16 of 375 = 4.3%
(CG-R-89) — every unscored edge could add or remove types from a path. Registration-not-read
is 39% of all edges: the identity stack (`UserManager`, `SignInManager`, …) behind
`ManageController` and the Identity pages, table-known and, under the rules in force for
run 8, still counted as unread; from run 9 (CG-R-96) these become boundary and leave the
denominator — stated so that the Gate B figures, which will run under CG-R-96, are not read
against this table.

The longest path is `Basket/Checkout` (49 production types, 29 edges): basket, order,
catalog, identity. The shortest are `Error`, `Privacy`, `Basket/Success` and the `Admin/Index`
render: one production type, no composition edge.

## 4. Facts along the path — proxy P-EP-4, incidence unmeasured

20 of 68 candidates touch a repository-mediated entity. By entity, the number of candidates:

| | Basket | CatalogItem | Order | CatalogBrand | CatalogType |
|---|---|---|---|---|---|
| read | 8 | 14 | 4 | 2 | 2 |
| written (one write-capable repository on the writing type) | 7 | 5 | 0 | 0 | 0 |
| possibly written (the writing type holds several) | 6 | 6 | 2 | 0 | 0 |
| touched via `DbSet` (read/write not distinguished) | 6 | 0 | 0 | 0 | 0 |

`Order` is never *written* under the proxy and twice *possibly written*: the type that adds an
order holds three write-capable repositories and the call site's type argument is not in the
inventory. That is the proxy's stated divergence showing, not a fact about A.

The 48 candidates with no fact under the proxy are the identity and account surface (the
`ManageController` actions, the Identity pages, the role/user endpoints): their state lives
behind `UserManager<T>` and `RoleManager<T>`, which the proxy does not read. **The proxy is
blind to identity state**, which is most of A's surface by count. Stated; not widened.

## 5. Overlaps — the merge signal (CG-R-106), reported not decided

**Identical path sets: 11 groups**, ten of them handlers on one type (GET/POST of the same
page or action) — structural, since constructor injection is per type — and one across types:
the three Account handlers `ConfirmEmail.OnGetAsync`, `Logout.OnGet`, `Register.OnGet`, whose
whole path is one type, `ApplicationUser`.

**Pairs at Jaccard ≥ 0.5: 250, of which 30 across types.** The within-type 220 are the same
structural fact. The 30 across types are where the signal is; the ones a reader should look at:

| Pair | Shared | Jaccard | What is shared (from the path lists) |
|---|---|---|---|
| `Basket/Index` (3 handlers) ~ `Basket` ViewComponent | 19 | 0.73–0.76 | the basket read path: `IBasketViewModelService`/`BasketViewModelService`, `IBasketQueryService`/`BasketQueryService`, `Basket`, `BasketItem`, `CatalogItem`, `CatalogContext`, `EfRepository`, `UriComposer` |
| `UserController.Logout` / `GetCurrentUser` (Web, cookie) ~ `AuthenticateEndpoint` (PublicApi, JWT) | 5 | 0.50–0.56 | `ApplicationUser`, `ITokenClaimsService`/`IdentityTokenClaimService`, `AuthorizationConstants`, `UserNotFoundException` — **two hosts, two transports, one path** |
| `CreateCatalogItem` ~ `UpdateCatalogItem` ~ `CatalogItemGetById` ~ `CatalogItemListPaged` | 9–12 | 0.53–0.60 | the catalog item path: `CatalogItem`, `CatalogItemDto`, `IRepository`/`EfRepository`, `CatalogContext`, `IUriComposer`/`UriComposer`, `CatalogSettings` |
| `UserGetById` ~ `UserGetByUserName` ~ `UserList` ~ `UserGetRolesById` ~ `UpdateRole@UserManagement` | 5–7 | 0.50–0.78 | `ApplicationUser`, `ApplicationUserExtensions`, `UserDto`, `GetUserResponse` (the `UserManager` behind them is external: registration-not-read) |
| `DeleteUser` ~ `DeleteUserFromRole` ~ `SaveRolesForUser` ~ `DeleteRole` | 3 | 0.50–0.60 | `ApplicationUser` and the two role-constant types only — a thin overlap |

Whether any of these is one act reached by several transports, or several acts sharing
infrastructure, is exactly what the session does not decide. The measure says only that the
transport boundaries and the path boundaries do not coincide there.

**Types reached by the most candidates** are infrastructure, not acts: `ApplicationUser` (45
of 68), `IAppLogger`/`LoggerAdapter` (28), `IEmailSender`/`LoggerEmailSender` (21),
`CatalogContext`/`EfRepository` (20), `IRepository` (18). A shared logger is shared, not a
merge; the list is printed so the reader can discount it, per CG-R-52's known divergence.

## 6. Proxies (CG-R-77) and limits (CG-R-103), with incidence on A

| Id | Incidence on A |
|---|---|
| P-EP-1 handler name rule | 18 public `PageModel` methods match, 0 do not |
| P-EP-2 page route from the file path | 1 of 12 pages carries a `@page` template (`Basket/Index`, `"{handler?}"`), by hand; the instrument cannot see it — the `OnPostUpdate` path is printed as `?handler=Update` where the page actually routes `/Basket/Index/Update` |
| P-EP-3 `Invoke` rule | 1 ViewComponent with `InvokeAsync`, 0 without |
| P-EP-4 facts | unmeasured — first use |
| L-EP-1 builder calls absent | 0 references to `M:Microsoft.AspNetCore.Builder.*` in 6,992; 3 endpoints invisible |
| L-EP-2 call arguments not emitted | 23 of 23 FastEndpoints routes unread |
| L-EP-3 Razor not read | 71 razor files, 45 `@inject` in production projects (BlazorAdmin 23/44, Web 48/1) |

## 7. Findings — reported, not repaired

- **F-EP-1 — primary constructors carry the type body's references.** For a type declared
  with a C# primary constructor, the reader emits the `#ctor` member with every reference the
  other members make (`AuthenticateEndpoint.#ctor`: 25 references = the union of `Configure`
  and `ExecuteAsync`). Incidence on A: 23 of 23 FastEndpoints constructors carry `Configure`'s
  calls; 34 constructors in the solution carry every call edge of every other member of their
  type; 21 source files in Web and PublicApi declare a primary constructor. Effect on this
  gate: none on the set (the duplicate authorisation labels are collapsed in display); the
  walk reaches the same types either way. Effect on the run-8 figures: none on composition
  edges (a constructor's *parameters* are unaffected); any per-member reference count in the
  ground-truth measure would double-count. A reader defect; under CG-R-103 reported here and
  not repaired.
- **F-EP-2 — the reader does not see endpoint mapping.** L-EP-1 is a reader gap first found
  by this gate: no call on `WebApplication`/`IEndpointRouteBuilder` extension members reaches
  the reference graph. Three of A's 71 integration points are invisible, and so is every
  routing and authorisation convention configured there. Not repaired.
- **The first run's binary was one edit behind** (§1). Discarded and re-run; recorded.
- **The ground-truth key format** needed one correction before the run (§2); recorded in the
  file and here.

## 8. Prohibitions, checked

| Prohibition | How it was kept |
|---|---|
| no act names | every id is `kind:Type@Namespace.Member#METHOD`, the transport's own names; no field holds an act name, a settlement or an answerer |
| no slice declarations | the tool writes nothing; the set is a print |
| no inferred actors or scale | `expected_actor_kinds`, `population_order_of_magnitude`, `rate_order_of_magnitude` are `null` on all 68 (tested: null, never an empty list) |
| no authored determination | `supported_throughput.{sustained,peak,window,behaviour_above_limit}` are `null` on all 68 |
| no library surface as entry points | ApplicationCore, Infrastructure, BlazorShared yield no candidate; only hosts do |
| reader not improved | the reader binary was not rebuilt; run-8 A output byte-identical |
| B out of scope | B's inventory was not opened |

## 9. Registers

**In force:** CG-R-105 … CG-R-109 as filed. **`[PROPOSED]`:** the criterion
(`gateA-criterion.md`), the four proxies, the three limits, the overlap measure, this set.
**`[OPEN]`:** (a) whether BlazorAdmin's 23 Razor components are A's candidates too — they are
a client's entry points, reached by the server only through the fallback file; (b) whether a
health-check endpoint is an integration point in CG-R-105's sense (an actor outside the system
initiates — a probe does); (c) rulings CG-R-99 … CG-R-104 were not received (`arrived-inputs.md`
§4).

## 10. Weakest point

**The path is type-granular beyond its first hop.** A handler's own calls are followed
member by member, but every dependency it resolves opens the *whole* implementing type, so
two handlers on one type reach the same set whatever they do — 220 of the 250 overlapping
pairs are that artefact, and the identical-set groups are almost all of it. The overlap
measure can therefore only discriminate across types, and a codebase whose acts live on
shared types would read as fully merged. The second weakness is P-EP-4: it reads which entity
a path *can* reach through a repository, not which it *does* change, and it does not read
identity state at all — 48 of 68 candidates show no fact.

## 11. Held

Gate B needs two things that are Emil's: ratification of this set (with each accepted
candidate named — naming is ratification, CG-R-106 — and each rejection given a reason), and
slice declarations against the accepted ones. The session does not proceed past this gate
without them.
