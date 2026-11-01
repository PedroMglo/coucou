//! Selection only. No device IO, fallback, grant, or physical-identity claim.
const MAX_INPUTS:usize=256;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super)enum Failure{Missing,Ambiguous,Unavailable,InventoryLimit,Invalid}
impl Failure{
    pub(super)fn message(self)->String{match self{
        Self::Missing=>"The selected audio source is unavailable; no other source was selected.",
        Self::Ambiguous=>"The selected audio source is ambiguous; choose an unambiguous source before recording.",
        Self::Unavailable=>"The audio source inventory could not be confirmed.",
        Self::InventoryLimit=>"The audio source inventory exceeds the selection limit.",
        Self::Invalid=>"The microphone selection could not be reviewed.",
    }.into()}
}
pub(super)struct Description{pub name:String,pub monitor:bool,pub default_alias:bool}
pub(super)fn unique<T>(devices:impl IntoIterator<Item=T>,matches:impl Fn(&T)->bool)->Result<T,Failure>{
    let mut selected=None;
    for(index,device)in devices.into_iter().enumerate(){
        if index>=MAX_INPUTS{return Err(Failure::InventoryLimit);}
        if matches(&device){if selected.is_some(){return Err(Failure::Ambiguous);}selected=Some(device);}
    }
    selected.ok_or(Failure::Missing)
}
/// Legacy names remain exact, case-sensitive data. A missing or ambiguous
/// explicit selection must never acquire the host's unrelated default device.
pub(super)fn microphone<T,I:IntoIterator<Item=T>>(
    wanted:Option<&str>,devices:impl FnOnce()->Result<I,Failure>,default:impl FnOnce()->Option<T>,describe:impl Fn(&T)->Description,
)->Result<T,Failure>{
    if let Some(name)=wanted{
        if name.is_empty()||name.len()>1024{return Err(Failure::Invalid);}
        unique(devices()?,|d|{let d=describe(d);!d.monitor&&!d.default_alias&&d.name==name})
    }else{
        let device=default().ok_or(Failure::Missing)?;
        if describe(&device).monitor{return Err(Failure::Missing);}
        Ok(device)
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[derive(Clone,Debug,PartialEq,Eq)]struct Device{id:u32,name:&'static str,monitor:bool,alias:bool}
    fn mic(id:u32,name:&'static str)->Device{Device{id,name,monitor:false,alias:false}}
    fn describe(d:&Device)->Description{Description{name:d.name.into(),monitor:d.monitor,default_alias:d.alias}}
    #[test]fn explicit_missing_or_unavailable_input_never_reads_or_acquires_default(){
        for inventory in[Ok(vec![mic(2,"other")]),Err(Failure::Unavailable)]{
            let result=microphone(Some("chosen"),||inventory,||panic!("unrelated default consulted"),describe);
            assert!(matches!(result,Err(Failure::Missing|Failure::Unavailable)));
        }
    }
    #[test]fn duplicate_names_are_refused_in_both_orders_instead_of_selecting_a_different_device(){
        for rows in[vec![mic(1,"same"),mic(2,"same")],vec![mic(2,"same"),mic(1,"same")]]{
            assert_eq!(microphone(Some("same"),||Ok(rows),||panic!("default fallback"),describe),Err(Failure::Ambiguous));
        }
    }
    #[test]fn monitor_or_virtual_default_alias_cannot_shadow_an_explicit_microphone(){
        let mut monitor=mic(2,"selected");monitor.monitor=true;let mut alias=mic(3,"selected");alias.alias=true;
        for rows in[vec![monitor.clone(),mic(1,"selected"),alias.clone()],vec![alias.clone(),mic(1,"selected"),monitor.clone()]]{
            assert_eq!(microphone(Some("selected"),||Ok(rows),||panic!("default fallback"),describe).unwrap().id,1);
        }
        assert_eq!(microphone(Some("selected"),||Ok(vec![monitor,alias]),||panic!("default fallback"),describe),Err(Failure::Missing));
    }
    #[test]fn explicit_default_uses_only_the_current_default_and_refuses_a_monitor(){
        assert_eq!(microphone(None,||->Result<Vec<Device>,Failure>{panic!("unexpected enumeration")},||Some(mic(7,"default")),describe).unwrap().id,7);
        let mut monitor=mic(8,"default");monitor.monitor=true;
        assert_eq!(microphone(None,||Ok(Vec::<Device>::new()),||Some(monitor),describe),Err(Failure::Missing));
        assert_eq!(microphone(None,||Ok(vec![mic(7,"available")]),||None,describe),Err(Failure::Missing));
    }
    #[test]fn invalid_or_excessive_selection_fails_before_a_different_input_is_opened(){
        for name in["".to_owned(),"x".repeat(1025)]{assert_eq!(microphone(Some(&name),||->Result<Vec<Device>,Failure>{panic!("invalid selection enumerated")},||panic!("default fallback"),describe),Err(Failure::Invalid));}
        let rows=(0..257).map(|i|mic(i,"other"));assert_eq!(unique(rows,|d|d.id==1),Err(Failure::InventoryLimit));
        assert_eq!(unique((0..256).map(|i|mic(i,"other")),|d|d.id==255).unwrap().id,255);
    }
    #[test]fn missing_or_duplicate_loopback_target_never_selects_another_monitor(){
        let rows=vec![("hostA","other.monitor"),("hostB","wanted.monitor")];
        assert_eq!(unique(rows,|d|*d==("hostA","wanted.monitor")),Err(Failure::Missing));
        assert_eq!(unique(vec![("hostA","wanted.monitor");2],|d|*d==("hostA","wanted.monitor")),Err(Failure::Ambiguous));
        assert_eq!(unique(vec![("hostA","other.monitor"),("hostA","wanted.monitor")],|d|*d==("hostA","wanted.monitor")).unwrap(),("hostA","wanted.monitor"));
    }
}
