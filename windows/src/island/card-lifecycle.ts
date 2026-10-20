import type {FsmState} from './fsm';

// The native pet and card surfaces report their pointers separately. Allow one
// 33 ms sample to cross the small gap between them without closing the card.
const MANUAL_POINTER_GRACE_MS=50;

/** Explicit cards have an open/closed lifetime, with no wake or compact state. */
export class CardLifecycle {
 state:FsmState='hidden';
 onTransition:((from:FsmState,to:FsmState)=>void)|null=null;
 pinned=false;
 openOnHover=false;
 petitToHiddenDelay=0;
 homeCollapseDueAt:number|null=null;
 private delay=15;
 private timer:number|null=null;
 private keyboardEditing=false;
 private manualOpen=false;
 private petInside=false;
 private cardInside=false;
 constructor(private readonly held:()=>boolean=()=>false){}
 get manual(){return this.manualOpen;}
 setManual(value:boolean,petInside=false,cardInside=false){this.cancelTimers();this.manualOpen=value;this.petInside=petInside;this.cardInside=cardInside;}
 setPetInside(value:boolean){this.petInside=value;if(value)this.cancelTimers();else if(this.manualOpen&&!this.cardInside)this.mouseLeft();}
 refreshHold(){if(this.manualOpen&&!this.petInside&&!this.cardInside)this.mouseLeft();}
 get homeToPetitDelay(){return this.delay;}
 set homeToPetitDelay(value:number){
  if(!Number.isFinite(value)||value<0||value===this.delay)return;
  this.delay=value;if(this.timer!==null)this.mouseLeft();
 }
 get editing(){return this.keyboardEditing;}
 set editing(value:boolean){this.keyboardEditing=value;if(value&&!this.manualOpen)this.cancelTimers();}
 launch(){}
 reveal(){}
 click(){}
 greetComplete(){}
 userInteracted(){if(this.state==='home')this.cancelTimers();}
 mouseEntered(){this.cardInside=true;if(this.state==='home')this.cancelTimers();}
 mouseLeft(){
  this.cardInside=false;
  // Repeated outside samples must not postpone a close already in progress.
  if(this.manualOpen&&this.timer!==null)return;
  this.cancelTimers();
  if(this.state!=='home'||this.pinned||this.held()||this.manualOpen&&this.petInside||!this.manualOpen&&this.editing)return;
  const milliseconds=this.manualOpen?MANUAL_POINTER_GRACE_MS:this.delay*1000;
  this.homeCollapseDueAt=performance.now()+milliseconds;
  this.timer=window.setTimeout(()=>{
   this.timer=null;this.homeCollapseDueAt=null;
   if(!this.pinned&&!this.held()&&!(this.manualOpen?this.petInside||this.cardInside:this.editing))this.forceHidden();
  },milliseconds);
 }
 forceHome(){this.cancelTimers();this.transition('home');}
 forcePetit(){this.forceHidden();}
 forceHidden(){this.cancelTimers();this.manualOpen=false;this.petInside=this.cardInside=false;this.transition('hidden');}
 cancelTimers(){if(this.timer!==null)window.clearTimeout(this.timer);this.timer=null;this.homeCollapseDueAt=null;}
 private transition(next:'home'|'hidden'){
  if(this.state===next)return;const from=this.state;this.state=next;this.onTransition?.(from,next);
 }
}
