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

The Close button and a second click on the same pet hide its card immediately.
Closing a permission keeps the request pending and its pet marked. Switching
cards restarts the configured leave countdown. Companion mode closes straight
to the habitat instead of showing an intermediate compact island.
Answering a permission preserves that pet's focus; it no longer restores a
previously selected integration (such as Gravity after a Codex approval).
If the user manually selects another pet, completion preserves that choice.

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
