Estado atual (20/10/2026): a **0.5.1-mochi.23 está instalada permanentemente**,
com launcher/autostart e preservação de conversas e preferências verificados.
A aceitação nativa está parcial. Uma falha de apresentação após mudança de
monitor deixou o cartão invisível; o reinício do mesmo binário recuperou o
alfa da superfície e o hover voltou a abrir o cartão, confirmado visualmente.
A .22 inclui a correção nativa R2 revista; a prevenção da repetição após mudança real de ecrã ainda precisa de aceitação.
Os cinco chats próprios abriram e reabriram na .23 com identidade e rascunho
exatos, sem envio. O Ollama recuperou automaticamente os seus 73 caracteres
depois de reiniciar o Coucou, sem os voltar a escrever.
Sistema → Som tem controlos próprios; a .20 acrescenta leitura/pesquisa de rede
e atividade da conta. Os restantes controlos próprios e a matriz completa de
sessões/definições continuam por concluir. Os sete indicadores direitos
permanecem na barra. O estado detalhado e os critérios de aceitação estão em
[PETS-ARCHITECTURE.md](PETS-ARCHITECTURE.md).

# Living companions on Linux

> Current Linux delivery status is maintained in
> [PETS-ARCHITECTURE.md](PETS-ARCHITECTURE.md). The installed version is .23 (562 frozen source files);
> native presentation/hotplug acceptance is incomplete. The five LLM cards passed count checks in .17. Own System sound mute
> and unmute passed on a temporary virtual output in .18. Later version notes below are
> historical evidence, not confirmation that every requested capability shipped.

The current design uses Coucou's expanding island in the centre of the system
bar. Open **Pets**, choose the configured provider and use **Conversar** for the
floating chat. Its session sidebar distinguishes the pet's own conversations
from terminal sessions; opening or closing the window does not itself interrupt
work. A session's origin and identifier determine where a message or decision is
sent. When history cannot establish a unique active turn, it remains readable
and control is refused with an explanation.

There are five chat owners: Codex, Gravity, Copilot CLI, Ollama and GitHub.
Spoty owns music and PC audio; it no longer has an LLM chat, model picker or
new LLM session. Existing Spoty history is preserved. Models and stable names
are configured in `~/.config/coucou/pets.json`; email is enabled only for Ollama
by default. A pet without the relevant document capability must reject the drop.

The old shelter home widget and the former AI status widget are superseded by
the island. The native system blocks and transparent gaps remain. Right-side
icons for background apps, peripherals, CPU/system readings, clipboard, battery
and notifications remain on the bar. Only the old control-centre entry may be
retired after complete own island controls are accepted. The earlier shelter description below documents the
previous implementation and must not be used to re-enable that widget.

Super alone toggles the companion voice session. Say the configured pet name
before the request. Voice enters the selected pet's own conversation, without
pasting into a terminal. Stop ends listening; the pet's explicit task controls
remain responsible for work already admitted. Mixed PT/EN recognition and real
Raquel latency still need the later quality acceptance recorded in the architecture.


Each declared integration has one pet with a stable identity. Click it to open
its own card beside it; permissions remain explicit user decisions. Clicking
a pet does not open its underlying application. The Ollama pet owns the local
chat, manual conversation and dictation. Their feedback appears in its speech
bubble; the old side pill is hidden while companions are enabled.

## Placement and behaviour

`companionBarMode` uses a version-1 geometry document at
`$XDG_RUNTIME_DIR/coucou-bar-space.json`. The station's DMS `MochiSpace` plugin
publishes the logical screen bounds, bar height and actual occupied rectangles.
GIO watches replacement as well as changes. Geometry is bounded and validated;
missing geometry hides the pets, retaining the tray entry. Only free intervals
with ten pixels of clearance are used. Bar controls and application content
are never paths for ambient wandering. The ordinary clearance mode remains
available when bar geometry is not configured.

