import type { CompanionsSnapshot } from "./companions";
type Band = NonNullable<CompanionsSnapshot["band"]>;
/** The home and its drawer occupy free bar space only, including on narrow bars. */
export function shelterLayout(band: Band | null | undefined, count: number, expanded: boolean) {
  if (!band || !count) return null;
  const lane = band.lanes.filter(([a,b])=>b-a>=32).sort((a,b)=>(b[1]-b[0])-(a[1]-a[0]))[0];
  if (!lane) return null;
  const width = expanded ? Math.min(lane[1]-lane[0],32+count*36) : 32;
  const left = lane[1]-width;
  const lanes = band.lanes.flatMap(([a,b])=>a===lane[0] && b===lane[1] ? (left-a>=48 ? [[a,left-4] as [number,number]] : []) : [[a,b] as [number,number]]);
  return {left,width,top:Math.max(0,(band.height-36)/2),height:Math.min(36,band.height),lanes};
}

/** A speech tail stays attached to its moving owner and never crosses bar controls. */
export function speechPlacement(x:number,y:number,scale:number,band:Band | null | undefined,width:number,others:readonly number[]=[]) {
  if (!band) return {left:Math.max(8,Math.min(width-320,x-150)),top:y+38,width:312,side:"top",tip:150};
  const lane=band.lanes.find(([a,b])=>x>=a && x<=b);
  if(!lane)return null;
  const radius=24*scale;
  const left=Math.max(lane[0],...others.filter(v=>v<x).map(v=>v+radius+4));
  const right=Math.min(lane[1],...others.filter(v=>v>x).map(v=>v-radius-4));
  const a=x-radius-6-left,b=right-x-radius-6;
  const onRight=b>=a,available=Math.min(312,onRight?b:a);
  if(available<48)return null;
  const height=Math.min(38,band.height-2);
  return {left:onRight?x+radius+6:x-radius-6-available,top:Math.max(1,(band.height-height)/2),width:available,height,side:onRight?"left":"right",tip:Math.min(19,band.height/2)};
}

/** Public speech is brief; thinking/listening are expressions, never status cards. */
export function speechText(phase:string,message:string,messageAt:number,now:number) {
  if(["approval","question","error","ratelimit","failed"].includes(phase)) return ({approval:"Preciso da tua aprovação",question:"Preciso de uma resposta tua",error:"Preciso da tua atenção",ratelimit:"Atingi o limite de utilização",failed:"Não consegui terminar"} as Record<string,string>)[phase];
  if (!message.trim() || (phase!=="speaking" && now-messageAt>9000)) return "";
  return message;
}
