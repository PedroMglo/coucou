/** A failed write is actionable in every window, including the island.
 * Keep the explanation local to the current window and out of conversation logs. */
export function reportSettingsError(error:unknown){
 if(typeof document==='undefined')return;
 const id='settings-save-error';let banner=document.getElementById(id);
 if(!banner){
  banner=document.createElement('div');banner.id=id;banner.setAttribute('role','alert');
  banner.style.cssText='position:fixed;inset:8px 12px auto;z-index:2147483647;display:flex;align-items:center;gap:12px;padding:12px 14px;border:1px solid #fca5a5;border-radius:12px;background:#281b1f;color:#ffe4e6;font:12px/1.5 system-ui;box-shadow:0 4px 20px #0006;pointer-events:auto;max-height:calc(100vh - 16px);overflow:auto;box-sizing:border-box';
  const text=document.createElement('span');text.style.flex='1';banner.append(text);
  const dismiss=document.createElement('button');dismiss.type='button';dismiss.textContent='Fechar';dismiss.setAttribute('aria-label','Fechar aviso de erro ao guardar');dismiss.style.cssText='color:inherit;background:transparent;border:1px solid #ffffff40;padding:6px 9px;border-radius:6px;cursor:pointer';dismiss.onclick=()=>banner?.remove();banner.append(dismiss);document.body.append(banner);
 }
 banner.firstElementChild!.textContent=`Não foi possível guardar as definições. ${String(error).replace(/^Error:\s*/,'')}`;
}
