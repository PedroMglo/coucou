//! Spoken routing and inactivity policy. No microphone, network or desktop.
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pet {
    Codex,
    Copilot,
    Gravity,
    Claude,
    Gemini,
    Ollama,
    Spotify,
    GitHub,
    Notion,
    Vercel,
    N8n,
    Resend,
    Stripe,
    Calcom,
}

impl Pet {
    pub const ALL: [Self; 14] = [Self::Codex, Self::Copilot, Self::Gravity, Self::Claude, Self::Gemini, Self::Ollama,Self::Spotify,Self::GitHub,Self::Notion,Self::Vercel,Self::N8n,Self::Resend,Self::Stripe,Self::Calcom];
    pub fn from_provider(provider: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.provider() == provider)
    }
    pub fn provider(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Gravity => "antigravity",
            Self::Claude => "claude-cli",
            Self::Gemini => "gemini-cli",
            Self::Ollama => "ollama",
            Self::Spotify => "integration_spotify",
            Self::GitHub => "integration_github",
            Self::Notion => "integration_notion",
            Self::Vercel => "integration_vercel",
            Self::N8n => "integration_n8n",
            Self::Resend => "integration_resend",
            Self::Stripe => "integration_stripe",
            Self::Calcom => "integration_calcom",

        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Copilot => "Copilot",
            Self::Gravity => "Gravity",
            Self::Claude => "Claude",
            Self::Gemini => "Gemini",
            Self::Ollama => "Ollama",
            Self::Spotify => "Spotify",
            Self::GitHub => "GitHub",
            Self::Notion => "Notion",
            Self::Vercel => "Vercel",
            Self::N8n => "n8n",
            Self::Resend => "Resend",
            Self::Stripe => "Stripe",
            Self::Calcom => "Cal.com",

        }
    }
    pub fn pill(self) -> &'static str {
        match self {
            Self::Codex => "agent_codex",
            Self::Copilot => "agent_copilot",
            Self::Gravity => "agent_antigravity",
            Self::Claude => "integration_claude",
            Self::Gemini => "agent_gemini",
            Self::Ollama => "ai_ollama",
            Self::Spotify => "integration_spotify",
            Self::GitHub => "integration_github",
            Self::Notion => "integration_notion",
            Self::Vercel => "integration_vercel",
            Self::N8n => "integration_n8n",
            Self::Resend => "integration_resend",
            Self::Stripe => "integration_stripe",
            Self::Calcom => "integration_calcom",

        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Select(Pet, String),
    Stop,
    CancelTask,
    TaskStatus,
    Dictate,
    Message(String),
}

#[cfg(test)]
mod task_control_tests {
    use super::*;
    #[test]
    fn explicit_task_stop_and_audio_off_are_distinct_even_when_addressing_a_pet() {
        for text in ["interrompe a tarefa","cancela a tarefa","pára a tarefa","cancel task"] {assert_eq!(parse(text),Command::CancelTask);}
        for text in ["desligar microfone","parar conversa"] {assert_eq!(parse(text),Command::Stop);}
        let Command::Select(Pet::Codex,question)=parse("Codex, interrompe a tarefa") else{panic!("pet address lost")};
        assert_eq!(parse(&question),Command::CancelTask);
        for text in ["estado da tarefa","como está a tarefa","task status"] {assert_eq!(parse(text),Command::TaskStatus);}
        assert_eq!(parse("parar o servidor"),Command::Message("parar o servidor".into()));
    }
}