All configured pets are available from a small home button in the free bar space,
including immediately after login when no client is open. Clicking it reveals a
scrollable row of sleeping residents in the bar; selecting one recalls its exact
card without launching an application. The drawer reserves its own geometry and
releases its input region when closed. A mouse wheel scrolls crowded drawers.

Opening a known AI CLI/application recalls its configured pet before the first
prompt or hook. Linux reconciles same-user executable identities every two seconds,
including clients already open before Coucou. It emits only presence changes, never
prompts, process arguments, titles or session files. Codex app-server transports,
Electron helper processes and the Ollama service do not count as user clients.
An unchanged process sample does not postpone rest. Closing the last observed client
also clears a missed SessionEnd, while native conversations and pending decisions
retain their ownership. Other integrations retain their own event/MPRIS sources.

Quiet pets fade into the shelter after 60 seconds without meaningful activity.
Work, permissions and questions stay visible. Clicking the home button, a manual
call, an original card or voice recalls a hidden pet. Reduced motion, static
sleeping canvases and the eight-second interaction grace remain supported.
The habitat waits for actual published free bar space before mapping, preventing
an empty early surface being placed below the bar during login. A newly mapped
webview requests the current snapshot; monitor changes reconfigure the habitat.

Thinking, working and listening are expressions and animations on the pet itself,
with a speaking mouth for public responses. They no longer create permanent status
cards. Brief actual messages use a small speech tail beside the pet, positioned on
every animation frame and clipped to free bar space without crossing other pets or
bar controls. The text can be read in full by explicitly clicking the bubble.
Completion hooks supply the final speech without stealing focus or automatically
opening a full result card. Approval/question cards retain their original controls.

Real activity wakes resting pets for walking. Active pets take longer random walks,
pause, look around and greet neighbours. Alerts take priority over movement.
An open card holds its own pet still. Bar changes reposition its card without
changing the selected integration or view.

The habitat cannot take keyboard focus. Only separate body circles and
explicitly opened menus/bubbles receive pointer input. Empty space passes
clicks through. Reduced motion and pause remain supported. Notification sounds
have one owner; the habitat does not initialize another active sound engine.

## Voice, music and resources

Ctrl+Alt+V holds dictation; Ctrl+Alt+C starts/ends a manual conversation.
Ctrl+Alt+Enter finishes capture and Ctrl+Alt+Esc cancels. The Ollama pet menu
also exposes these actions. Continuous hotword listening is optional and is
disabled on this station to keep Bluetooth stereo between uses. Capture ends
before ASR/model generation. The local voice backend restricts Ollama to the
loopback address, bounds requests and supports cancellation without shell or
permission automation. Transcripts are not diagnostic logs.

The ASR model loads on use, not simply because the shortcut is enabled. One
idle task releases its cache after 60 seconds without requests, retaining an
in-use recognizer safely. Active processing may delay release. No animation,
hover or geometry update calls an LLM.

Spotify uses actual MPRIS state. The card and Play are separate actions;
playback does not optimistically claim success. A Linux station may provide
`coucou-spotify-background` on PATH. It must start an authenticated player out
of the way without raising/moving existing windows. The station's Hyprland
helper uses a short-lived class rule to handle Flatpak process indirection.
Without a working helper/player queue, opening Spotify remains an explicit
fallback. MPRIS does not provide account sign-in or arbitrary music search.

Rendering is bounded to 24 fps while walking/dancing/greeting, 4 fps while
awake and stationary, 1 fps for reduced motion, and no recurring frames when
all pets sleep or are absent. Presence transitions use the same bounded rendering loop.
The separate native client reconciliation runs every two seconds only while the
habitat is enabled; unchanged samples emit no events and restart no frames. Sleeping canvases are cached even while neighbours animate.
Hidden habitats stop timers. Regions update at most 10 times a second and only
if changed. The settings animation is gated by native window visibility as
well as document visibility; WebKit alone did not reliably suspend it.

