import {h} from '../views/dom';
import {t,N_} from '../i18n/i18n';
import {IslandAvailabilityPreferences,type IslandAvailabilityKey,type IslandAvailabilityPatch} from '../core/island-availability';
import type {Settings} from '../core/state';
const names={petControlBar:N_("Pet island"),petSystemCenter:N_("System in the island")};
const saving={petControlBar:N_("Saving the pet island…"),petSystemCenter:N_("Saving system access…")};

export function buildIslandAvailability(options:{initial:unknown;save(patch:IslandAvailabilityPatch):Promise<unknown>;read():Promise<unknown>;adopt(flags:Pick<Settings,'petControlBar'|'petSystemCenter'>):void}){
 const petsHint=h('div',{class:'hint'}),systemHint=h('div',{class:'hint'});
 const el=h('div',{class:'island-availability-controls',style:'display:flex;flex-direction:column;gap:12px'}),controls=new Map<IslandAvailabilityKey,{label:HTMLElement;button:HTMLButtonElement}>();
 for(const key of ['petControlBar','petSystemCenter'] as const){
  const label=h('label'),button=h('button',{type:'button',class:'switch'}) as HTMLButtonElement;
  button.dataset.preference=key;button.onclick=()=>void preferences.change(key);
  controls.set(key,{label,button});el.append(h('div',{class:'row'},label,button),key==='petControlBar'?petsHint:systemHint);
 }
 const status=h('p',{class:'hint',role:'status',style:'min-height:1.5em'});
 const refresh=h('button',{type:'button',onclick:()=>void preferences.refresh()}) as HTMLButtonElement;
 el.append(status,h('div',{class:'row'},refresh));
 const preferences=new IslandAvailabilityPreferences({...options,changed:render});
 function render(){
  const flags=preferences.flags,busy=preferences.busy;
  for(const[key,control]of controls){
   const name=t(names[key]);control.label.textContent=name;
   control.button.setAttribute('aria-label',name);control.button.setAttribute('aria-pressed',preferences.known?String(flags[key]):'mixed');
   control.button.setAttribute('aria-busy',String(preferences.pending===key));control.button.classList.toggle('on',flags[key]);control.button.disabled=busy||!preferences.known;
  }
  petsHint.textContent=t("Show the pet gallery and management when desktop companions are enabled.");
  systemHint.textContent=t("Keep the clock, date and system controls available even when desktop companions are off.");
  const text=preferences.status==='saving'?t(saving[preferences.pending??'petSystemCenter']):preferences.status==='checking'?t("Checking the saved preferences…"):preferences.status==='saved'?t("Preferences confirmed."):preferences.status==='changed'?t("The current values changed. The confirmed values were kept."):preferences.status==='failed'?t("Could not save this change. The current confirmed values are shown."):preferences.status==='unconfirmed'?t("Could not confirm the current preferences. Refresh before changing them."):'',error=['failed','unconfirmed'].includes(preferences.status);
  if(status.textContent!==text)status.textContent=text;status.setAttribute('role',error?'alert':'status');status.className=error?'notice err':'hint';
  refresh.textContent=t("Refresh preferences");refresh.hidden=!error;refresh.disabled=busy;
  el.setAttribute('aria-busy',String(busy));
 }
 render();return {el,observe:(settings:unknown)=>preferences.observe(settings),render};
}
