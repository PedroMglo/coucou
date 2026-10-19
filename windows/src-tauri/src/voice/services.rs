//! Deterministic owner controls: quick playback commands need no model turn.
use tauri::AppHandle;
use super::brain::Reply;
#[derive(Debug,PartialEq)]
enum SpotifyAction {Open,Play,Pause,Toggle,Next,Previous,Volume(f64),Search(String),Item(String)}
fn parse(query:&str)->Option<SpotifyAction> {
 let text=super::wake::normalize(query);
 let fixed=match text.as_str() {
  "abre"|"abrir"|"inicia"|"iniciar"|"open"|"start"|"launch"|"abre o spotify"|"open spotify"=>Some(SpotifyAction::Open),
  "toca"|"continua"|"retoma"|"reproduz"|"play"|"resume"|"play music"=>Some(SpotifyAction::Play),
  "pausa"|"pausar"|"para a musica"|"parar a musica"|"pause"|"pause music"|"stop music"=>Some(SpotifyAction::Pause),
  "toca ou pausa"|"play pause"=>Some(SpotifyAction::Toggle),
  "seguinte"|"proxima"|"proxima musica"|"musica seguinte"|"avanca"|"avanca a musica"|"next"|"next track"|"skip"=>Some(SpotifyAction::Next),
  "anterior"|"musica anterior"|"volta atras"|"previous"|"previous track"=>Some(SpotifyAction::Previous),
  _=>None,
 };if fixed.is_some(){return fixed;}
 if text.starts_with("volume ") {
  let values=text.split_whitespace().filter_map(|s|s.parse::<f64>().ok()).collect::<Vec<_>>();
  if values.len()==1 && (0.0..=100.0).contains(&values[0]){return Some(SpotifyAction::Volume(values[0]));}
 }
 let words=query.split_whitespace().collect::<Vec<_>>();
 for prefix in ["pesquisa","procura","search","find","toca","reproduz","play"] {
  if text.starts_with(&format!("{prefix} ")) {
   let term=words[1..].join(" ");
   if term.starts_with("spotify:") || term.starts_with("https://open.spotify.com/") {return Some(SpotifyAction::Item(term));}
   if ["toca","reproduz","play"].contains(&prefix){return None;}
   return Some(SpotifyAction::Search(term));
  }
 }
 None
}
fn uri(text:&str)->Option<String> {
 let value=if text.starts_with("spotify:"){text.to_string()}else{
  let url=reqwest::Url::parse(text).ok()?;
  if url.scheme()!="https" || url.host_str()!=Some("open.spotify.com") || !url.username().is_empty() || url.password().is_some() || url.port().is_some(){return None;}
  let pieces=url.path_segments()?.filter(|p|!p.is_empty()).collect::<Vec<_>>();
  let p=if pieces.first()?.starts_with("intl-"){&pieces[1..]}else{&pieces};
  if p.len()!=2 {return None;}format!("spotify:{}:{}",p[0],p[1])
 };
 let p=value.split(':').collect::<Vec<_>>();
 (p.len()==3 && p[0]=="spotify" && ["track","album","playlist","episode"].contains(&p[1]) && p[2].len()==22 && p[2].bytes().all(|b|b.is_ascii_alphanumeric())).then_some(value)
}
pub fn spotify(app:&AppHandle,query:&str)->Option<Reply> {
 let action=parse(query)?;
 if let Err(text)=crate::pet_config::require("integration_spotify","music"){return Some(Reply{text,acted:false,language:Some("pt-PT".into())});}
 let (text,acted)=match action {
  SpotifyAction::Open=>{let ok=tauri::async_runtime::block_on(crate::spotify::spotify_open());(if ok{"Pedi ao Spotify para abrir."}else{"Não consegui abrir o Spotify."}.into(),ok)},
  SpotifyAction::Search(term)=>{
   let opened=tauri::async_runtime::block_on(crate::spotify::spotify_open());
   if opened {
    let encoded=reqwest::Url::parse("https://open.spotify.com").ok().and_then(|mut url|{url.path_segments_mut().ok()?.push("search").push(&term);Some(url.path().trim_start_matches("/search/").to_string())})?;
    crate::platform::open_url(&format!("spotify:search:{encoded}"));
    (format!("Pedi ao Spotify para pesquisar {term}. Podes escolher a faixa na aplicação ou pedir-me para a selecionar."),true)
   }else{("Não consegui abrir a pesquisa do Spotify.".into(),false)}
  },
  SpotifyAction::Item(item)=>{
   let item=uri(&item);let ok=item.is_some_and(|id|tauri::async_runtime::block_on(crate::spotify::spotify_play_item(app.clone(),id)));
   (if ok{"O Spotify aceitou o pedido de reprodução."}else{"Não consegui reproduzir esse item do Spotify."}.into(),ok)
  },
  control=>{
   let (command,value)=match control {SpotifyAction::Play=>("play",None),SpotifyAction::Pause=>("pause",None),SpotifyAction::Toggle=>("playPause",None),SpotifyAction::Next=>("next",None),SpotifyAction::Previous=>("previous",None),SpotifyAction::Volume(v)=>("volume",Some(v)),_=>unreachable!()};
   let before=tauri::async_runtime::block_on(crate::spotify::spotify_refresh(app.clone()));
   if command=="pause" && before.as_ref().is_some_and(|s|!s.running || !s.playing){("O Spotify já está em pausa.".into(),false)}else{
    let ok=tauri::async_runtime::block_on(crate::spotify::spotify_control(app.clone(),command.into(),value));
    let after=if ok{tauri::async_runtime::block_on(crate::spotify::spotify_refresh(app.clone()))}else{None};
    let verified=match command {"pause"=>after.as_ref().is_some_and(|s|!s.playing),"play"=>after.as_ref().is_some_and(|s|s.playing),"volume"=>after.as_ref().is_some_and(|s|(s.volume as f64-value.unwrap()).abs()<2.0),_=>ok};
    let response=if !ok {"O Spotify não aceitou esse controlo.".into()}else if verified {match command {"pause"=>"O Spotify ficou em pausa.".into(),"play"=>"O Spotify está a reproduzir.".into(),"volume"=>format!("Volume do Spotify definido para {} por cento.",value.unwrap()),"next"=>"Pedi ao Spotify a faixa seguinte.".into(),"previous"=>"Pedi ao Spotify a faixa anterior.".into(),_=>"O Spotify aceitou o controlo.".into()}}else{"O Spotify aceitou o comando, mas ainda não consegui confirmar o estado.".into()};
    (response,ok)
   }
  }
 };
 Some(Reply{text,acted,language:Some("pt-PT".into())})
}
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn portuguese_and_english_owner_controls_are_specific_and_do_not_toggle_a_pause(){
  for input in ["pausa","pause","pausar","para a música"]{assert_eq!(parse(input),Some(SpotifyAction::Pause));}
  for input in ["próxima música","next track","avança"]{assert_eq!(parse(input),Some(SpotifyAction::Next));}
  assert_eq!(parse("volume para 30 por cento"),Some(SpotifyAction::Volume(30.0)));assert_eq!(parse("volume 150"),None);
  assert_eq!(parse("pesquisa Radiohead"),Some(SpotifyAction::Search("Radiohead".into())));
  assert_eq!(parse("explica como o Spotify funciona"),None,"free-form requests use the owner's own assistant conversation");
 }
 #[test]fn only_spotify_items_can_be_selected_for_playback(){
  assert_eq!(uri("https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC?si=test"),Some("spotify:track:4uLU6hMCjMI75M1A2tKUQC".into()));
  for text in ["https://example.com/track/4uLU6hMCjMI75M1A2tKUQC","spotify:search:example","spotify:track:bad","file:///tmp/test"]{assert!(uri(text).is_none());}
 }
}
