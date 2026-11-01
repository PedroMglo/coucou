import {h} from '../views/dom';
import {buildSettingsReview} from './io-review';
import type {SettingsReviewActions} from '../core/settings-review';
import {t} from '../i18n/i18n';
import type {SettingsIoStatus} from '../core/settings-io';

/** The button only reads status. It never reloads, saves or repeats an effect. */
export function buildSettingsIoStatus(read:()=>Promise<SettingsIoStatus>,reviewActions?:SettingsReviewActions){
 const message=h('p',{}),refreshButton=h('button',{type:'button'},t("Refresh"));
 const el=h('div',{className:'setting-note',role:'status','data-settings-io':'pending'},message,refreshButton);
 const review=reviewActions?buildSettingsReview(reviewActions,refresh):null;if(review)el.append(review.el);
 let epoch=0;
 async function refresh(){
  const own=++epoch;refreshButton.disabled=true;
  try{const status=await read();if(own!==epoch)return;
   el.dataset.settingsIo=status.phase;message.textContent=status.message;el.hidden=status.newEffectsAllowed&&!(status.suspendedOperations?.length);review?.setPhase(status.phase);
  }catch(error){if(own!==epoch)return;el.dataset.settingsIo='unavailable';message.textContent=String(error);el.hidden=false;}
  finally{if(own===epoch)refreshButton.disabled=false;}
 }
 refreshButton.addEventListener('click',()=>{void refresh();});
 return {el,refresh};
}