pub fn parse(text: &str) -> Command {
    let normalized = super::wake::normalize(text);
    if matches!(
        normalized.as_str(),
        "parar conversa"
            | "terminar conversa"
            | "desligar microfone"
            | "stop conversation"
            | "stop listening"
    ) {
        return Command::Stop;
    }
    if matches!(normalized.as_str(),"interromper tarefa"|"interromper a tarefa"|"interrompe a tarefa"|"interrompe tarefa"|"cancelar tarefa"|"cancelar a tarefa"|"cancela a tarefa"|"cancela tarefa"|"para a tarefa"|"parar a tarefa"|"stop task"|"cancel task") {
        return Command::CancelTask;
    }
    if matches!(normalized.as_str(),"estado da tarefa"|"como vai a tarefa"|"como esta a tarefa"|"estado do trabalho"|"task status") {return Command::TaskStatus;}
    if matches!(normalized.as_str(), "ditado" | "ditar" | "dictation") {
        return Command::Dictate;
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    let norm: Vec<String> = words.iter().map(|w| super::wake::normalize(w)).collect();
    let mut start = 0;
    // Only an address at the beginning selects a pet. Mentioning Codex inside
    // a question must not switch provider or lose the start of the question.
    if norm
        .first()
        .is_some_and(|w| matches!(w.as_str(), "ola" | "hey" | "ok"))
    {
        start = 1;
    }
    let prefixes: &[&[&str]] = &[
        &["chama", "o"],
        &["chama"],
        &["muda", "para"],
        &["troca", "para"],
        &["falar", "com"],
    ];
    for prefix in prefixes {
        if norm[start..]
            .iter()
            .map(String::as_str)
            .take(prefix.len())
            .eq(prefix.iter().copied())
        {
            start += prefix.len();
            break;
        }
    }
    let Some(first) = norm.get(start).map(String::as_str) else {
        return Command::Message(text.trim().into());
    };
    let (pet, count) = match (first, norm.get(start + 1).map(String::as_str)) {
        ("codex", _) => (Pet::Codex, 1),
        ("copilot", Some("cli")) => (Pet::Copilot,2),
        ("copilot"|"copiloto", _) => (Pet::Copilot,1),
        ("gravity" | "antigravity" | "antigravidade", _) => (Pet::Gravity, 1),
        ("anti", Some("gravity" | "gravidade")) => (Pet::Gravity, 2),
        ("claude", _) => (Pet::Claude, 1),
        ("gemini", _) => (Pet::Gemini, 1),
        ("ollama" | "olama", _) => (Pet::Ollama, 1),
        ("spotify" | "spoty", _) => (Pet::Spotify,1),
        ("github", _) => (Pet::GitHub,1),
        ("notion", _) => (Pet::Notion,1),
        ("vercel", _) => (Pet::Vercel,1),
        ("n8n", _) => (Pet::N8n,1),
        ("resend", _) => (Pet::Resend,1),
        ("stripe", _) => (Pet::Stripe,1),
        ("calcom", _) => (Pet::Calcom,1),

        _ => return Command::Message(text.trim().into()),
    };
    let question = words[start + count..]
        .join(" ")
        .trim_start_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .to_string();
    Command::Select(pet, question)
}

/// Learned spellings are matched at word boundaries, longest first. A spelling
/// shared by two pets is ambiguous and never routes a request automatically.
pub fn parse_with_aliases(
    text: &str,
    aliases: &std::collections::BTreeMap<String, Vec<String>>,
) -> Command {
    let builtin = parse(text);
    if !matches!(builtin, Command::Message(_)) {
        return builtin;
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    for count in (1..=words.len()).rev() {
        let prefix = super::wake::normalize(&words[..count].join(" "));
        let matches: Vec<Pet> = Pet::ALL
            .into_iter()
            .filter(|pet| {
                aliases.get(pet.provider()).is_some_and(|names| {
                    names
                        .iter()
                        .any(|name| !prefix.is_empty() && super::wake::normalize(name) == prefix)
                })
            })
            .collect();
        if matches.len() == 1 {
            return Command::Select(matches[0], words[count..].join(" "));
        }
        if matches.len() > 1 {
            return builtin;
        }
    }
    builtin
}

/// Only time waiting for the user counts. Model execution and speech may take
/// longer than this interval; the next listening window starts after the reply.
pub struct Idle {
    touched: Instant,
    limit: Duration,
}
impl Idle {
    pub fn new(now: Instant, seconds: u64) -> Self {
        Self {
            touched: now,
            limit: Duration::from_secs(seconds.clamp(15, 600)),
        }
    }
    pub fn touch(&mut self, now: Instant) {
        self.touched = now;
    }
    pub fn expired(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.touched) >= self.limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_service_name_addresses_its_own_conversation_instead_of_the_codex_pet() {
        for pet in Pet::ALL {assert_eq!(Pet::from_provider(pet.provider()),Some(pet));}
        for (spoken,pet) in [("Spotify, pause",Pet::Spotify),("GitHub, verifica o CI",Pet::GitHub),("Notion, abre a app",Pet::Notion)] {
            let Command::Select(selected,_) = parse(spoken) else {panic!("service was not addressed")};assert_eq!(selected,pet);assert_ne!(selected.provider(),"codex");
        }
    }

    #[test]
    fn names_switch_and_preserve_the_question() {
        assert_eq!(parse("Codex."), Command::Select(Pet::Codex, String::new()));
        assert_eq!(
            parse("Olá Codex, abre o Firefox."),
            Command::Select(Pet::Codex, "abre o Firefox.".into())
        );
        assert_eq!(
            parse("chama o Gravity"),
            Command::Select(Pet::Gravity, String::new())
        );
        assert_eq!(
            parse("Anti Gravity, vê esta pasta"),
            Command::Select(Pet::Gravity, "vê esta pasta".into())
        );
        assert_eq!(
            parse("Gemini conta uma história"),
            Command::Select(Pet::Gemini, "conta uma história".into())
        );
    }
    #[test]
    fn ordinary_mentions_and_followups_are_messages() {
        for text in [
            "explica o Codex",
            "e depois?",
            "gravityfield",
            "troca para",
            "",
        ] {
            assert_eq!(parse(text), Command::Message(text.into()));
        }
        assert_eq!(parse("desligar microfone"), Command::Stop);
        assert_eq!(
            parse("parar o servidor"),
            Command::Message("parar o servidor".into())
        );
        assert_eq!(parse("ditado"), Command::Dictate);
    }
    #[test]
    fn inactivity_resets_after_speech_and_replies_and_is_bounded() {
        let now = Instant::now();
        let mut idle = Idle::new(now, 60);
        assert!(!idle.expired(now + Duration::from_secs(59)));
        assert!(idle.expired(now + Duration::from_secs(60)));
        idle.touch(now + Duration::from_secs(200)); // slow model finished
        assert!(!idle.expired(now + Duration::from_secs(259)));
        assert!(idle.expired(now + Duration::from_secs(260)));
        assert!(!Idle::new(now, 0).expired(now + Duration::from_secs(14)));
        assert!(Idle::new(now, u64::MAX).expired(now + Duration::from_secs(600)));
    }
    #[test]
    fn learned_pronunciations_are_local_and_unambiguous() {
        let mut aliases = std::collections::BTreeMap::from([(
            "codex".into(),
            vec!["códex".into(), "co dex".into()],
        )]);
        assert_eq!(
            parse_with_aliases("Co dex abre a pasta", &aliases),
            Command::Select(Pet::Codex, "abre a pasta".into())
        );
        assert_eq!(
            parse_with_aliases("co dexter", &aliases),
            Command::Message("co dexter".into())
        );
        aliases.insert("antigravity".into(), vec!["co dex".into()]);
        assert_eq!(
            parse_with_aliases("co dex", &aliases),
            Command::Message("co dex".into())
        );
    }
}
