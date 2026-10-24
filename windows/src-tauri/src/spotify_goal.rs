//! Native instance binding for goal steps. This layer does not grant execution;
//! the goal dispatcher must hold its worker/resource and debit before_effect.
use dbus::{blocking::{Connection, stdintf::org_freedesktop_dbus::Properties}, arg::PropMap};
use serde::{Serialize, Deserialize};
use std::{sync::Mutex, time::{Duration,Instant}};
const BUS: &str = "org.mpris.MediaPlayer2.spotify";
const PATH: &str = "/org/mpris/MediaPlayer2";
const ROOT: &str = "org.mpris.MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const DBUS: &str = "org.freedesktop.DBus";
const CALL: Duration = Duration::from_millis(1000);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Binding {
    pub schema_version:u32, pub bus_id:String, pub owner:String,
    pub uid:u32, pub pid:u32, pub identity:String,
}
/// Retained local connection, never created from a deserialized Binding. It
/// never reconnects, starts an application, or follows a replacement owner.
pub(crate) struct Target { connection:Mutex<Connection>, binding:Binding }
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag="action",rename_all="snake_case",deny_unknown_fields)]
pub(crate) enum Action { Play, Pause, SetVolume { volume:u8 } }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Snapshot { pub playback_status:String, pub volume:f64, pub observed_at:u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Failure { InvalidAddress,Unavailable,Changed,InvalidState,Cancelled,Deadline,InvalidAction,NotAuthorized,Transport }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub(crate) enum Delivery { NotDispatched,Accepted,Uncertain }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(crate) struct Observation {
    pub delivery:Delivery, pub before:Option<Snapshot>, pub after:Option<Snapshot>,
    pub effect_verified:bool, pub scope_current:bool, pub failure:Option<Failure>,
}
fn now()->u64 { crate::assistant_memory::now() }
fn local_address(address:&str)->bool {
    address.starts_with("unix:") && address.len()<=4096 && !address.contains(';')
        && !address.chars().any(char::is_control)
}
fn peer(conn:&Connection)->Result<(String,String,u32,u32),Failure>{
    let p=conn.with_proxy(DBUS,"/org/freedesktop/DBus",CALL);
    let (bus,):(String,)=p.method_call(DBUS,"GetId",()).map_err(|_|Failure::Unavailable)?;
    let (owner,):(String,)=p.method_call(DBUS,"GetNameOwner",(BUS,)).map_err(|_|Failure::Unavailable)?;
    let (uid,):(u32,)=p.method_call(DBUS,"GetConnectionUnixUser",(owner.as_str(),)).map_err(|_|Failure::Unavailable)?;
    let (pid,):(u32,)=p.method_call(DBUS,"GetConnectionUnixProcessID",(owner.as_str(),)).map_err(|_|Failure::Unavailable)?;
    if bus.is_empty()||bus.len()>128||!owner.starts_with(':')||owner.len()>256||uid!=unsafe{libc::geteuid()}||pid==0{return Err(Failure::Changed);}
    Ok((bus,owner,uid,pid))
}
impl Target {
    pub(crate) fn capture()->Result<Self,Failure>{
        let address=std::env::var("DBUS_SESSION_BUS_ADDRESS").map_err(|_|Failure::Unavailable)?;
        Self::at(&address)
    }
    #[cfg(test)]pub(crate) fn fixture_at(address:&str)->Result<Self,Failure>{Self::at(address)}
    fn at(address:&str)->Result<Self,Failure>{
        if !local_address(address){return Err(Failure::InvalidAddress);}
        let connection=Connection::new_address(address).map_err(|_|Failure::Unavailable)?;
        let(bus_id,owner,uid,pid)=peer(&connection)?;
        let proxy=connection.with_proxy(owner.as_str(),PATH,CALL);
        let identity:String=proxy.get(ROOT,"Identity").map_err(|_|Failure::Unavailable)?;
        if identity.trim().is_empty()||identity.len()>256||identity.chars().any(char::is_control){return Err(Failure::Changed);}
        let binding=Binding{schema_version:1,bus_id,owner,uid,pid,identity};
        let target=Self{connection:Mutex::new(connection),binding};
        target.read()?;Ok(target)
    }
    pub(crate) fn binding(&self)->&Binding{&self.binding}
    fn check(&self,connection:&Connection)->Result<(),Failure>{
        if !connection.channel().is_connected(){return Err(Failure::Unavailable);}
        let(bus,owner,uid,pid)=peer(connection)?;
        if bus!=self.binding.bus_id||owner!=self.binding.owner||uid!=self.binding.uid||pid!=self.binding.pid{return Err(Failure::Changed);}
        let identity:String=connection.with_proxy(self.binding.owner.as_str(),PATH,CALL).get(ROOT,"Identity").map_err(|_|Failure::Changed)?;
        if identity!=self.binding.identity{return Err(Failure::Changed);}Ok(())
    }
    fn snapshot(&self,connection:&Connection)->Result<Snapshot,Failure>{
        self.check(connection)?;
        let properties:PropMap=connection.with_proxy(self.binding.owner.as_str(),PATH,CALL).get_all(PLAYER).map_err(|_|Failure::Unavailable)?;
        let playback_status=properties.get("PlaybackStatus").and_then(|v|v.0.as_str()).filter(|s|matches!(*s,"Playing"|"Paused"|"Stopped")).ok_or(Failure::InvalidState)?.to_owned();
        let volume=properties.get("Volume").and_then(|v|v.0.as_f64()).filter(|v|v.is_finite()&&(0.0..=1.0).contains(v)).ok_or(Failure::InvalidState)?;
        self.check(connection)?;
        Ok(Snapshot{playback_status,volume,observed_at:now()})
    }
    pub(crate) fn read(&self)->Result<Snapshot,Failure>{
        let connection=self.connection.lock().map_err(|_|Failure::Unavailable)?;
        self.snapshot(&connection)
    }
    /// before_effect must durably authorize/debit exactly this call. It is
    /// invoked once, after native target validation and immediately before IO.
    pub(crate) fn apply(&self,action:&Action,deadline:Instant,current:impl Fn()->bool,before_effect:impl FnOnce()->Result<(),Failure>)->Observation{
        let mut observation=Observation{delivery:Delivery::NotDispatched,before:None,after:None,effect_verified:false,scope_current:false,failure:None};
        fn failed(mut observation:Observation,failure:Failure)->Observation{observation.failure=Some(failure);observation}
        if matches!(action,Action::SetVolume{volume} if *volume>100){return failed(observation,Failure::InvalidAction);}
        if Instant::now()>=deadline{return failed(observation,Failure::Deadline);}
        if !current(){return failed(observation,Failure::Cancelled);}
        let connection=match self.connection.lock(){Ok(c)=>c,Err(_)=>return failed(observation,Failure::Unavailable)};
        let before=match self.snapshot(&connection){Ok(s)=>s,Err(e)=>return failed(observation,e)};
        observation.before=Some(before);
        if Instant::now()>=deadline{observation.failure=Some(Failure::Deadline);return observation;}
        if !current(){observation.failure=Some(Failure::Cancelled);return observation;}
        if let Err(e)=before_effect(){observation.failure=Some(e);return observation;}
        if Instant::now()>=deadline{observation.failure=Some(Failure::Deadline);return observation;}
        if !current(){observation.failure=Some(Failure::Cancelled);return observation;}
        if let Err(e)=self.check(&connection){observation.failure=Some(e);return observation;}
        // Address the captured unique owner, not the mutable well-known alias.
        let proxy=connection.with_proxy(self.binding.owner.as_str(),PATH,CALL);
        let response:Result<(),dbus::Error>=match action{
            Action::Play=>proxy.method_call(PLAYER,"Play",()),
            Action::Pause=>proxy.method_call(PLAYER,"Pause",()),
            Action::SetVolume{volume}=>proxy.set(PLAYER,"Volume",f64::from(*volume)/100.0),
        };
        observation.delivery=if response.is_ok(){Delivery::Accepted}else{Delivery::Uncertain};
        if response.is_err(){observation.failure=Some(Failure::Transport);}
        // A failed ACK is reconciled by observation; it never repeats IO.
        observation.after=match self.snapshot(&connection){Ok(state)=>Some(state),Err(error)=>{if observation.failure.is_none(){observation.failure=Some(error);}None}};
        observation.scope_current=current()&&Instant::now()<deadline&&self.check(&connection).is_ok();
        observation.effect_verified=observation.scope_current&&observation.after.as_ref().is_some_and(|state|match action{
            Action::Play=>state.playback_status=="Playing",
            Action::Pause=>matches!(state.playback_status.as_str(),"Paused"|"Stopped"),
            Action::SetVolume{volume}=>(state.volume-f64::from(*volume)/100.0).abs()<=0.000_001,
        });
        observation
    }
}

#[cfg(test)]#[path="spotify_goal_tests.rs"]pub(crate) mod tests;