## Validation and platform limits

For mochi.4, TypeScript, all 457 JavaScript tests and the Fedora RPM build pass.
Tests simulate ten minutes of movement, narrow displays, live bar resizing,
bounds/collisions, presence transitions, idle-session rest, manual recall,
unchanged hover targets, separate identities, approval focus and Spotify's
actual-player confirmation. This presence update changes no Rust application
logic. Linux compilation uses the project's Fedora container; never deploy a
test-rebuilt hook over a relay whose hash a user already trusted.

An equivalent ten-minute lifecycle simulation with six open quiet sessions
reduced recurring-frame demand from 600 to 60 seconds; configured-only pets
went from six visible to zero. These are model observations, not CPU savings.
Native checks confirmed a synthetic session appearing, resting without movement
after a minute, then shrinking/fading out on SessionEnd. The test executes no
agent commands. The Codex/Gravity approval regression is also covered by state
tests; concurrent real requests can prevent an isolated native approval test.

Native station checks confirmed bar placement, clicking Spotify without
launching it, then Play yielding MPRIS Playing on an inactive special workspace
with terminal focus preserved. Audio capture tests and real-user dictation
checks cover Sony and MOMENTUM 4 headphones; their transport correction lives
in the station repository. Browser simulations do not establish compositor
focus, fullscreen, scale or monitor behavior. Packaging, final native acceptance
and resource measurements remain part of the installation review.

Code and dependencies belong to this repository. Host configuration and
sanitized measured results belong to `PedroMglo/sysworkspace`. Windows and
unsupported Linux desktops retain the original companion path; Linux-specific
changes do not imply those platforms were exercised.

## Original cards, closing and provider ownership

Hooks route permissions, questions, errors and completed results to the original
full card anchored to the owning pet. A permission retains its command and
Allow/Deny controls. The habitat never grants a decision on a pet click.
Missing owners wait briefly for their snapshot; no other pet is substituted.

The Close button and a second click on an ordinary pet hide its card immediately.
A pet waiting for a decision reopens its approval or question instead of its home.
Closing a permission keeps the request pending and its pet marked. Switching
cards restarts the configured leave countdown. Companion mode closes straight
to the habitat instead of showing an intermediate compact island.
Answering a permission restores the pet selected before the interruption.
If the user manually selects another pet meanwhile, completion preserves that
choice. A request handed back to the terminal remains marked as waiting there;
reviewing it focuses that exact session. Native chat decisions carry their full
target and request ID into the corresponding conversation, including after reload.

The island webview owns event handlers even while no card is visible. On Linux
it must remain mapped as a transparent, zero-input strip: hiding its GTK window
suspended event processing and broke pet clicks. Opening wakes its geometry
before delivering the owning pet event. This is separate from rendering: hidden
islands still stop their frame loop and pointer polling.

Model pickers and drafts follow the selected provider. Changing pets closes
and invalidates the old picker, including late model-list responses. A Codex
pet cannot invoke the legacy Ollama voice action. Voice feedback without an
explicit new provider contract belongs only to the Ollama pet.

The independent conversation worktree is `../coucou-codex`. Its committed
text-account base a6479f01f5712d618f5f4682abec99495267eaf0 is incorporated here;
version 0.5.1-mochi.2 integrates its continuous voice sessions. Voice events now
carry the owning pet ID; local manual dictation retains Ollama ownership.
Presentation changes must keep that ownership boundary, without adding
cross-provider fallbacks. See [voice accounts](LINUX-VOICE-ACCOUNTS.md).

Music assistance uses MPRIS, not an LLM. Recent-track search is bounded to fifty
tracks in memory; explicit Spotify links use OpenUri without Raise. Catalog
search is a separately labelled external action. Optional ad ducking requires
an explicit spotify:ad: ID and restores volume only when the user has not
changed it. Stalled playback notices require sixty seconds without progress.
The native music card scrolls within its panel, keeping these controls reachable.
Real advertisements and full-catalog OAuth search have not been validated.

