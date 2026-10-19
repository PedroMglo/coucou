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
 const cards=new Map<string,{el:HTMLButtonElement;name:HTMLElement;state:HTMLElement;action:HTMLElement;pet:PetIdentity}>();
 return {el,sync(pets:PetIdentity[]){
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
    const name=document.createElement('strong'),state=document.createElement('span'),action=document.createElement('span');state.className='pet-gallery-state';action.className='pet-gallery-action';
    button.append(scene,name,state,action);card={el:button,name,state,action,pet};cards.set(pet.id,card);
    button.onclick=()=>open(cards.get(pet.id)!.pet);button.oncontextmenu=e=>{e.preventDefault();sessions(cards.get(pet.id)!.pet);};
    button.onpointermove=e=>{const r=button.getBoundingClientRect();button.style.setProperty('--tilt-x',`${-(e.clientY-r.y-r.height/2)/r.height*8}deg`);button.style.setProperty('--tilt-y',`${(e.clientX-r.x-r.width/2)/r.width*10}deg`);};
    button.onpointerleave=()=>{button.style.setProperty('--tilt-x','0deg');button.style.setProperty('--tilt-y','0deg');};grid.append(button);
   }
   if(grid.children[index]!==card.el)grid.insertBefore(card.el,grid.children[index]??null);index++;
   card.pet=pet;card.name.textContent=pet.name;
   const state=pet.attention?'Precisa de ti':pet.busy?'A trabalhar':pet.state==='sleeping'?'Em descanso':'Disponível';
   card.state.textContent=state;card.action.textContent=pet.attention?'Rever pedido ↗':pet.sessions?`${pet.sessions} sessões · Abrir pet →`:'Abrir pet →';
   card.el.dataset.attention=String(pet.attention);card.el.dataset.busy=String(pet.busy);card.el.style.setProperty('--pet-color',pet.color);card.el.setAttribute('aria-label',`${pet.name} · ${state} · ${pet.attention?'Rever pedido':'Abrir pet'}`);
  }
 }};
}
