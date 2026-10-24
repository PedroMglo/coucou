// Everything the Coucou window shows and edits about dictation: the history,
// the numbers, the dictionary, the snippets, the styles and the preferences.
// Small JSON files in the app's folder (%LOCALAPPDATA%\Coucou\Flow or
// ~/.local/share/coucou/Flow), on this computer only. Port of FlowStore.swift.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::logic::FlowKey;
use super::text::{self, new_id, Day, Replacement, Snippet};

/// One dictation, kept for the history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    /// Milliseconds since 1970.
    pub date: u64,
    pub text: String,
    /// The app it was pasted in ("Slack"), when known.
    #[serde(default)]
    pub app: Option<String>,
    pub words: u64,
    /// How long you spoke.
    pub seconds: f64,
}

/// The numbers of one day. Kept even when the history is emptied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DayStats {
    pub words: u64,
    pub seconds: f64,
    pub dictations: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    /// "2026-10-14" → that day's numbers.
    pub days: BTreeMap<String, DayStats>,
    /// App name → words dictated in it.
    pub apps: BTreeMap<String, u64>,
}

/// How the words are written in an app, when the clean-up is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Style {
    /// Only the dictionary, replacements and snippets: code, terminals, AI chats.
    Verbatim,
    /// Without the "um" and the repeats, punctuation fixed, the tone kept.
    #[default]
    Natural,
    /// Full sentences, for mails.
    Polished,
}

/// An app with its own style. `id` is the program's name in lower case
/// ("outlook", "slack", "windowsterminal"), what the foreground window tells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppStyle {
    pub id: String,
    pub name: String,
    pub style: Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    #[default]
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SummaryEngine {
    #[default]
    Claude,
    Off,
}

/// The preferences of dictation and of the window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Prefs {
    /// Hold a key to dictate anywhere. Off until the user turns it on.
    pub enabled: bool,
    pub key: FlowKey,
    pub edge: Edge,
    /// The microphone by name; empty is the system's default.
    pub mic: String,
    /// Days the history is kept: 0 = not kept, -1 = forever.
    pub keep_days: i64,
    /// Clean the words with Claude before pasting them (needs the Claude key).
    pub cleanup: bool,
    /// The style of the apps that have none of their own.
    pub default_style: Style,
    pub summary_engine: SummaryEngine,
    /// Open the Coucou window when Coucou is opened (not when it starts with the computer).
    pub open_at_launch: bool,
    /// « OK Coucou »: listen for it and answer with Claude (needs the Claude key).
    pub voice: bool,
    /// How Coucou answers out loud: "system", "elevenlabs" or "off".
    pub voice_output: String,
    /// The ElevenLabs voice: "female" or "male".
    pub voice_gender: String,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            enabled: false,
            key: FlowKey::default(),
            edge: Edge::Bottom,
            mic: String::new(),
            keep_days: 30,
            cleanup: false,
            default_style: Style::Natural,
            summary_engine: SummaryEngine::Claude,
            open_at_launch: true,
            voice: false,
            voice_output: "system".into(),
            voice_gender: "female".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Words {
    pub words: Vec<String>,
    pub replacements: Vec<Replacement>,
}

/// Apps that get a style from the start: mails polished, chats natural, code
/// and AI chats as said.
pub fn default_app_styles() -> Vec<AppStyle> {
    let s = |id: &str, name: &str, style| AppStyle { id: id.into(), name: name.into(), style };
    vec![
        s("outlook", "Outlook", Style::Polished),
        s("olk", "Outlook (new)", Style::Polished),
        s("thunderbird", "Thunderbird", Style::Polished),
        s("slack", "Slack", Style::Natural),
        s("discord", "Discord", Style::Natural),
        s("whatsapp", "WhatsApp", Style::Natural),
        s("telegram", "Telegram", Style::Natural),
        s("teams", "Microsoft Teams", Style::Natural),
        s("claude", "Claude", Style::Verbatim),
        s("windowsterminal", "Windows Terminal", Style::Verbatim),
        s("code", "Visual Studio Code", Style::Verbatim),
        s("cursor", "Cursor", Style::Verbatim),
        s("gnome-terminal-server", "Terminal", Style::Verbatim),
        s("konsole", "Konsole", Style::Verbatim),
        s("kitty", "kitty", Style::Verbatim),
    ]
}

pub struct Store {
    pub history: Vec<Entry>,
    pub stats: Stats,
    pub words: Words,
    pub snippets: Vec<Snippet>,
    pub app_styles: Vec<AppStyle>,
    pub prefs: Prefs,
    /// When the history is not kept, the last dictation still waits here
    /// (memory only) for "paste again".
    last_unkept: Option<String>,
    folder: PathBuf,
}

