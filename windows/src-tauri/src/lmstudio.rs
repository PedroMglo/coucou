//! Native, read-only model identity. Metadata is not an execution permit.
use crate::{chat::ModelInfo,local_chat::Server,net};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{collections::BTreeSet,time::Duration};
const ERROR:&str="Os metadados nativos do LM Studio não confirmam o modelo escolhido.";
const MAX_MODELS:usize=256;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Binding{
    pub schema_version:u32,pub endpoint:String,pub model_key:String,
    pub instance_id:Option<String>,pub model_hash:String,
}
fn id(s:&str)->bool{crate::pet_capabilities::valid_model_id(s)}
fn digest(value:&impl Serialize)->Result<String,String>{
    let mut data=b"coucou-lmstudio-native-binding-v1\0".to_vec();data.extend(serde_json::to_vec(value).map_err(|_|ERROR)?);
    Ok(ring::digest::digest(&ring::digest::SHA256,&data).as_ref().iter().map(|b|format!("{b:02x}")).collect())
}
pub(crate)fn endpoint_key(raw:&str,generation:u64)->Option<String>{let url=net::normalise_server_url(raw).ok()?;digest(&(url.as_str().trim_end_matches('/'),generation)).ok()}
impl Binding{
    pub(crate)fn valid_for(&self,model:&str)->bool{
        self.schema_version==1&&id(&self.model_key)&&id(model)
            &&self.instance_id.as_ref().map_or(self.model_key==model,|s|id(s)&&s==model)
            &&self.model_hash.len()==64&&self.model_hash.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'a'..=b'f'))
            &&net::normalise_server_url(&self.endpoint).is_ok_and(|url|net::is_loopback_url(&url)&&url.as_str().trim_end_matches('/')==self.endpoint)
    }
}
#[derive(Clone)]struct Instance{id:String,config:Value}
#[derive(Clone)]struct Model{key:String,label:String,kind:String,identity:Value,instances:Vec<Instance>}
#[derive(Clone)]pub(crate)struct Catalog{endpoint:String,auth:String,models:Vec<Model>}
impl Catalog{
    pub(crate)fn parse(server:&Server,value:&Value)->Result<Self,String>{
        if server.id!="lmstudio"{return Err(ERROR.into());}
        let base=crate::local_chat::base_url(server)?;if !net::is_loopback_url(&base){return Err("Este pet LM Studio requer o servidor local explicitamente configurado.".into());}
        let raw=value.get("models").and_then(Value::as_array).filter(|v|v.len()<=MAX_MODELS).ok_or(ERROR)?;
        let mut models=vec![];let mut keys=BTreeSet::new();
        for row in raw{
            if row.get("remote").is_some_and(|v|v!=&Value::Bool(false)&&!v.is_null())||["remote_host","remote_model"].iter().any(|k|row.get(k).is_some_and(|v|!v.is_null()&&v.as_str().is_none_or(|s|!s.trim().is_empty()))){return Err(ERROR.into());}
            let key=row.get("key").and_then(Value::as_str).filter(|s|id(s)).ok_or(ERROR)?.to_string();
            if !keys.insert(key.clone()){return Err(ERROR.into());}
            let kind=row.get("type").and_then(Value::as_str).filter(|s|matches!(*s,"llm"|"embedding"|"decision")).ok_or(ERROR)?.to_string();
            let label=row.get("display_name").and_then(Value::as_str).filter(|s|!s.trim().is_empty()&&s.len()<=256&&!s.chars().any(char::is_control)).unwrap_or(&key).to_string();
            if kind=="llm"&&(!row.get("size_bytes").and_then(Value::as_u64).is_some_and(|n|n>0)||!matches!(row.get("format").and_then(Value::as_str),Some("gguf"|"mlx"))){return Err(ERROR.into());}
            let loaded=row.get("loaded_instances").and_then(Value::as_array).filter(|v|v.len()<=32).ok_or(ERROR)?;
            let mut instances=vec![];let mut seen=BTreeSet::new();
            for instance in loaded{
                let name=instance.get("id").and_then(Value::as_str).filter(|s|id(s)).ok_or(ERROR)?.to_string();
                let config=instance.get("config").filter(|v|v.is_object()&&serde_json::to_vec(v).is_ok_and(|b|b.len()<=8192)).ok_or(ERROR)?.clone();
                if !seen.insert(name.clone()){return Err(ERROR.into());}instances.push(Instance{id:name,config});
            }
            // Select identity fields, not description, timings or loaded count.
            let identity=json!({"type":kind,"key":key,"publisher":row.get("publisher"),"architecture":row.get("architecture"),"format":row.get("format"),"sizeBytes":row.get("size_bytes"),"quantization":row.get("quantization"),"selectedVariant":row.get("selected_variant")});
            models.push(Model{key,label,kind,identity,instances});
        }
        Ok(Self{endpoint:base.as_str().trim_end_matches('/').into(),auth:digest(&server.key)?,models})
    }
    pub(crate)fn binding(&self,requested:&str)->Result<Binding,String>{
        if !id(requested){return Err(ERROR.into());}let mut found=vec![];
        for model in &self.models{
            if model.key==requested{if model.kind!="llm"{return Err(ERROR.into());}found.push((model,None));}
            for instance in &model.instances{if instance.id==requested&&instance.id!=model.key{if model.kind!="llm"{return Err(ERROR.into());}found.push((model,Some(instance)));}}
        }
        if found.len()!=1{return Err(ERROR.into());}let(model,instance)=found[0];
        let binding=Binding{schema_version:1,endpoint:self.endpoint.clone(),model_key:model.key.clone(),instance_id:instance.map(|i|i.id.clone()),model_hash:digest(&(&self.endpoint,&self.auth,&model.identity,instance.map(|i|(&i.id,&i.config))))?};
        if !binding.valid_for(requested){return Err(ERROR.into());}Ok(binding)
    }
    pub(crate)fn models(&self)->Vec<ModelInfo>{
        let mut names=BTreeSet::new();let mut result=vec![];
        for model in self.models.iter().filter(|m|m.kind=="llm"){
            for name in std::iter::once(&model.key).chain(model.instances.iter().map(|i|&i.id)){
                if names.insert(name.clone())&&self.binding(name).is_ok(){result.push(ModelInfo{id:name.clone(),label:if name==&model.key{model.label.clone()}else{format!("{} · {name}",model.label)}});}
            }
        }result
    }
}
/// One explicit native metadata read. No load, download, inference or fallback.
pub(crate)async fn catalog(server:&Server,current:impl Fn()->bool)->Result<Catalog,String>{
    if server.id!="lmstudio"{return Err(ERROR.into());}let base=crate::local_chat::base_url(server)?;
    if !net::is_loopback_url(&base){return Err("Este pet LM Studio requer o servidor local explicitamente configurado.".into());}
    let generation=crate::local_chat::lmstudio_key_generation();if generation%2!=0{return Err("A credencial do LM Studio está a mudar; volta a rever o pedido.".into());}
    let ticket=crate::pet_capabilities::begin_lmstudio(&server.url,generation);
    let result=net::while_current(||current()&&generation==crate::local_chat::lmstudio_key_generation(),||async{
        let client=net::local_client(&base,Duration::from_secs(3))?;
        let mut request=client.get(net::join(&base,"api/v1/models"));if let Some(key)=&server.key{request=request.bearer_auth(key);}
        let response=request.send().await.map_err(|_|"Não foi possível consultar o LM Studio local.")?;
        if matches!(response.status().as_u16(),401|403){return Err("O LM Studio exige uma credencial válida para este servidor.".into());}
        if !response.status().is_success(){return Err("O servidor não disponibilizou os metadados nativos LM Studio v1.".into());}
        let bytes=net::read_capped(response,512*1024).await?;let value:Value=serde_json::from_slice(&bytes).map_err(|_|ERROR)?;
        Catalog::parse(server,&value)
    }).await;
    crate::pet_capabilities::finish_lmstudio(ticket,result.as_ref().ok());result
}
pub(crate)async fn inspect(server:&Server,model:&str,current:impl Fn()->bool)->Result<Binding,String>{catalog(server,current).await?.binding(model)}

#[cfg(test)]#[path="lmstudio_tests.rs"]mod tests;
