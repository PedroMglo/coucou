import {h} from '../views/dom';
import {t} from '../i18n/i18n';

/** Shows a confirmed preference, not proof that the host registered a login item.
 * A failed/unknown operation is read back, never automatically sent again. */
export function buildAutostartSetting(options:{initial:unknown;save(value:boolean):Promise<unknown>;read():Promise<unknown>;adopt(value:boolean):void}){
 let value=typeof options.initial==='boolean'?options.initial:false,known=typeof options.initial==='boolean',busy=false,observations=0,error='',status='';
 const label=h('label'),button=h('button',{type:'button',class:'switch','data-preference':'autostart'}) as HTMLButtonElement;
 const note=h('p',{role:'status',class:'hint'}),refresh=h('button',{type:'button',text:'Consultar preferência de arranque',onclick:()=>void check()}) as HTMLButtonElement;
 const el=h('div',{},h('div',{class:'row'},label,button),note,refresh);
 function accept(next:unknown){if(typeof next!=='boolean')return false;value=next;known=true;options.adopt(value);return true;}
 function render(){label.textContent=t("Launch at startup");button.setAttribute('aria-label',label.textContent);button.setAttribute('aria-pressed',known?String(value):'mixed');button.setAttribute('aria-busy',String(busy));button.classList.toggle('on',value);button.disabled=busy||!known;note.textContent=error||status;note.setAttribute('role',error?'alert':'status');refresh.hidden=!error&&known;refresh.disabled=busy;}
 async function read(){const observed=observations;try{const next=await options.read();if(observations!==observed)return known;if(!accept(next))throw Error('A preferência de arranque não foi confirmada.');return true;}catch(e){if(observations!==observed)return known;known=false;error=[error,String(e).replace(/^Error:\s*/,'')].filter(Boolean).join(' ');return false;}}
 async function check(){if(busy)return;busy=true;error='';status='A consultar a preferência de arranque…';render();try{if(await read())status='Preferência atual consultada. O registo no sistema mantém a sua verificação própria.';}finally{busy=false;render();}}
 async function change(){if(busy||!known)return;const wanted=!value;busy=true;error='';status='A guardar a preferência de arranque…';render();try{try{await options.save(wanted);}catch(e){error=String(e).replace(/^Error:\s*/,'');}if(await read())status=value===wanted?'Preferência de arranque confirmada.':'A preferência atual foi conservada.';}finally{busy=false;render();}}
 button.addEventListener('click',()=>void change());render();return {el,render,observe(next:unknown){if(accept(next)){observations++;render();}}};
}
