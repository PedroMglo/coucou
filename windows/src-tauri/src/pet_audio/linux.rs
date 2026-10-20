use super::types::*;
use serde_json::Value;
use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, process::Stdio, sync::OnceLock, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use tokio::{io::AsyncReadExt, process::Command, sync::Mutex};

const LIMIT: usize = 4 * 1024 * 1024;
const GONE: &str = "O dispositivo ou a aplicação mudou. Atualiza a lista e tenta novamente.";
#[derive(Default)] struct Cache { value: Option<(Instant, Snapshot)> }
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
fn cache() -> &'static Mutex<Cache> { CACHE.get_or_init(|| Mutex::new(Cache::default())) }
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }

// Bounded output and kill-on-drop apply while reading too; a hung daemon cannot
// retain a GTK thread or grow an unbounded buffer. stderr stays private.
async fn run(program: &str, args: &[String], seconds: u64) -> Result<Vec<u8>, String> {
    let operation = async {
        let mut child = Command::new(program).args(args).env("LC_ALL", "C")
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null())
            .kill_on_drop(true).spawn().map_err(|_| "O serviço de som não está disponível.".to_string())?;
        let mut bytes = Vec::new();
        child.stdout.take().ok_or("O serviço de som não respondeu.")?.take((LIMIT + 1) as u64)
            .read_to_end(&mut bytes).await.map_err(|_| "Não foi possível ler o estado do som.".to_string())?;
        if bytes.len() > LIMIT { return Err("A lista de som excedeu o limite de leitura.".into()); }
        if !child.wait().await.map_err(|_| "O serviço de som foi interrompido.".to_string())?.success() {
            return Err("O serviço de som não confirmou o pedido. Atualiza e tenta novamente.".into());
        }
        Ok(bytes)
    };
    tokio::time::timeout(Duration::from_secs(seconds), operation).await
        .map_err(|_| "O serviço de som não respondeu a tempo.".to_string())?
}
async fn pactl(args: &[String]) -> Result<Vec<u8>, String> { run("/usr/bin/pactl", args, 4).await }
async fn json(args: &[&str]) -> Result<Value, String> {
    let args = [vec!["--format=json".to_string()], args.iter().map(|s| s.to_string()).collect()].concat();
    serde_json::from_slice(&pactl(&args).await?).map_err(|_| "O serviço devolveu uma lista de som inválida.".into())
}
fn clean(value: &Value, max: usize) -> String {
    value.as_str().unwrap_or("").chars().filter(|c| !c.is_control()).take(max).collect()
}
fn property<'a>(v: &'a Value, name: &str) -> &'a Value { &v["properties"][name] }
fn serial(v: &Value) -> Option<String> {
    let s = property(v, "object.serial");
    s.as_str().filter(|s| !s.is_empty()).map(str::to_string).or_else(|| s.as_u64().map(|s| s.to_string()))
}
fn key(cookie: &str, kind: Kind, v: &Value) -> String {
    let mut hash = DefaultHasher::new();
    (cookie, format!("{kind:?}"), v["index"].as_u64(), serial(v), v["name"].as_str(), v["client"].as_u64()).hash(&mut hash);
    format!("audio-{:016x}", hash.finish())
}
fn volume(v: &Value) -> Option<f64> {
    let channels = v["volume"].as_object()?;
    let values: Option<Vec<_>> = channels.values().map(|c| c["value"].as_f64().filter(|n| n.is_finite() && *n >= 0.0).map(|n| n / 65536.0 * 100.0)).collect();
    let values = values?;
    (!values.is_empty()).then(|| values.iter().copied().fold(0.0_f64, f64::max))
}
fn choices(v: &Value, kind: Kind) -> Vec<Choice> {
    if kind == Kind::Card {
        return v["profiles"].as_object().map(|profiles| profiles.iter().take(128).map(|(id, p)| Choice {
            id: id.clone(), label: clean(&p["description"], 160), available: !matches!(p["available"].as_str(), Some("no" | "not available")) && p["available"] != false,
        }).collect()).unwrap_or_default();
    }
    v["ports"].as_array().map(|ports| ports.iter().take(64).filter_map(|p| Some(Choice {
        id: p["name"].as_str()?.to_string(), label: clean(&p["description"], 160),
        available: !matches!(p["availability"].as_str(), Some("not available" | "no")),
    })).collect()).unwrap_or_default()
}
fn nodes(cookie: &str, data: &Value, kind: Kind, default: &str) -> Result<Vec<Node>, String> {
    let values = data.as_array().ok_or("A lista de som está incompleta.")?;
    if values.len() > 512 { return Err("A lista de som excedeu o limite de dispositivos e aplicações.".into()); }
    values.iter().map(|v| {
        let index = v["index"].as_u64()?;
        let selector = if kind.stream() { index.to_string() } else { v["name"].as_str()?.to_string() };
        let app = clean(property(v, "application.name"), 96);
        let description = clean(if kind.stream() { property(v, "media.name") } else { &v["description"] }, 160);
        let label = if kind.stream() && !app.is_empty() { app } else if kind == Kind::Card { clean(property(v, "device.description"), 160) } else { description.clone() };
        let monitor = kind == Kind::Input && (selector.ends_with(".monitor") || property(v, "device.class") == "monitor");
        let active_option = v[if kind == Kind::Card { "active_profile" } else { "active_port" }].as_str().map(str::to_string);
        let volume = volume(v);
        let channels = v["volume"].as_object();
        let stereo = channels.is_some_and(|v| v.len() == 2 && v.contains_key("front-left") && v.contains_key("front-right"));
        Some(Node {
            key: key(cookie, kind, v), kind, label: if label.is_empty() { if kind.stream() { "Aplicação".into() } else { "Dispositivo de som".into() } } else { label }, description,
            volume, muted: v["mute"].as_bool(), balance: v["balance"].as_f64().filter(|v| v.is_finite() && (-1.0..=1.0).contains(v)),
            is_default: !kind.stream() && kind != Kind::Card && selector == default,
            device_key: v[if kind == Kind::Playback { "sink" } else { "source" }].as_u64().map(|i| i.to_string()), monitor,
            controllable: !cookie.is_empty() && serial(v).is_some(), can_volume: volume.is_some() && v["has_volume"] != false && v["volume_writable"] != false,
            can_balance: stereo, active: v["state"] == "RUNNING" || kind.stream() && v["corked"] == false,
            options: choices(v, kind), active_option, selector, index,
        })
    }).collect::<Option<Vec<_>>>().ok_or_else(|| "A lista de som está incompleta. Atualiza antes de alterar dispositivos.".into())
}
fn parse(info: Value, out: Value, input: Value, play: Value, rec: Value, cards: Value) -> Result<Snapshot, String> {
    let cookie = info["cookie"].as_str().unwrap_or("");
    let outputs = nodes(cookie, &out, Kind::Output, info["default_sink_name"].as_str().unwrap_or(""))?;
    let inputs = nodes(cookie, &input, Kind::Input, info["default_source_name"].as_str().unwrap_or(""))?;
    let mut playback = nodes(cookie, &play, Kind::Playback, "")?;
    let mut recording = nodes(cookie, &rec, Kind::Recording, "")?;
    for (streams, devices) in [(&mut playback, &outputs), (&mut recording, &inputs)] {
        for stream in streams { stream.device_key = devices.iter().find(|d| Some(d.index.to_string()) == stream.device_key).map(|d| d.key.clone()); }
    }
    Ok(Snapshot { available: true, observed_at: Some(now()), outputs, inputs, playback, recording, cards: nodes(cookie, &cards, Kind::Card, "")?, error: None })
}
async fn read() -> Snapshot {
    let result = async {
        let (info, out, input, play, rec, cards) = tokio::try_join!(json(&["info"]), json(&["list", "sinks"]), json(&["list", "sources"]), json(&["list", "sink-inputs"]), json(&["list", "source-outputs"]), json(&["list", "cards"]))?;
        let final_info = json(&["info"]).await?;
        if info["cookie"] != final_info["cookie"] { return Err("O serviço de som reiniciou durante a leitura. Atualiza a lista.".into()); }
        // The last default-device observation wins when another mixer changes
        // it during our read. A later native change is still allowed to win.
        parse(final_info, out, input, play, rec, cards)
    }.await;
    result.unwrap_or_else(|error| Snapshot { error: Some(error), ..Snapshot::default() })
}
pub async fn snapshot() -> Snapshot {
    let mut cache = cache().lock().await;
    if let Some((at, value)) = &cache.value { if at.elapsed() < Duration::from_millis(900) { return value.clone(); } }
    let value = read().await;
    cache.value = Some((Instant::now(), value.clone())); value
}