## mochi.6 acceptance

TypeScript/Vite, 466 JavaScript tests and native workspace tests pass (511 app
checks plus 34 relay checks; ten platform-specific tests remain ignored).
Browser interaction checks use the compiled companion surface at the station's
actual 42-pixel band and free interval: thinking has no text card, speech stays
inside that band, six residents remain accessible, and selecting Gravity routes
only to Gravity. These checks do not execute a model or an application command.
Login mapping and actual compositor stacking are separately checked at delivery.

## Assistant architecture and delivery status

The current implementation, per-pet roles, model defaults, conversation gaps,
voice validation and remaining acceptance work are tracked in
[PETS-ARCHITECTURE.md](PETS-ARCHITECTURE.md). This replaces the chronological
implementation notes for mochi.7 through mochi.11. Earlier sections above describe
the habitat baseline; the architecture guide is authoritative for newer provider,
model-profile and service-assistant behavior.

## Current UX review and deferred audio work

The October 2026 review covers every chat surface, settings route and nested
control, including drafts, errors, repeated clicks, keyboard navigation, file
delivery and concurrent requests. Native observation is required separately from
mocked UI and transport checks. Rendering a button is not functional acceptance.
The immediate delivery priorities are reviewable help requests and the central
home combining pets with clock, date, media, weather, notifications and desktop
panels. The revised home has two primary pages, Pets and System. Pets opens
on the whole catalogue, including resting pets; individual details are opt-in.
Its session, task, provider, local-model and profile tools remain available as
subsections. Horizontal touch/trackpad navigation and page switches preserve
profile drafts. The gallery uses the original lit bot renderer, animation and
outfits rather than the small flat pill avatars. Native acceptance confirmed the real approval for its exact session.
The installed mochi.14 presentation was checked again after login on the laptop
display at scale 2: all six pets, both primary pages and all five management
subsections remain accessible. Each system action opened its actual DMS panel;
calendar, media and weather open from the central entry, as do notifications and
quick settings. The old clock/media/weather block is absent. The five opening
acknowledgements took 0.45–0.49 seconds in that run; this measures panel opening,
not a full audit of the controls inside DMS. Media playback was inactive.
The local-model controls were also exercised with the installed Gemma model:
loading produced a real Ollama memory entry, and the confirmed release returned
it to the initial empty state. No chat inference or model download was needed.
The system repository owns the versioned host cutover. The follow-up transparency
repair was also observed on the installed desktop: wallpaper is visible around
the compact centre while the side controls remain accessible. Native
file delivery also passed for the island interior, a direct pet drop and a folder
inside the floating chat, with no inference needed to accept an attachment.

The user explicitly deferred new sound, speech, voice and dictation work until
the text and task experience is mature. Retain the following for that later phase:

- Device and permission feedback, cancellation and recovery for audio capture.
- Measured streaming and playback latency, with a reliable text equivalent.
- Voice selection and language handling per pet, including interruption behavior.
- Recording and transcription storage acknowledgements, recovery and background
  writer ordering before accepting that existing audio workflow.

The settings review corrected phone clipboard errors with visible manual recovery,
and serialised complete service selections so rapid changes preserve every choice.
Initial loading gates edits; failed saves retain the latest selection for retry.
The colour palette now closes when its page leaves, including keyboard navigation.
The centre reserves space for errors instead of covering the pet actions. Browser
journeys cover these failure and recovery paths. Recap persistence acknowledgements
remain a reported, deferred backend item.

