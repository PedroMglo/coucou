/** An intent is only a UI lifetime guard. It never cancels or rolls back native work. */
export class SettingsRequestIntent {
 private sequence=0;private alive=true;
 constructor(private active:()=>boolean=()=>true){}
 begin(){return ++this.sequence;}
 invalidate(){this.sequence++;}
 current(ticket:number){return this.alive&&ticket===this.sequence&&this.active();}
 dispose(){this.alive=false;this.invalidate();}
}
