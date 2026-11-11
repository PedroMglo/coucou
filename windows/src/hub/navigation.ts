/** Preserve newer user/native navigation when startup lookup resolves late. */
export function createHubNavigation(
 read:()=>Promise<string|null>,revision:()=>number,apply:(page:string|null)=>Promise<void>,
){
 let pending=0;
 return async()=>{
  const ticket=++pending,original=revision(),page=await read();
  if(ticket!==pending||revision()!==original)return;
  await apply(page);
 };
}