The installed mochi.14 frontend passed TypeScript, Vite and 607 JavaScript checks
before packaging. The mochi.15 frontend is frozen with TypeScript, Vite and
673 JavaScript checks passing; the first native candidate passed all thirteen system openings and five pet
chat ownership checks. Spotify retains its separate MPRIS music controls. It adds visible recovery for native chat close/resize errors,
immediate gallery/roster agreement, and separate error and approval actions.
Session archives render fifty rows per origin, with progressive loading, full
archive search and the selected archived session retained. A synthetic archive
of 1,647 sessions passed search, paging and keyboard-focus checks.

The System page now includes Wi-Fi/network, Bluetooth, audio, battery, recording,
clipboard, disks and tray controls, in addition to the five existing entries.
Status remains explicit when unknown or stale; device actions use the native
DMS panels. Exact opaque tray identifiers route application activation and menus.
A recording indicator remains visible on compact and Pets views while recording
is active, and shows uncertainty after a failed reading. Browser checks cover
all eight routes, exact tray actions, repeated clicks, failures, stale values and
widths from 320 to 1,440 pixels. No real recording was started. Native DMS panels
and the clipboard contents area were observed on the laptop display; tray close
by X, Escape and outside click, plus tray-menu Escape/outside, returned real
keyboard input to an isolated test editor. Disk and tray-menu alignment was corrected in the host-owned QML and
observed again with both popups centred. The right legacy block stays until the
integrated mochi.15 centre passes acceptance and the host owner performs cutover.
Native ultrawide acceptance remains pending because that display is disconnected.

The central gallery/card now consumes its own native file-drop events. An exact
pet or explicit recipient choice owns the one-use document token. Cancellation,
slow ingestion, expiry and a failed handoff remain recoverable without sending
a chat or consuming a successful receipt twice. Browser delivery checks passed;
the first native PDF delivery reached the exact Ollama owner but exposed a
gallery overlay after handoff. The frontend now ignores stale pointer entries
from the former expanded region; a focused hover-enabled browser regression
passes, including reopening through a real compact hover. The rebuilt candidate passed native PDF and folder delivery to Ollama with
the actions fully accessible, plus reopening without a new ingestion. Removing
the PDF attachment and cancelling the folder delivery both cleared the chat
attachment. The compact clock and genuine hover opening remained functional.

Ollama execution presentation uses verified, conversation-specific model capability
metadata. Conversation-only models are labelled accordingly, and task actions
remain subject to the pet policy and an exact, once-only command review. The
review retains the complete original JSON as well as literal arguments, working
directory and timeout. Task details show individual execution receipts, including
unknown outcomes and unconfirmed cleanup; closing a window does not establish
that a task stopped. Policy changes update the floating-chat badge immediately.
The command review and receipt contracts were checked with synthetic events.
A native QA prompt using Qwen3:1.7b was rejected for invalid command arguments
before approval; the UI reported that no command started and displayed the
verified model capability. A later Qwen3.5:4b-q4_K_M proposal reached the real approval card. The first
approval expired safely; an explicit task retake presented a new human decision.
After that human approval, the receipt reported one exit0 execution,13 stdout
bytes,0 stderr bytes and no truncation; the UI showed the literal
MOCHI_QA15_OK result. No automatic decision, retry or parser relaxation was used.
Stopping a later pending command review retired the approval and busy indicator;
the native journal confirmed cancellation by the user with no command start or
output. The empty chat after Stop remains a reported follow-up: the UI should
show the preserved request and verified interruption receipt.
Weather location and freshness remain reported backend follow-ups: the compact
contract does not yet identify the configured city.

The shared System centre is independent of desktop-pet visibility. Turning
companions or the pet bar off leaves the clock and System controls available
when System centre is enabled; its default opening is System. Browser checks
cover flag changes, retained profile drafts, sixty-five seconds idle and saves
that preserve the actual companionMode value. A native pre-recomposition run
retained the clock for more than eighty seconds without interaction.

