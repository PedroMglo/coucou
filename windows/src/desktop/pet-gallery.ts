import {PetObservability,observePetObservability} from '../core/pet-observability-resource';
import {petObservation} from '../core/pet-observability';
import {scopedSessionSummary} from '../core/scoped-session-summary';
import {PetPreferences} from '../mochi/pet-preferences';
import {petCapabilities} from '../mochi/capabilities';
import {createMiniBot} from '../mochi/minibots';
import type {PetIdentity} from '../mochi/companions';
import type {AgentTask} from '../core/state';

/** The centre opens on the whole household, including resting pets. */
export function petGallery(open:(pet:PetIdentity)=>void,sessions:(pet:PetIdentity)=>void){
 const el=document.createElement('section');el.id='pet-gallery';el.setAttribute('aria-label','Todos os pets');
 const heading=document.createElement('header'),title=document.createElement('h1'),summary=document.createElement('p');
 title.textContent='Os teus pets';heading.append(title,summary);
 const grid=document.createElement('div');grid.className='pet-gallery-grid';
 const hint=document.createElement('p');hint.className='pet-gallery-hint';hint.textContent='Escolhe um pet para conversar ou acompanhar o que está a fazer.';el.append(heading,grid,hint);
 const cards=new Map<string,{el:HTMLButtonElement;name:HTMLElement;state:HTMLElement;action:HTMLElement;metrics:HTMLElement;pet:PetIdentity}>();
 let current:PetIdentity[]=[];
 const visible=()=>el.isConnected&&!el.hidden&&!document.hidden&&(!el.closest<HTMLElement>('#pet-control')||el.closest<HTMLElement>('#pet-control')!.dataset.mode==='home');
 const view={el,sync(pets:PetIdentity[]){
  current=pets;if(visible())void PetObservability.refresh();
  const ids=new Set(pets.map(p=>p.id));for(const[id,c]of cards)if(!ids.has(id)){c.el.remove();cards.delete(id);}
  const waiting=pets.filter(p=>p.attention).length,busy=pets.filter(p=>p.busy).length;
  summary.textContent=[`${pets.length} disponíveis`,busy?`${busy} a trabalhar`:'',waiting?`${waiting} a precisar de ti`:''].filter(Boolean).join(' · ');
  let index=0;for(const pet of pets){
   let card=cards.get(pet.id);
   if(!card){
    const button=document.createElement('button');button.type='button';button.className='pet-gallery-card';button.dataset.pet=pet.id;
    const scene=document.createElement('span');scene.className='pet-gallery-scene';scene.setAttribute('aria-hidden','true');
    const shadow=document.createElement('span');shadow.className='pet-gallery-shadow';
    const task:AgentTask={id:pet.id,name:pet.name,color:pet.color,state:pet.state,source:'agent',isIntegration:true,steps:[],stepIndex:0};
    scene.append(shadow,createMiniBot(task,56,()=>cards.get(pet.id)?.pet.dancing??false,{outfit:()=>cards.get(pet.id)?.pet.outfit??pet.outfit}));
    const name=document.createElement('strong'),state=document.createElement('span'),action=document.createElement('span'),metrics=document.createElement('span');metrics.className='pet-gallery-metrics';state.className='pet-gallery-state';action.className='pet-gallery-action';
    button.append(scene,name,state,metrics,action);card={el:button,name,state,metrics,action,pet};cards.set(pet.id,card);
    button.onclick=()=>open(cards.get(pet.id)!.pet);button.oncontextmenu=e=>{e.preventDefault();if(pet.id==='integration_spotify')open(cards.get(pet.id)!.pet);else sessions(cards.get(pet.id)!.pet);};
    button.onpointermove=e=>{const r=button.getBoundingClientRect();button.style.setProperty('--tilt-x',`${-(e.clientY-r.y-r.height/2)/r.height*8}deg`);button.style.setProperty('--tilt-y',`${(e.clientX-r.x-r.width/2)/r.width*10}deg`);};
    button.onpointerleave=()=>{button.style.setProperty('--tilt-x','0deg');button.style.setProperty('--tilt-y','0deg');};grid.append(button);
   }
   if(grid.children[index]!==card.el)grid.insertBefore(card.el,grid.children[index]??null);index++;
   card.pet=pet;card.name.textContent=pet.name;
   const decision=pet.state==='approval'||pet.state==='question';
   const state=decision?'Precisa de ti':pet.state==='error'?'Precisa de atenção':pet.state==='ratelimit'?'Limite atingido':pet.busy?'A trabalhar':pet.state==='sleeping'?'Em descanso':'Disponível';
   const observed=pet.id==='integration_spotify'?null:petObservation(PetObservability.snapshot,pet.id),info=scopedSessionSummary(observed,!!PetObservability.error,PetPreferences.config?.pets[pet.id]?.provider);card.metrics.hidden=pet.id==='integration_spotify'||!observed&&!petCapabilities(pet.id).chat;
   card.metrics.textContent=info.summary;card.metrics.title=info.detail;card.metrics.dataset.stale=String(info.stale);
   const action=pet.id==='integration_spotify'?'Música e som':decision?'Rever pedido':pet.attention?'Ver aviso':'Abrir pet';
   card.state.textContent=pet.id==='integration_spotify'?'Música e som do PC':state;card.action.textContent=pet.id==='integration_spotify'?'Música e som →':pet.attention?`${action} ↗`:'Abrir pet →';
   card.el.dataset.attention=String(pet.attention);card.el.dataset.busy=String(pet.busy);card.el.style.setProperty('--pet-color',pet.color);card.el.setAttribute('aria-label',`${pet.name} · ${state} · ${pet.id==='integration_spotify'?'Controlos de áudio':card.metrics.hidden?'':info.summary} · ${action}`);
  }
 }};
 observePetObservability(visible,()=>view.sync(current));
 return view;
}
