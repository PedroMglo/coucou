// Fixed product code. No file/provider/model content is evaluated as QML.
pub(super)const QML:&str=r#"
import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Pipewire
ShellRoot {
 id: root
 property var captured: null
 property bool capturedOnce: false
 property bool lost: false
 property int sequence: 0
 readonly property int maxRequests: Math.min(256,Math.max(1,Number(Quickshell.env("COUCOU_PW_MAX_REQUESTS")||16)))
 readonly property int lifetimeMs: Math.min(180000,Math.max(1,Number(Quickshell.env("COUCOU_PW_LIMIT_MS")||30000)))
 property var pending: null
 property var preparedEffect: null
 property bool preparedOnce: false
 property bool committed: false
 property bool connectedOnce: false
 readonly property bool effectsEnabled: Quickshell.env("COUCOU_PW_EFFECTS")==="1"
 property var outputNodes: Pipewire.nodes.values.filter(n => n.audio && n.isSink && !n.isStream)
 readonly property string session: Quickshell.env("COUCOU_PW_SESSION")
 function invalidate() {
  if (capturedOnce && (!Pipewire.ready || !captured || !captured.ready || outputNodes.indexOf(captured) < 0)) lost = true;
 }
 onOutputNodesChanged: invalidate()
 onCapturedChanged: invalidate()
 Connections { target: Pipewire; function onReadyChanged() { root.invalidate(); } }
 Connections { target: root.captured; function onReadyChanged() { root.invalidate(); } }
 PwObjectTracker { objects: root.captured ? [root.captured] : root.outputNodes }
 function snapshot(node) {
  const serial = String(node.properties["object.serial"] || "");
  const volume = Number(node.audio.volume) * 100;
  return {id:node.id, serial:serial, volumePercent:Number.isFinite(volume) && volume>=0 && volume<=150 ? volume : null, muted:typeof node.audio.muted==="boolean" ? node.audio.muted : null};
 }
 function reply(ok, outputs, selected, prepared=false, accepted=false, transaction=null) {
  channel.write(JSON.stringify({version:2, session:session, sequence:sequence, ok:ok, outputs:outputs, captured:selected, lost:lost, prepared:prepared, accepted:accepted, transaction:transaction})+"\n");channel.flush();pending=null;
 }
 function resolve() {
  if (!pending) return;
  invalidate();
  if (lost) { reply(false,[],null);return; }
  if (!Pipewire.ready) return;
  if (pending.operation==="prepare" && effectsEnabled && capturedOnce && captured && !preparedOnce && !committed) {
   preparedOnce=true;preparedEffect={transaction:pending.transaction,effect:pending.effect};
   reply(true,[],snapshot(captured),true,false,preparedEffect.transaction);return;
  }
  if (pending.operation==="commit" && effectsEnabled && capturedOnce && captured && preparedEffect && !committed && pending.transaction===preparedEffect.transaction) {
   const staged=preparedEffect;preparedEffect=null;committed=true;
   // Consume before the setter. Lost ACK, replay or a second commit cannot
   // create another write. This response is not native server/hardware ACK.
   if (staged.effect.kind==="volume") captured.audio.volume=staged.effect.percent/100;
   else captured.audio.muted=staged.effect.muted;
   reply(true,[],snapshot(captured),false,true,staged.transaction);return;
  }

  // Once pinned, untracked outputs may be unbound. Only the original object
  // is required for readback; never reselect it by an id or default.
  if (pending.operation==="read" && capturedOnce && captured && !lost) { reply(true,[],snapshot(captured));return; }
  const nodes=outputNodes.filter(n => n.ready);
  if (nodes.length!==outputNodes.length) return;
  if (nodes.length>64) { reply(false,[],null);return; }
  if (pending.operation==="outputs" && !capturedOnce) { reply(true,nodes.map(n=>snapshot(n)),null);return; }
  if (pending.operation==="capture" && !capturedOnce) {
   const selected=nodes.filter(n=>n.id===pending.id && String(n.properties["object.serial"]||"")===pending.serial);
   if (selected.length!==1) { lost=true;reply(false,[],null);return; }
   captured=selected[0];capturedOnce=true;invalidate();reply(!lost,[],lost ? null : snapshot(captured));return;
  }
  if (pending.operation==="read" && capturedOnce && captured && !lost) { reply(true,[],snapshot(captured));return; }
  lost=true;reply(false,[],null);
 }
 Socket {
  id:channel;path:Quickshell.env("COUCOU_PW_SOCKET");connected:true
  onConnectionStateChanged: { if (connected) root.connectedOnce=true;else if (root.connectedOnce) Qt.quit(); }
  parser:SplitParser {
   onRead:data=>{
    if (data.length>2048 || root.pending || root.sequence>=root.maxRequests) { Qt.quit();return; }
    let v;try { v=JSON.parse(data); } catch(e) { Qt.quit();return; }
    if (!v || typeof v!=="object" || Array.isArray(v)) { Qt.quit();return; }
    const keys=Object.keys(v).sort().join(",");
    const expected=v.operation==="prepare" ? "effect,id,operation,sequence,serial,session,transaction,version" : v.operation==="commit" ? "id,operation,sequence,serial,session,transaction,version" : "id,operation,sequence,serial,session,version";
    if (keys!==expected || v.version!==2 || v.session!==root.session || !Number.isInteger(v.sequence) || v.sequence!==root.sequence+1 || ["outputs","capture","read","prepare","commit"].indexOf(v.operation)<0) { Qt.quit();return; }
    if (v.operation==="capture" ? (!Number.isInteger(v.id)||v.id<=0||typeof v.serial!=="string"||!/^[0-9]{1,24}$/.test(v.serial)) : (v.id!==null||v.serial!==null)) { Qt.quit();return; }
    if (v.operation==="prepare" || v.operation==="commit") {
     if (!root.effectsEnabled || typeof v.transaction!=="string" || !/^[a-f0-9]{64}$/.test(v.transaction)) { Qt.quit();return; }
     if (v.operation==="prepare") {
      if (!v.effect || typeof v.effect!=="object") { Qt.quit();return; }
      const e=v.effect,k=Object.keys(e).sort().join(",");
      const valid=e.kind==="volume" ? k==="kind,percent" && Number.isInteger(e.percent) && e.percent>=0 && e.percent<=100 : e.kind==="mute" && k==="kind,muted" && typeof e.muted==="boolean";
      if (!valid) { Qt.quit();return; }
     }
    }
    root.sequence=v.sequence;root.pending=v;root.resolve();
   }
  }
 }
 Timer { interval:20;running:true;repeat:true;onTriggered:root.resolve() }
 Timer { interval:root.lifetimeMs;running:true;onTriggered:Qt.quit() }
}
"#;
