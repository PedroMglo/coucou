//! Small PT/EN grammar for sound controls; never a general assistant prompt.
#[derive(Debug, PartialEq)]
pub enum SoundCommand { Volume { input: bool, percent: f64 }, Mute { input: bool, muted: bool }, Status }
pub fn parse(text: &str) -> Option<SoundCommand> {
    use SoundCommand::*;
    let fixed = match text {
        "silenciar sistema" | "silenciar o sistema" | "silenciar pc" | "mute system" | "mute pc" => Some(Mute { input: false, muted: true }),
        "repor som do sistema" | "ativar som do sistema" | "unmute system" | "unmute pc" => Some(Mute { input: false, muted: false }),
        "silenciar microfone" | "silenciar o microfone" | "mute microphone" | "mute mic" => Some(Mute { input: true, muted: true }),
        "reativar microfone" | "reativar o microfone" | "unmute microphone" | "unmute mic" => Some(Mute { input: true, muted: false }),
        "estado do som" | "estado do audio" | "audio status" | "sound status" => Some(Status),
        _ => None,
    };
    if fixed.is_some() { return fixed; }
    for (prefix, input) in [("volume do sistema ", false), ("volume do pc ", false), ("system volume ", false), ("pc volume ", false), ("volume do microfone ", true), ("microphone volume ", true), ("mic volume ", true)] {
        if let Some(tail) = text.strip_prefix(prefix) {
            let words: Vec<_> = tail.split_whitespace().collect();
            let numbers: Vec<_> = words.iter().filter_map(|s| s.trim_end_matches('%').parse::<f64>().ok()).collect();
            let allowed = words.iter().all(|s| ["para", "por", "cento", "percent", "to", "at"].contains(s) || s.trim_end_matches('%').parse::<f64>().is_ok());
            if allowed && numbers.len() == 1 && numbers[0].is_finite() && (0.0..=100.0).contains(&numbers[0]) {
                return Some(Volume { input, percent: numbers[0] });
            }
        }
    }
    None
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn accepts_exact_pt_en_sound_commands_only() {
        assert_eq!(parse("volume do sistema para 40 por cento"), Some(SoundCommand::Volume {input:false,percent:40.0}));
        assert_eq!(parse("microphone volume 25%"), Some(SoundCommand::Volume {input:true,percent:25.0}));
        assert_eq!(parse("silenciar microfone"), Some(SoundCommand::Mute {input:true,muted:true}));
        assert_eq!(parse("unmute system"), Some(SoundCommand::Mute {input:false,muted:false}));
        for query in ["volume do sistema 40 e apagar ficheiros", "system volume 200", "volume 40", "explica o som", "system volume NaN"] { assert_eq!(parse(query), None, "{query}"); }
    }
}
