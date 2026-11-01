//! Bounded, cancellable synthesis at the explicitly selected service.
//! No retries, redirects, engine substitution or renderer-selected endpoint.
use super::tts::Current;
const VOICES_BYTES:usize=256*1024;
const AUDIO_BYTES:usize=8*1024*1024;

pub(super) fn elevenlabs(text:&str,gender:&str,current:Current)->Option<Vec<u8>>{
    if !current()||text.trim().is_empty()||text.chars().count()>8000{return None;}
    let key=crate::secrets::get("elevenlabs-api-key")?;
    let base=reqwest::Url::parse("https://api.elevenlabs.io/").ok()?;
    let client=reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10)).build().ok()?;
    tauri::async_runtime::block_on(crate::net::while_current(||current(),
        ||request(&client,&base,text,gender,&key,&current))).ok()
}

async fn request(client:&reqwest::Client,base:&reqwest::Url,text:&str,gender:&str,key:&str,current:&Current)->Result<Vec<u8>,String>{
    let check=||if current(){Ok(())}else{Err(crate::net::CANCELLED.to_string())};
    check()?;
    let response=client.get(base.join("v1/voices").map_err(|_|"invalid speech destination")?)
        .header("xi-api-key",key).send().await.map_err(|_|"voice discovery failed")?;
    if !response.status().is_success(){return Err("voice discovery refused".into());}
    let bytes=crate::net::read_capped(response,VOICES_BYTES).await?;check()?;
    let body:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|"invalid voice discovery")?;
    let voices=body["voices"].as_array().filter(|v|v.len()<=256).ok_or("invalid voice inventory")?;
    // Gender is an explicit choice; absence cannot select another gender.
    let eligible:Vec<_>=voices.iter().filter(|v|v["labels"]["gender"].as_str().is_some_and(|g|g.eq_ignore_ascii_case(gender))).cloned().collect();
    let voice=super::pick_voice(&eligible,gender).ok_or("selected voice unavailable")?;
    if voice.is_empty()||voice.len()>128||!voice.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_'){return Err("invalid voice destination".into());}
    let destination=base.join(&format!("v1/text-to-speech/{voice}?output_format=mp3_44100_128")).map_err(|_|"invalid speech destination")?;
    let request=client.post(destination).header("xi-api-key",key).header("accept","audio/mpeg")
        .json(&serde_json::json!({"text":text,"model_id":"eleven_flash_v2_5"})).build().map_err(|_|"invalid speech request")?;
    check()?; // Metadata wait never carries authority into a paid POST.
    let response=client.execute(request).await.map_err(|_|"speech delivery unconfirmed")?;
    if !response.status().is_success(){return Err("speech synthesis refused".into());}
    let audio=crate::net::read_capped(response,AUDIO_BYTES).await?;check()?;
    if audio.is_empty(){return Err("empty speech result".into());}Ok(audio)
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::{io::{Read,Write},net::{TcpListener,TcpStream},sync::{Arc,atomic::{AtomicBool,Ordering}},time::Duration};
    fn read(stream:&mut TcpStream)->String{
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut data=Vec::new();let mut byte=[0];
        while !data.ends_with(b"\r\n\r\n"){if stream.read(&mut byte).unwrap()==0{break;}data.push(byte[0]);assert!(data.len()<16384);}
        let head=String::from_utf8(data.clone()).unwrap();let size=head.lines().find_map(|s|s.to_lowercase().strip_prefix("content-length:").and_then(|v|v.trim().parse::<usize>().ok())).unwrap_or(0);
        assert!(size<32768);let mut body=vec![0;size];stream.read_exact(&mut body).unwrap();data.extend(body);String::from_utf8(data).unwrap()
    }
    fn reply(stream:&mut TcpStream,status:&str,body:&[u8]){write!(stream,"HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();stream.write_all(body).unwrap();}
    fn client()->reqwest::Client{reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(2)).build().unwrap()}
    fn fixture_url(listener:&TcpListener)->reqwest::Url{reqwest::Url::parse(&format!("http://{}/",listener.local_addr().unwrap())).unwrap()}

    #[tokio::test]
    async fn selected_service_receives_exact_text_model_and_voice_without_fallback(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let base=fixture_url(&listener);
        let worker=std::thread::spawn(move||{
            let(mut first,_)=listener.accept().unwrap();assert!(read(&mut first).starts_with("GET /v1/voices "));
            reply(&mut first,"200 OK",br#"{"voices":[{"voice_id":"selected-voice","labels":{"gender":"female","language":"en"}}]}"#);drop(first);
            let(mut second,_)=listener.accept().unwrap();let request=read(&mut second);assert!(request.starts_with("POST /v1/text-to-speech/selected-voice?output_format=mp3_44100_128 "));
            let body:serde_json::Value=serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();assert_eq!(body,serde_json::json!({"text":"fixture text","model_id":"eleven_flash_v2_5"}));reply(&mut second,"200 OK",b"synthetic audio");
        });
        let current:Current=Arc::new(||true);let client=client();let out=crate::net::while_current(||current(),||request(&client,&base,"fixture text","female","fixture-key",&current)).await.unwrap();assert_eq!(out,b"synthetic audio");worker.join().unwrap();
    }

    #[tokio::test]
    async fn revocation_during_discovery_prevents_synthesis_post(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let base=fixture_url(&listener);let alive=Arc::new(AtomicBool::new(true));let native=alive.clone();
        let worker=std::thread::spawn(move||{
            let(mut stream,_)=listener.accept().unwrap();read(&mut stream);native.store(false,Ordering::Release);
            reply(&mut stream,"200 OK",br#"{"voices":[{"voice_id":"original","labels":{"gender":"female"}}]}"#);drop(stream);listener.set_nonblocking(true).unwrap();std::thread::sleep(Duration::from_millis(30));assert!(listener.accept().is_err(),"POST after revocation");
        });
        let current:Current=Arc::new(move||alive.load(Ordering::Acquire));assert!(request(&client(),&base,"fixture","female","fixture-key",&current).await.is_err());worker.join().unwrap();
    }

    #[tokio::test]
    async fn redirect_oversize_or_changed_voice_target_never_reaches_post(){
        for response in ["HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/other\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",VOICES_BYTES+1),String::new()]{
            let listener=TcpListener::bind("127.0.0.1:0").unwrap();let base=fixture_url(&listener);
            let worker=std::thread::spawn(move||{
                let(mut stream,_)=listener.accept().unwrap();read(&mut stream);
                if response.is_empty(){reply(&mut stream,"200 OK",br#"{"voices":[{"voice_id":"../../changed","labels":{"gender":"female"}}]}"#);}else{stream.write_all(response.as_bytes()).unwrap();}
                drop(stream);listener.set_nonblocking(true).unwrap();std::thread::sleep(Duration::from_millis(30));assert!(listener.accept().is_err());
            });
            let current:Current=Arc::new(||true);assert!(request(&client(),&base,"fixture","female","fixture-key",&current).await.is_err());worker.join().unwrap();
        }
    }
}
