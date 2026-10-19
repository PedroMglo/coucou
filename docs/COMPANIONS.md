# Living companions on Linux

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

The installed frontend passed TypeScript, Vite and 607 JavaScript checks before
packaging. Remaining frontend follow-ups include visible recovery when native
chat close or resize fails, immediate agreement between gallery selection and
the detail header, and distinguishing an error from a request awaiting a human
decision. These follow-ups are not yet present in mochi.14. Weather location and
freshness were also reported to the backend owner: the current compact weather
contract does not identify the configured city.