Linux now has a dedicated pet-cards.html entry. Pet-control owns the only home,
gallery and management surface. The contextual controller reuses the original
bot renderer and document, chat, Spotify and decision components, but does not
construct the classic greeting, wake strip, compact mini-grid, gallery, ticker,
plan timer or visual listeners. Its separate CardLifecycle has only open/closed
transitions. It does not construct the singleton desktop controller, subscribe
to its events, publish its snapshots or query desktop_info. Work/completion and
music updates publish metadata without revealing a second surface. The original
index presentation remains for the other targets, outside the Linux runtime.

Explicit document delivery and requests work with desktop pets off. The focused
pet retains its provider and document context, independently of the global chat
provider. Help opens the exact native request/session. Closing a pending decision
does not answer it; a later handoff closes the card without reopening the old
home. Contextual headers contain pet actions, sessions, attachment, configuration
and close controls, with no second home or gallery. Session and profile management
return to the central surface, while application settings open the existing hub.

Explicit home/management opening with both passive presentations disabled is
temporary: closing returns to the disabled state without writing preferences or
waking pets. If System integration is disabled, its page explains that state and
opens application settings; it does not poll or invoke denied system actions.
The native DMS lease fallback remains continuity of system controls after the
application exits, rather than another Coucou presentation.

Browser composition checks verify absence of the legacy DOM, state machine,
singleton and listeners, plus startup infrastructure, slow cursor movement,
work/completion/music updates, System-only document/email ownership and draft
retention, exact approval fold/reopen/handoff, and explicit off-flag management.
A form is synchronised before accepting input, preventing an immediate first edit
from being erased by its initial redraw. Hidden views leave the accessibility and
pointer paths immediately; old pet content does not linger through a transition.
The email card reserves enough height for both actions. Isolated email checks
also cover text delimiters, portal error/retry, pending-click guards, per-pet and
folder capabilities, keyboard navigation and 360-pixel layout. They do not open a
mail client or send email; native compose acceptance is separate.

Review follow-up routes each modern help request by its immutable request ID
and canonical pet/provider/origin/conversation target, including service pets
whose logical owner differs from their CLI backend. Routing does not restore a
conversation or replace another pet's focus/draft. Pending routing metadata is
bounded and uses snapshot epochs as well as ended-request tombstones, so a late
read cannot resurrect a retired request after eviction. Failed opening has an
explicit retry for the same live request and target.

Native opening now returns whether the requested window was actually shown.
A withdrawn or superseded open does not emit a presentation receipt. Request-bound
Help is checked natively through presentation; this reads request authority and
never consumes or answers it. Contextual cards use their local presentation token
and exact target to ignore late receipts for another pet, conversation or reopened
card. A generic chat opened from the centre is independent and does not fold a
card whose identity cannot be established. Cross-window Help for the same live
request/CID can fold that exact card. A System panel's successful hide-popup
request deliberately folds the contextual card while preserving its pending
approval and drafts. Ending a different request never releases another approval's
pin. Two isolated browser realms verified these cross-window rules without
inference, decisions, session restoration or settings writes.

Desktop pets retain their own activity, speech and document targets. Native
baseline acceptance on candidate8 confirmed a working Codex pet receiving both
a PDF and a folder directly, with correct ownership and cancellation, and an
Ollama pet appearing during actual local responses. Closing the chat during a
response exposed a missing speech bubble; candidate9 addresses the presentation
failures and the backend separately supplies activity for plain responses.
When other pets leave too little room in the bar, speech falls below the bar
with its tail attached to the owner, a readable preview and its navigation
action. Completion gives the final streamed text a fresh reading window;
duplicate receipts do not extend it. Message navigation keeps the immutable
conversation target captured by that turn, even if the current conversation
changes, and habitat Help passes the current request ID through the native guard.
An expired decision cannot reopen an overview on the next render frame. The
dashboard uses its own avatar so the controller bot does not cover its metrics.
Native9 acceptance confirmed an actual Ollama pet during Qwen and Gemma replies,
readable speech with the system centre expanded, physical message navigation to
conversation A while B remained current, and keyboard input in another application
while the chat stayed open. Direct PDF and folder drops on the Ollama pet reached
its document actions; folder cancellation closed the card. Six live habitat/bar
preference changes preserved an explicit chat and its unsent synthetic draft;
re-enabling the habitat after closing that chat did not reopen it.

