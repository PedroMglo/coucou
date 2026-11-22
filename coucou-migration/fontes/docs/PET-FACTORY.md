# Pet factory

Private candidate for the next Linux build; integration and native acceptance are coordinated separately from the hover/approval hotfix.

The bundled source is `windows/src-tauri/pets.factory.json`. It describes a shared base, two product molds (`llm` and `application`), optional named extensions, and stable per-pet overrides. Common changes are inherited rather than copied between Codex, Copilot, Gravity, Ollama or application pets.

Resolution order is **base → mold → extension → pet override**. Nested objects merge; arrays and scalar values replace. Each resolved blueprint is immutable. Editable configuration documents are independent clones.

## Ownership

- `base.preferences` and `base.presentation`: shared profile structure/mode and presentation. Stable identity, provider and access policy are prohibited here and in molds/extensions.
- `molds.llm`: LLM-family differences. Provider-specific models and local/native execution remain explicit.
- `molds.application`: application-family differences, including no agent session/model UI unless an extension explicitly supports it.
- `extensions.codex-account`: shared configured Codex model profiles and profile/session presentation, used by the Codex pet and service assistants.
- `pets[id].preferences`: required explicit stable name/aliases, configured provider and initial integration grants; optional per-pet local-execution policy and model overrides. These protected fields cannot be inherited.
- `pets[id].routing`: explicit existing chat/voice/task adapter metadata; never inferred from a mold or an AI-looking name.
- `pets[id].presentation`: specific controls or card labels. Spoty keeps its music/audio route and two relevant access fields.

GitHub and the other configured service assistants remain application pets even though their implementation uses a Codex account. Spoty remains application-only: no new LLM chat, model picker, task execution or session route is created by the factory.

The TypeScript resolver is `windows/src/core/pet-factory.ts`. Current consumers are provider lookup, task ownership/names, profile lookup, capabilities, the pet gallery and pet settings. Gallery avatar size, radius, gap, context action, metrics and labels come from the same resolved presentation. The existing shared chat/approval renderers, hover logic, native adapters and habitat remain in their existing modules. The overview avatar slot is reserved metadata and is not wired in this candidate, to avoid the concurrent hotfix.

The native resolver in `windows/src-tauri/src/pet_factory.rs` resolves **configuration defaults only**, caches the bundled result once and returns clones through `pet_config::defaults()`. It does not consume frontend presentation/routing as authorization. The resolved document retains the existing version1 schema and all25 current default identities, models and access values. Both resolvers and the snapshot generator reject inherited name/aliases/provider/permissions/localExecution; every pet explicitly declares its identity/provider/grants.

## Making a common change

Change the shared base for every pet, or the relevant mold for that family. For example:

```json
{
  "molds": {
    "llm": {"presentation": {"galleryCardRadius": 17}},
    "application": {"presentation": {"profiles": false, "sessions": false, "metrics": false, "contextAction": "open"}}
  }
}
```

An instance can override one field without copying its mold. An application adapter that supports model-backed work can opt into its existing explicit extension. A new integration still needs its actual native adapter, platform/catalog entry and any provider contract registration; a template does not manufacture implementation or credentials.

Model defaults are specific to each provider or the named account extension. Changing a generic mold never swaps a pet's CLI/account. Runtime saved names, aliases, model choices, grants and local-execution policies retain their existing read/save/validation paths; the factory performs no bulk rewrite or migration of user files.

`pets.default.json` remains a compatibility snapshot for existing tooling/tests, not the production default source. After deliberately reviewing a bundled default change:

```sh
node scripts/generate-pet-defaults.mjs --check
node scripts/generate-pet-defaults.mjs
```

The first command checks semantic equality; the second regenerates only that project snapshot. Neither reads or writes HOME. Tests compare the native and frontend resolved defaults with the legacy snapshot, including Ollama's explicit local-execution policy and all integration grants.

## Boundaries

Molds describe product reuse. Native original request authority, provider/session identity, local-execution approvals, application permissions and service capabilities remain enforced in their reviewed adapters. A public class/capability, a template change or a routing label cannot grant access or authorize an operation.

No existing stable ID is renamed. No model/account/device/GUI operation, global installation or host configuration is needed to resolve or test the factory. The initial candidate intentionally preserves the current appearance and default values. Future common styling changes can use the inherited presentation fields without editing each pet.

## Team input and next shared seams

The integration, frontend and backend owners contributed the following boundaries. This candidate covers the configuration/catalog/lookup/gallery/settings slice above. The remaining seams should be migrated in successors after the independent hover/approval hotfix; they are not claimed as implemented here.

| Shared seam | Preserve when moving it into the factory |
| --- | --- |
| Identity, appearance and catalogue | Stable ID and canonical configured name; aliases remain identity-specific. Default colour/avatar/outfit and platform visibility can be deduplicated, while a native dated snapshot continues to describe observed state. |
| State and card shell | One vocabulary for idle, working, needs approval, needs information, finished and error; shared header action slots for Sessions, Configure and Close, plus scrolling/content slots. Approval/question ownership must win over home/terminal metadata. Do not duplicate or modify the concurrent hover/header/overview/CSS hotfix. |
| Asynchronous feedback | Carry exact pet + runtime target + request, fence late results and retain current focus. A template must never invent a session ID, conversation ID, source or native authority. |
| LLM conversation | Fast/reasoning/auto structure is reusable; actual model availability is provider-specific and an explicit selected model wins. Chat, history and drafts retain the full runtime SessionTarget and origin. Native global history is not the pet's library; observed terminals are not active turns; unknown usage remains null rather than false zero. |
| Application extension | Explicit app actions and attachment/plugin slots; Spoty owns its own music/PC-audio controls. MPRIS observation is not System permission. Service assistants may reuse the LLM profile/chat implementation while keeping application/logical ownership distinct from physical Codex. |
| Settings and drafts | Preserve provider-specific dirty/version guards, retained drafts and unknown/present/absent secret state; saving or displaying a mold never opens an approval or starts work. |
| Voice | Obtain the canonical current config.name for each request. Do not replace speech words, force a language, or turn learned recognition aliases into the spoken owner name. Current voice and lexicon implementations are unchanged. |
| Runtime access | Public capability/availability and a dated native snapshot are observations, not authorization. System operations retain original Prepared, current context/fence, owner/producer, TTL and actual writer checks. Always Allow remains untouched; email stays Ollama-only by default. |

No live account, installed-model or runtime capability is inferred from a factory entry. Routing metadata replaces existing static UI tables only. Logical owners are unique per chat/voice/task channel; sharing a physical Codex account never aliases GitHub's logical pet. Unknown or malformed provider lookups fail closed rather than matching an absent route.