#[derive(Debug)] enum Operation { Pactl(Vec<String>), Input(String) }
fn plan(s: &Snapshot, r: &Request) -> Result<Operation, String> {
    if !s.available { return Err("O serviço de som não está disponível.".into()); }
    let node = s.nodes().find(|n| n.key == r.target_key && n.controllable).ok_or(GONE)?;
    let selector = node.selector.clone();
    let noun = match node.kind { Kind::Output => "sink", Kind::Input => "source", Kind::Playback => "sink-input", Kind::Recording => "source-output", Kind::Card => "card" };
    let args = match r.action {
        Action::Volume if node.kind != Kind::Card && node.can_volume => {
            let value = r.value.filter(|v| v.is_finite() && (0.0..=150.0).contains(v)).ok_or("Escolhe um volume entre 0 e 150%. Acima de 100% amplifica o som.")?;
            vec![format!("set-{noun}-volume"), selector, format!("{value:.0}%")]
        }
        Action::Mute if node.kind != Kind::Card && node.muted.is_some() => vec![format!("set-{noun}-mute"), selector, if r.muted.ok_or("Indica o estado de silêncio pretendido.")? { "1" } else { "0" }.into()],
        Action::Default if node.kind == Kind::Input && !node.monitor => return Ok(Operation::Input(selector)),
        Action::Default if node.kind == Kind::Output => vec!["set-default-sink".into(), selector],
        Action::Route if node.kind.stream() => {
            let destination = s.nodes().find(|n| Some(&n.key) == r.destination_key.as_ref() && n.controllable && n.kind == if node.kind == Kind::Playback { Kind::Output } else { Kind::Input }).ok_or(GONE)?;
            // Loopback/monitor streams keep an explicit route; never turn a
            // system recording into a microphone recording through a global move.
            if destination.monitor { return Err("Escolhe uma entrada de microfone para esta aplicação.".into()); }
            vec![format!("move-{noun}"), selector, destination.selector.clone()]
        }
        Action::Port if matches!(node.kind, Kind::Output | Kind::Input) => {
            let option = r.option.as_ref().and_then(|id| node.options.iter().find(|p| &p.id == id && p.available)).ok_or("A porta escolhida deixou de estar disponível.")?;
            vec![format!("set-{noun}-port"), selector, option.id.clone()]
        }
        Action::Profile if node.kind == Kind::Card => {
            let option = r.option.as_ref().and_then(|id| node.options.iter().find(|p| &p.id == id && p.available)).ok_or("O perfil escolhido deixou de estar disponível.")?;
            vec!["set-card-profile".into(), selector, option.id.clone()]
        }
        Action::Balance if node.can_balance && node.can_volume => {
            let value = r.value.filter(|v| v.is_finite() && (-1.0..=1.0).contains(v)).ok_or("O balanço deve ficar entre esquerda e direita.")?;
            let peak = node.volume.ok_or(GONE)?;
            // Only a true two-channel node is offered by the parser.
            vec![format!("set-{noun}-volume"), selector, format!("{:.0}%", peak * (1.0 - value.max(0.0))), format!("{:.0}%", peak * (1.0 + value.min(0.0)))]
        }
        _ => return Err("Este controlo não está disponível para o destino escolhido.".into()),
    };
    Ok(Operation::Pactl(args))
}
fn confirmed(s: &Snapshot, r: &Request) -> bool {
    let Some(n) = s.nodes().find(|n| n.key == r.target_key) else { return false; };
    if !s.available { return false; }
    match r.action {
        Action::Volume => n.volume.zip(r.value).is_some_and(|(a, b)| (a - b.round()).abs() < 1.1),
        Action::Mute => n.muted == r.muted,
        Action::Default => n.is_default,
        Action::Route => n.device_key == r.destination_key,
        Action::Port | Action::Profile => n.active_option == r.option,
        Action::Balance => n.balance.zip(r.value).is_some_and(|(a, b)| (a - b).abs() < 0.04),
    }
}
pub async fn control(request: Request) -> Result<Receipt, String> { control_for(request,None).await }
pub(super) async fn system_control(request: Request, check: std::sync::Arc<dyn Fn()->bool+Send+Sync>) -> Result<Receipt, String> { control_for(request,Some(check)).await }
async fn control_for(request: Request, system: Option<std::sync::Arc<dyn Fn()->bool+Send+Sync>>) -> Result<Receipt, String> {
    // Serialize requests from both surfaces and always reread identities before
    // mutation. IDs from a detached headset/closed app never select a new node.
    let mut cache = cache().lock().await;
    let before = read().await;
    if system.as_ref().is_some_and(|check|!super::system_policy::request_allowed(&before,&request,check.as_ref())){
        return Err("Escolhe um dispositivo de entrada, saída ou perfil nesta página. O som das aplicações pertence ao Spoty.".into());
    }
    let operation = plan(&before, &request)?;
    let mut warning = None;
    match operation {
        Operation::Pactl(args) => { pactl(&args).await?; }
        Operation::Input(name) => {
            let home = std::env::var_os("HOME").ok_or("A sessão não tem uma pasta pessoal.")?;
            let helper = std::path::PathBuf::from(home).join(".local/bin/audio-barra");
            let helper = helper.to_str().ok_or("O seletor de microfone não está disponível.")?;
            let bytes = run(helper, &[name], 30).await?;
            let result: Value = serde_json::from_slice(&bytes).map_err(|_| "A troca de microfone não foi confirmada.")?;
            if result["ok"] != true { return Err("A troca de microfone não foi confirmada.".into()); }
            warning = result["warning"].as_str().map(|s| s.chars().filter(|c| !c.is_control()).take(400).collect());
        }
    }
    let mut after = read().await;
    let mut accepted = confirmed(&after, &request);
    if !accepted {
        tokio::time::sleep(Duration::from_millis(120)).await;
        after = read().await; accepted = confirmed(&after, &request);
    }
    cache.value = Some((Instant::now(), after.clone()));
    if !accepted && warning.is_none() { warning = Some("O pedido foi enviado, mas o estado ainda não o confirma. Atualiza antes de repetir.".into()); }
    Ok(Receipt { snapshot: after, confirmed: accepted, warning })
}

