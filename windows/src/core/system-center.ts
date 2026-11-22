export type SystemPanel='calendar'|'media'|'weather'|'notifications'|'quick-settings'|'wifi'|'bluetooth'|'battery'|'audio'|'clipboard'|'disks'|'recording'|'tray';
export interface SystemTrayItem {id:string;label:string;hasMenu:boolean;onlyMenu:boolean}
export interface SystemCenterStatus {
 available:boolean;updatedAt:number|null;
 media:{title:string;artist:string;playing:boolean}|null;
 weather:{label:string;temperature:number;unit:string;location?:string|null}|null;
 notifications:number|null;
 network?:{available:boolean;enabled:boolean|null;connected:boolean|null;ssid:string|null;signal:number|null;ethernet:boolean|null}|null;
 bluetooth?:{available:boolean;enabled:boolean|null;connectedNames:string[]}|null;
 battery?:{present:boolean;percent:number|null;charging:boolean|null;plugged:boolean|null}|null;
 audio?:{available:boolean;volume:number|null;muted:boolean|null;device:string|null;micMuted:boolean|null}|null;
 recording?:{available:boolean;active:boolean|null;label:string}|null;
 clipboard?:{available:boolean}|null;disks?:{available:boolean}|null;tray?:SystemTrayItem[]|null;
}
const percent=(n:number|null|undefined)=>n!==null&&n!==undefined&&Number.isFinite(n)?`${Math.round(n)}%`:null;
/** Missing data is unknown, never a fabricated off/empty state. */
export function systemDetails(status:SystemCenterStatus|null):Record<'wifi'|'bluetooth'|'battery'|'audio'|'clipboard'|'disks'|'recording'|'tray',string>{
 const unknown='Estado indisponível';if(!status?.available)return {wifi:unknown,bluetooth:unknown,battery:unknown,audio:unknown,clipboard:unknown,disks:unknown,recording:unknown,tray:unknown};
 const n=status.network,b=status.bluetooth,p=status.battery,a=status.audio,r=status.recording;
 return {
  wifi:!n?unknown:!n.available?'Rede indisponível':n.connected===true?[n.ssid||(n.ethernet?'Ligação por cabo':'Ligado'),n.ssid?percent(n.signal):null].filter(Boolean).join(' · '):n.enabled===false?'Wi-Fi desligado':n.connected===false?'Sem ligação':unknown,
  bluetooth:!b?unknown:!b.available?'Sem adaptador':b.enabled===false?'Desligado':b.connectedNames.length?b.connectedNames.join(', '):b.enabled===true?'Ligado · sem dispositivos':unknown,
  battery:!p?unknown:!p.present?'Sem bateria':p.percent===null?unknown:[percent(p.percent),p.charging===true?'A carregar':p.plugged===true?'Ligado à corrente':null].filter(Boolean).join(' · '),
  audio:!a?unknown:!a.available?'Áudio indisponível':[a.muted===true?'Sem som':percent(a.volume)??unknown,a.device].filter(Boolean).join(' · '),
  recording:!r?unknown:!r.available?'Gravação indisponível':r.active===true?'A gravar o ecrã':r.active===false?r.label||'Sem gravação ativa':r.label||unknown,
  clipboard:status.clipboard?.available?'Histórico da área de transferência':status.clipboard?'Indisponível':unknown,
  disks:status.disks?.available?'Unidades e suportes removíveis':status.disks?'Indisponível':unknown,
  tray:status.tray?status.tray.length?`${status.tray.length} aplicações`:'Sem ícones ativos':unknown,
 };
}