fn load<T: DeserializeOwned>(folder: &std::path::Path, name: &str) -> Option<T> {
    let text = std::fs::read_to_string(folder.join(name)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn today() -> Day {
    let t = crate::platform::local_time();
    Day { year: t.year as i32, month: t.month, day: t.day }
}

impl Store {
    pub fn folder() -> PathBuf {
        crate::settings::data_dir().join("Flow")
    }

    pub fn load() -> Store {
        Self::load_from(Self::folder())
    }

    pub fn load_from(folder: PathBuf) -> Store {
        let mut store = Store {
            history: load(&folder, "history.json").unwrap_or_default(),
            stats: load(&folder, "stats.json").unwrap_or_default(),
            words: load(&folder, "words.json").unwrap_or_default(),
            snippets: load(&folder, "snippets.json").unwrap_or_default(),
            app_styles: load(&folder, "styles.json").unwrap_or_else(default_app_styles),
            prefs: load(&folder, "prefs.json").unwrap_or_default(),
            last_unkept: None,
            folder,
        };
        store.prune(now_ms());
        store
    }

    fn save<T: Serialize>(&self, value: &T, name: &str) -> std::io::Result<()> {
        use std::io::Write;
        // Serialization must fail before creating/changing anything on disk.
        let bytes = serde_json::to_vec_pretty(value).map_err(std::io::Error::other)?;
        crate::platform::ensure_private_dir(&self.folder)?;
        let tmp = self.folder.join(format!(".{name}.{}-{}.tmp", std::process::id(), new_id()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)] {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // If exclusive creation fails, this call owns no file to remove.
        let mut file = options.open(&tmp)?;
        let write = (|| -> std::io::Result<()> {
            file.write_all(&bytes)?;
            file.sync_all()
        })();
        drop(file);
        let result = write.and_then(|()| std::fs::rename(&tmp, self.folder.join(name)));
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result
    }

    /// Persist the replacement before publishing it in RAM. History pruning
    /// stays with the caller after this preference write has succeeded.
    pub fn set_prefs(&mut self, prefs: Prefs) -> std::io::Result<Prefs> {
        self.save(&prefs, "prefs.json")?;
        Ok(std::mem::replace(&mut self.prefs, prefs))
    }
    pub fn set_words(&mut self, words: Words) -> std::io::Result<()> {
        self.save(&words, "words.json")?;
        self.words = words;
        Ok(())
    }
    pub fn set_snippets(&mut self, snippets: Vec<Snippet>) -> std::io::Result<()> {
        self.save(&snippets, "snippets.json")?;
        self.snippets = snippets;
        Ok(())
    }
    pub fn set_styles(&mut self, styles: Vec<AppStyle>) -> std::io::Result<()> {
        self.save(&styles, "styles.json")?;
        self.app_styles = styles;
        Ok(())
    }
    fn save_history(&self) {
        if let Err(err) = self.save(&self.history, "history.json") {
            crate::log::line(format!("flow: could not save history.json: {err}"));
        }
    }
    fn save_stats(&self) {
        if let Err(err) = self.save(&self.stats, "stats.json") {
            crate::log::line(format!("flow: could not save stats.json: {err}"));
        }
    }

    /// The words with the replacements and the dictionary's spelling, snippets set aside.
    pub fn prepare(&self, raw: &str) -> text::Prepared {
        text::prepare(raw, &self.words.words, &self.words.replacements, &self.snippets)
    }

    pub fn style_for(&self, app: Option<&str>) -> Style {
        app.and_then(|id| self.app_styles.iter().find(|s| s.id == id))
            .map(|s| s.style)
            .unwrap_or(self.prefs.default_style)
    }

    /// The last thing dictated, for "paste again".
    pub fn last_text(&self) -> Option<String> {
        self.history.first().map(|e| e.text.clone()).or_else(|| self.last_unkept.clone())
    }

    pub fn record(&mut self, text: &str, app: Option<&str>, seconds: f64, day: Day, now: u64) -> Option<Entry> {
        let count = text::word_count(text) as u64;
        if count == 0 {
            return None;
        }
        let d = self.stats.days.entry(day.key()).or_default();
        d.words += count;
        d.seconds += seconds;
        d.dictations += 1;
        if let Some(app) = app.filter(|a| !a.is_empty()) {
            *self.stats.apps.entry(app.to_string()).or_default() += count;
        }
        self.save_stats();
        if self.prefs.keep_days == 0 {
            self.last_unkept = Some(text.to_string());
            return None;
        }
        let entry = Entry { id: new_id(), date: now, text: text.to_string(), app: app.map(String::from), words: count, seconds };
        self.history.insert(0, entry.clone());
        self.prune(now);
        self.save_history();
        Some(entry)
    }

    pub fn delete(&mut self, id: &str) {
        self.history.retain(|e| e.id != id);
        self.save_history();
    }

    /// Empties the history; the numbers stay.
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.last_unkept = None;
        self.save_history();
    }

    /// Empties the history and the numbers.
    pub fn clear_everything(&mut self) {
        self.clear_history();
        self.stats = Stats::default();
        self.save_stats();
    }

    pub fn prune(&mut self, now: u64) {
        let before = self.history.len();
        if self.prefs.keep_days == 0 {
            self.history.clear();
        } else if self.prefs.keep_days > 0 {
            let limit = now.saturating_sub(self.prefs.keep_days as u64 * 86_400_000);
            self.history.retain(|e| e.date >= limit);
        }
        if self.history.len() != before {
            self.save_history();
        }
    }

    pub fn streak(&self, today: Day) -> u32 {
        let days: HashSet<String> = self.stats.days.iter().filter(|(_, d)| d.words > 0).map(|(k, _)| k.clone()).collect();
        text::streak(&days, today)
    }

    /// Words, seconds and dictations since `from` (None: everything).
    pub fn totals(&self, from: Option<Day>) -> (u64, f64, u64) {
        let from = from.map(Day::key);
        let mut out = (0, 0.0, 0);
        for (key, d) in &self.stats.days {
            if from.as_ref().is_none_or(|f| key >= f) {
                out.0 += d.words;
                out.1 += d.seconds;
                out.2 += d.dictations;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (Store, PathBuf) {
        let dir = std::env::temp_dir().join(format!("coucou-flow-{}-{}", std::process::id(), new_id()));
        (Store::load_from(dir.clone()), dir)
    }

    #[test]
    fn a_dictation_counts_and_is_kept_for_the_chosen_days() {
        let (mut s, dir) = temp_store();
        let day = Day { year: 2026, month: 10, day: 14 };
        let now = 1_800_000_000_000;
        s.record("bonjour à tous", Some("slack"), 2.0, day, now).unwrap();
        s.record("?", None, 1.0, day, now);
        assert_eq!(s.history.len(), 1);
        assert_eq!(s.stats.days["2026-10-14"].words, 3);
        assert_eq!(s.stats.apps["slack"], 3);
        assert_eq!(s.streak(day), 1);
        assert_eq!(s.totals(None), (3, 2.0, 1));
        // Older than 30 days: gone from the history, still in the numbers.
        s.prune(now + 31 * 86_400_000);
        assert!(s.history.is_empty());
        assert_eq!(s.totals(Some(day)).0, 3);
        // Not kept: the last one still waits for "paste again".
        s.prefs.keep_days = 0;
        assert!(s.record("encore un mot", None, 1.0, day, now).is_none());
        assert_eq!(s.last_text().as_deref(), Some("encore un mot"));
        // Saved on disk and read back.
        let again = Store::load_from(dir.clone());
        assert_eq!(again.stats.days["2026-10-14"].dictations, 2);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn styles_follow_the_app_or_the_default() {
        let (mut s, dir) = temp_store();
        assert_eq!(s.style_for(Some("outlook")), Style::Polished);
        assert_eq!(s.style_for(Some("code")), Style::Verbatim);
        assert_eq!(s.style_for(Some("notepad")), Style::Natural);
        s.prefs.default_style = Style::Polished;
        assert_eq!(s.style_for(None), Style::Polished);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn prefs_read_from_an_older_file_keep_their_defaults() {
        let p: Prefs = serde_json::from_str(r#"{"enabled":true,"key":"right_alt"}"#).unwrap();
        assert!(p.enabled);
        assert_eq!(p.key, FlowKey::RightAlt);
        assert_eq!(p.keep_days, 30);
        assert!(p.open_at_launch);
    }
}

#[cfg(test)]
mod setter_io_tests {
    use super::*;
    use serde_json::{json, Value};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("coucou-flow-setters-{}-{}", std::process::id(), new_id()));
            let mut options = std::fs::DirBuilder::new();
            #[cfg(unix)] {
                use std::os::unix::fs::DirBuilderExt;
                options.mode(0o700);
            }
            options.create(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    #[derive(Clone, Copy)]
    enum Setting { Prefs, Words, Snippets, Styles }
    impl Setting {
        fn all() -> [Self; 4] { [Self::Prefs, Self::Words, Self::Snippets, Self::Styles] }
        fn key(self) -> &'static str {
            match self { Self::Prefs => "prefs", Self::Words => "words", Self::Snippets => "snippets", Self::Styles => "styles" }
        }
        fn filename(self) -> String { format!("{}.json", self.key()) }
        fn replacement(self, store: &Store) -> Value {
            match self {
                Self::Prefs => {
                    let mut prefs = store.prefs.clone();
                    prefs.mic = "Microfone — São João 🐾".into();
                    prefs.enabled = !prefs.enabled;
                    json!(prefs)
                }
                Self::Words => {
                    let mut words = store.words.clone();
                    words.words.push("Évora 東京 🐾".into());
                    words.replacements.push(Replacement { id: "new-replacement".into(), from: "cafe".into(), to: "café".into() });
                    json!(words)
                }
                Self::Snippets => {
                    let mut snippets = store.snippets.clone();
                    snippets.push(Snippet { id: "new-snippet".into(), trigger: "assinatura".into(), text: "Até amanhã!\n東京 🐾".into() });
                    json!(snippets)
                }
                Self::Styles => {
                    let mut styles = store.app_styles.clone();
                    styles.push(AppStyle { id: "synthetic-editor".into(), name: "Editor português 🐾".into(), style: Style::Polished });
                    json!(styles)
                }
            }
        }
        fn apply(self, store: &mut Store, payload: Value) -> std::io::Result<Option<Prefs>> {
            match self {
                Self::Prefs => store.set_prefs(serde_json::from_value(payload).unwrap()).map(Some),
                Self::Words => store.set_words(serde_json::from_value(payload).unwrap()).map(|()| None),
                Self::Snippets => store.set_snippets(serde_json::from_value(payload).unwrap()).map(|()| None),
                Self::Styles => store.set_styles(serde_json::from_value(payload).unwrap()).map(|()| None),
            }
        }
    }

    fn seeded(folder: PathBuf) -> Store {
        let mut store = Store::load_from(folder);
        store.prefs.keep_days = -1;
        store.prefs.mic = "microfone anterior".into();
        store.prefs.voice_output = "off".into();
        store.history = vec![Entry { id: "synthetic-history".into(), date: now_ms(),
            text: "histórico anterior 🐾".into(), app: Some("synthetic-app".into()), words: 3, seconds: 2.5 }];
        store.stats.days.insert("2026-10-19".into(), DayStats { words: 3, seconds: 2.5, dictations: 1 });
        store.stats.apps.insert("synthetic-app".into(), 3);
        store.words = Words { words: vec!["palavra anterior".into()], replacements: vec![Replacement {
            id: "old-replacement".into(), from: "ola".into(), to: "olá".into() }] };
        store.snippets = vec![Snippet { id: "old-snippet".into(), trigger: "bom dia".into(), text: "Bom dia, João!".into() }];
        store.last_unkept = Some("última frase não guardada 🐾".into());
        store
    }

    fn snapshot(store: &Store) -> Value {
        json!({"prefs":store.prefs,"words":store.words,"snippets":store.snippets,"styles":store.app_styles,
            "history":store.history,"stats":store.stats,"lastUnkept":store.last_unkept,"lastText":store.last_text()})
    }

    fn persist_seed(store: &Store) {
        store.save(&store.prefs, "prefs.json").unwrap();
        store.save(&store.words, "words.json").unwrap();
        store.save(&store.snippets, "snippets.json").unwrap();
        store.save(&store.app_styles, "styles.json").unwrap();
        store.save(&store.history, "history.json").unwrap();
        store.save(&store.stats, "stats.json").unwrap();
    }

    fn assert_no_temporaries(folder: &std::path::Path) {
        for entry in std::fs::read_dir(folder).unwrap() {
            assert!(!entry.unwrap().file_name().to_string_lossy().ends_with(".tmp"));
        }
    }

    fn round_trip(setting: Setting) {
        let scratch = Scratch::new();
        let mut store = seeded(scratch.0.join("Flow"));
        persist_seed(&store);
        let old_prefs = store.prefs.clone();
        let unchanged_files: BTreeMap<_, _> = ["prefs.json", "words.json", "snippets.json", "styles.json", "history.json", "stats.json"]
            .into_iter().filter(|name| *name != setting.filename())
            .map(|name| (name, std::fs::read(store.folder.join(name)).unwrap())).collect();
        let payload = setting.replacement(&store);
        let mut expected = snapshot(&store); expected[setting.key()] = payload.clone();
        let previous = setting.apply(&mut store, payload.clone()).unwrap();
        if let Setting::Prefs = setting { assert_eq!(previous.unwrap(), old_prefs); }
        else { assert!(previous.is_none()); }
        assert_eq!(snapshot(&store), expected);
        let disk: Value = serde_json::from_slice(&std::fs::read(store.folder.join(setting.filename())).unwrap()).unwrap();
        assert_eq!(disk, payload);
        for (name, bytes) in unchanged_files { assert_eq!(std::fs::read(store.folder.join(name)).unwrap(), bytes); }
        let reloaded = Store::load_from(store.folder.clone());
        expected["lastUnkept"] = Value::Null; // Only this cache is intentionally RAM-only.
        assert_eq!(snapshot(&reloaded), expected);
        assert_no_temporaries(&store.folder);
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&store.folder).unwrap().permissions().mode() & 0o777, 0o700);
            assert_eq!(std::fs::metadata(store.folder.join(setting.filename())).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[test] fn preferences_round_trip_unicode_and_preserve_other_fields() { round_trip(Setting::Prefs); }
    #[test] fn words_round_trip_unicode_and_preserve_other_fields() { round_trip(Setting::Words); }
    #[test] fn snippets_round_trip_unicode_and_preserve_other_fields() { round_trip(Setting::Snippets); }
    #[test] fn styles_round_trip_unicode_and_preserve_other_fields() { round_trip(Setting::Styles); }

    #[test]
    fn all_four_setters_reject_a_regular_file_as_folder_without_changing_ram() {
        let scratch = Scratch::new(); let blocked = scratch.0.join("blocked");
        std::fs::write(&blocked, b"synthetic folder obstruction").unwrap();
        let mut store = seeded(blocked.clone()); let before = snapshot(&store);
        for setting in Setting::all() {
            let payload = setting.replacement(&store);
            assert!(setting.apply(&mut store, payload).is_err());
            assert_eq!(snapshot(&store), before);
            assert_eq!(std::fs::read(&blocked).unwrap(), b"synthetic folder obstruction");
        }
        assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 1);
    }

    #[test]
    fn all_four_setters_reject_target_directories_and_remove_their_temporary_files() {
        for setting in Setting::all() {
            let scratch = Scratch::new(); let mut store = seeded(scratch.0.join("Flow"));
            let target = store.folder.join(setting.filename());
            std::fs::create_dir_all(&target).unwrap();
            std::fs::write(target.join("sentinel"), b"synthetic target obstruction").unwrap();
            let before = snapshot(&store); let payload = setting.replacement(&store);
            assert!(setting.apply(&mut store, payload).is_err());
            assert_eq!(snapshot(&store), before); assert!(target.is_dir());
            assert_eq!(std::fs::read(target.join("sentinel")).unwrap(), b"synthetic target obstruction");
            assert_no_temporaries(&store.folder);
            assert_eq!(std::fs::read_dir(&store.folder).unwrap().count(), 1);
        }
    }

    #[test]
    fn serialization_failure_precedes_any_directory_or_file_creation() {
        struct Refuses;
        impl Serialize for Refuses {
            fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
                Err(<S::Error as serde::ser::Error>::custom("synthetic serialization failure"))
            }
        }
        let scratch = Scratch::new(); let store = seeded(scratch.0.join("must-not-exist"));
        let before = snapshot(&store); assert!(!store.folder.exists());
        let error = store.save(&Refuses, "prefs.json").unwrap_err();
        assert!(error.to_string().contains("synthetic serialization failure"));
        assert_eq!(snapshot(&store), before); assert!(!store.folder.exists());
        assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 0);
    }

    #[test]
    fn preference_persistence_succeeds_before_the_caller_prunes_history() {
        let scratch = Scratch::new(); let mut store = seeded(scratch.0.join("Flow"));
        persist_seed(&store);
        let history = store.history.clone(); let stats = json!(store.stats);
        let old_history_bytes = std::fs::read(store.folder.join("history.json")).unwrap();
        let mut prefs = store.prefs.clone(); prefs.keep_days = 0;
        let previous = store.set_prefs(prefs.clone()).unwrap();
        assert_eq!(previous.keep_days, -1);
        assert_eq!(json!(store.history), json!(history)); assert_eq!(json!(store.stats), stats);
        assert_eq!(std::fs::read(store.folder.join("history.json")).unwrap(), old_history_bytes);
        let persisted: Prefs = load(&store.folder, "prefs.json").unwrap();
        assert_eq!(persisted, prefs);
        store.prune(now_ms());
        assert!(store.history.is_empty()); assert_eq!(json!(store.stats), stats);
        assert_eq!(store.last_text().as_deref(), Some("última frase não guardada 🐾"));
        assert_no_temporaries(&store.folder);
    }
}
