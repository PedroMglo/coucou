/** Give the visible row label to controls whose icon/switch has no own name.
 * Dynamic settings (phone pairing, models, profiles) use the same convention. */
export function labelControls(root:HTMLElement){
 for(const control of root.querySelectorAll<HTMLElement>('button,input,select,textarea,[role="button"]')){
  if(control.hasAttribute('aria-label')||control.hasAttribute('aria-labelledby')||control.title)continue;
  if(control.tagName==='BUTTON'&&control.textContent?.trim())continue;
  const row=control.closest('.row,.hub-row');
  const label=row?.querySelector<HTMLElement>(':scope > label,:scope > .shortcut-name,:scope > .text > .title');
  const name=label?.textContent?.trim();if(name)control.setAttribute('aria-label',name);
 }
}