Candidate10 also captures the native session identity with each public message.
Its read action becomes available only after a read-only canonical external-session
lookup completes. Missing, owned or unsupported native identities show an honest
explanation and an explicitly labelled session-list action. Native IDs never become
invented Mochi conversation IDs. The lookup itself cannot open a chat or supersede
a newer manual opening; a new click uses the already confirmed target immediately.
A pending request has its own origin and action, independently of older speech.
Modern centre availability retires the old shelter DOM and input even when both
centre preferences are off. Unit and real-page browser checks cover these cases,
including late lookups, removal/reappearance of a pet and disabled-centre flags;
candidate10 still requires matching native acceptance.
The pet overview clears the previous conversation's usage before drawing a newly
current conversation. Missing or failed measurements stay unknown; late results
for another conversation or pet cannot replace the current counts. Real-page
checks cover pending/error reads and results completing out of order.

The native email journey reached the correct Ollama PDF and preserved all three
synthetic form fields. Betterbird opened percent-encoded subjects and two compose
windows were observed after one automated opening action. This is a reported
backend blocker: exact client fields, attachment and discard remain unaccepted;
no email was sent during the test.

Source11 adds explicit presentation acknowledgement for legacy permission cards:
the exact request, session and pet must still match after rendering, native surface
preparation and acknowledgement. Controls stay disabled until that acknowledgement
and a fresh activation guard. Stale callbacks cannot decide a newer request, and
a rejected answer remains visibly unresolved. Eight isolated real-page browser
scenarios cover delayed acknowledgements, folding, same-request reopening,
handoff and multi-step questions. These use a mock transport, not native decisions.
The source passes 696 JavaScript tests, TypeScript and Vite. Native11 Codex and
Copilot trials then showed the exact permission card and accepted presentation
receipt; closing each synthetic client handed its request back without a decision
or execution. Other native permission paths remain under integration review.
The card composition regression also preserves document/email
drafts and confirms that slow hover does not recreate the classic island.

The native Stop metadata now renders the interrupted original request as a task
status, separately from assistant messages, without sending or resuming anything.
Floating-chat browser checks at widths 360 and 760 preserve the draft, show
persistence failures, scroll long queries and clear obsolete status on a completed
new turn; another conversation cannot inherit the notice. A source11 compact
inline clipping defect placed the original request above the visible card after
composer focus. Source12 reserves room when that status appears, recalculates
geometry when it clears, and bounds query and feedback scrolling. Source-level
browser checks now confirm both short and long original requests, plus the draft
composer, inside the actual card clip below its header. Matching native12
acceptance remains pending; source11 was retained only for the isolated native
permission trial.

The next user journeys include truthful active-session counts on each pet,
management of work after a terminal window closes, and opening all conversations
in the expanded chat by default, with terminal opening as an explicit alternative.
Closing a window, interrupting a task and retiring a managed session require
distinct labels and verified backend effects; unknown liveness must not be counted
as confirmed active work. Backend lifecycle validation is coordinated separately.
The System integration also remains transitional: the intended island owns its
widget controls rather than opening native Linux/DMS panels. Compact pets above
date/time and pointer-leave dismissal (excluding expanded chats) are recorded
follow-ups after the permission and production delivery checks.

The recompiled Linux runtime still requires native acceptance of slow hover,
startup/flags, System-only documents and exact approvals, followed by close/crash
fallback and the host-owned retirement of the right legacy block. Installation,
restoration of the preserved production draft and focused signed commits remain
the integration owner's final delivery checkpoints.