pub async fn spoken(command: super::commands::SoundCommand) -> Result<(String, bool), String> {
    use super::commands::SoundCommand;
    let snapshot = snapshot().await;
    if !snapshot.available { return Err(snapshot.error.unwrap_or_else(|| "O serviço de som está indisponível.".into())); }
    if command == SoundCommand::Status {
        let output = snapshot.outputs.iter().find(|n| n.is_default).ok_or("Não há uma saída de som predefinida.")?;
        let volume = output.volume.map(|v| format!("{v:.0} por cento")).unwrap_or_else(|| "volume desconhecido".into());
        return Ok((format!("Saída {}, {}{}. O microfone {}.", output.label, volume, if output.muted == Some(true) { ", sem som" } else { "" }, snapshot.inputs.iter().find(|n| n.is_default).map(|n| match n.muted { Some(true)=>"está silenciado",Some(false)=>"não está silenciado",None=>"tem estado de silêncio desconhecido" }).unwrap_or("não está disponível")), false));
    }
    let input = match command { SoundCommand::Volume {input,..} | SoundCommand::Mute {input,..} => input, _=>false };
    let node = if input { &snapshot.inputs } else { &snapshot.outputs }.iter().find(|n| n.is_default).ok_or("Não há um dispositivo de som predefinido.")?;
    let request = match command {
        SoundCommand::Volume {percent,..} => Request { target_key: node.key.clone(), action: Action::Volume, value: Some(percent), muted: None, destination_key: None, option: None },
        SoundCommand::Mute {muted,..} => Request { target_key: node.key.clone(), action: Action::Mute, value: None, muted: Some(muted), destination_key: None, option: None },
        SoundCommand::Status => unreachable!(),
    };
    let result = control(request).await?;
    if !result.confirmed { return Ok((result.warning.unwrap_or_else(|| "O sistema ainda não confirmou a alteração do som.".into()), false)); }
    Ok((match command {
        SoundCommand::Volume {percent,..} => format!("Volume {} definido para {percent:.0} por cento.", if input { "do microfone" } else { "do sistema" }),
        SoundCommand::Mute {muted,..} => format!("{} {}.", if input { "Microfone" } else { "Som do sistema" }, if muted { "silenciado" } else { "reativado" }),
        SoundCommand::Status => unreachable!(),
    }, true))
}

