// A local presentation receipt may fold only the card that requested it.
// This token is UI context, never native session/decision authority.
let capture:(()=>string)|null=null;
export function registerCardPresentation(read:()=>string){capture=read;return()=>{if(capture===read)capture=null;};}
export function cardPresentation(){return capture?.();}