#[cfg(test)] mod tests {
    use super::*;
    use serde_json::json;
    fn device(index: u64, serial: u64, name: &str) -> Value {
        json!({"index":index,"name":name,"description":"Som sintético","properties":{"object.serial":serial.to_string()},
            "mute":false,"volume":{"front-left":{"value":32768},"front-right":{"value":32768}},"balance":0.0,
            "ports":[{"name":"speaker","description":"Colunas","availability":"availability unknown"},{"name":"headphones","description":"Auscultadores","availability":"not available"}],"active_port":"speaker"})
    }
    fn fixture() -> Snapshot {
        let stream = json!({"index":8,"client":20,"sink":3,"source":5,"properties":{"object.serial":"81","application.name":"Leitor sintético","media.name":"Faixa sintética"},"mute":false,"volume":{"mono":{"value":0}},"corked":false});
        parse(json!({"cookie":"server-A","default_sink_name":"speaker","default_source_name":"mic"}),json!([device(3,31,"speaker")]),json!([device(5,51,"mic"),device(6,61,"speaker.monitor")]),json!([stream.clone()]),json!([stream]),json!([])).unwrap()
    }
    fn request(node: &Node, action: Action) -> Request { Request {target_key:node.key.clone(),action,value:None,muted:None,destination_key:None,option:None} }
    fn arguments(operation: Operation) -> Vec<String> { match operation { Operation::Pactl(v)=>v, _=>panic!("unexpected helper") } }
    #[test] fn zero_volume_is_known_and_routes_resolve_exact_device_keys() {
        let s=fixture();assert_eq!(s.playback[0].volume,Some(0.0));assert_eq!(s.outputs[0].volume,Some(50.0));
        assert_eq!(s.playback[0].device_key.as_ref(),Some(&s.outputs[0].key));assert_eq!(s.recording[0].device_key.as_ref(),Some(&s.inputs[0].key));
        assert!(s.inputs[1].monitor);assert!(s.outputs[0].is_default&&s.inputs[0].is_default);
    }
    #[test] fn serial_and_server_restart_invalidate_old_targets() {
        let s=fixture();let mut changed=device(3,32,"speaker");
        assert_ne!(key("server-A",Kind::Output,&changed),s.outputs[0].key);
        changed["properties"]["object.serial"]=json!("31");assert_ne!(key("server-B",Kind::Output,&changed),s.outputs[0].key);
        let mut r=request(&s.outputs[0],Action::Mute);r.muted=Some(true);r.target_key="audio-gone".into();assert!(plan(&s,&r).is_err());
    }
    #[test] fn missing_serial_is_read_only_and_missing_volume_stays_unknown() {
        let mut v=device(3,31,"speaker");v["properties"]=json!({});v["volume"]=Value::Null;
        let s=nodes("server-A",&json!([v]),Kind::Output,"speaker").unwrap();assert!(!s[0].controllable);assert_eq!(s[0].volume,None);
    }
    #[test] fn stereo_balance_is_not_offered_for_surround() {
        let mut v=device(3,31,"speaker");v["volume"]["rear-left"]=json!({"value":32768});
        assert!(!nodes("server-A",&json!([v]),Kind::Output,"").unwrap()[0].can_balance);
        let s=fixture();let mut r=request(&s.outputs[0],Action::Balance);r.value=Some(-0.5);
        assert_eq!(arguments(plan(&s,&r).unwrap()),["set-sink-volume","speaker","50%","25%"]);
    }
    #[test] fn volume_limits_reject_invalid_numbers_and_do_not_change_mute() {
        let s=fixture();let mut r=request(&s.outputs[0],Action::Volume);
        for v in [-1.0,151.0,f64::NAN,f64::INFINITY] {r.value=Some(v);assert!(plan(&s,&r).is_err());}
        r.value=Some(75.0);assert_eq!(arguments(plan(&s,&r).unwrap()),["set-sink-volume","speaker","75%"]);
    }
    #[test] fn routing_uses_fresh_native_selector_and_rejects_wrong_kind_or_monitor() {
        let s=fixture();let mut r=request(&s.playback[0],Action::Route);r.destination_key=Some(s.outputs[0].key.clone());
        assert_eq!(arguments(plan(&s,&r).unwrap()),["move-sink-input","8","speaker"]);
        r.destination_key=Some(s.inputs[0].key.clone());assert!(plan(&s,&r).is_err());
        r=request(&s.recording[0],Action::Route);r.destination_key=Some(s.inputs[1].key.clone());assert!(plan(&s,&r).is_err());
    }
    #[test] fn input_default_always_uses_existing_bluetooth_preservation_helper() {
        let s=fixture();assert!(matches!(plan(&s,&request(&s.inputs[0],Action::Default)),Ok(Operation::Input(name)) if name=="mic"));
        assert!(plan(&s,&request(&s.inputs[1],Action::Default)).is_err());
    }
    #[test] fn unavailable_or_arbitrary_ports_never_become_arguments() {
        let s=fixture();let mut r=request(&s.outputs[0],Action::Port);
        for name in ["headphones","; touch /tmp/forbidden","unknown"] {r.option=Some(name.into());assert!(plan(&s,&r).is_err());}
        r.option=Some("speaker".into());assert_eq!(arguments(plan(&s,&r).unwrap()),["set-sink-port","speaker","speaker"]);
    }
    #[test] fn profile_options_handle_pulse_and_pipewire_availability_shapes() {
        let v=json!({"profiles":{"yes":{"description":"A2DP","available":true},"unknown":{"description":"HFP","available":"unknown"},"no":{"available":"no"},"false":{"available":false}}});
        let choices=choices(&v,Kind::Card);for c in choices {assert_eq!(c.available,matches!(c.id.as_str(),"yes"|"unknown"));}
    }
    #[test] fn receipt_requires_readback_even_when_native_process_accepted() {
        let s=fixture();let mut r=request(&s.outputs[0],Action::Volume);r.value=Some(75.0);assert!(!confirmed(&s,&r));
        let mut after=s.clone();after.outputs[0].volume=Some(75.0);assert!(confirmed(&after,&r));after.available=false;assert!(!confirmed(&after,&r));
        after.available=true;after.outputs[0].key="replacement".into();assert!(!confirmed(&after,&r));
    }
    #[test] fn mute_is_an_explicit_state_and_card_cannot_be_a_stream() {
        let s=fixture();let mut r=request(&s.playback[0],Action::Mute);assert!(plan(&s,&r).is_err());r.muted=Some(true);
        assert_eq!(arguments(plan(&s,&r).unwrap()),["set-sink-input-mute","8","1"]);
        assert!(plan(&s,&request(&s.outputs[0],Action::Route)).is_err());
    }
    #[test] fn malformed_or_truncated_inventory_is_not_an_empty_success() {
        assert!(nodes("server-A",&Value::Null,Kind::Output,"").is_err());
        assert!(parse(json!({}),json!([]),Value::Null,json!([]),json!([]),json!([])).is_err());
        assert!(nodes("server-A",&json!(vec![device(3,31,"speaker");513]),Kind::Output,"").is_err());
    }
    #[test] fn request_rejects_unknown_fields_and_actions() {
        assert!(serde_json::from_value::<Request>(json!({"targetKey":"x","action":"shell","program":"/bin/sh"})).is_err());
        assert!(serde_json::from_value::<Request>(json!({"targetKey":"x","action":"mute","muted":true,"selector":"speaker"})).is_err());
    }
    #[test] fn private_native_names_and_indices_are_not_sent_to_webview() {
        let s=fixture();let value=serde_json::to_value(s).unwrap();assert!(value["outputs"][0].get("selector").is_none());assert!(value["outputs"][0].get("index").is_none());
    }
}
