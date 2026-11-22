import {h} from '../views/dom';
import type {SettingsReviewActions,SettingsReviewPreview} from '../core/settings-review';
export function buildSettingsReview(actions:SettingsReviewActions,refresh:()=>Promise<void>){
 const message=h('p',{}),prepareButton=h('button',{type:'button','data-settings-io-review':'prepare'},'Rever os dados das preferências');
 const detail=h('div',{'data-settings-io-review':'detail'}),el=h('div',{'data-settings-io-review':'control'},message,prepareButton,detail);
 let epoch=0,pending:SettingsReviewPreview|null=null,inflight=false;
 function clear(){detail.replaceChildren();pending=null;}
 function show(preview:SettingsReviewPreview){
  clear();pending=preview;
  const blocks:[string,unknown][]=[['Origem e revisões',preview.origin],['Valores a adotar',preview.values],['Valores em utilização pelo backend',preview.runningValues],['Defaults incorporados',preview.builtInDefaults],['Pets, providers, modelos e destinos',preview.petsAndDestinations],['Referências possíveis de credenciais (presença e valores não consultados)',preview.credentialReferences]];
  detail.append(h('p',{},preview.message),h('p',{},`Âmbito: dados. Validade: ${new Date(preview.expiresAtMs).toLocaleString()}. As componentes ficam suspensas: ${preview.suspendedOperations.join(', ')}.`));
  for(const [label,value]of blocks)detail.append(h('details',{},h('summary',{},label),h('pre',{},JSON.stringify(value,null,2))));
  detail.append(h('p',{},`Campos com default nesta leitura: ${preview.defaultedFields.join(', ')||'nenhum'}. Campos não reconhecidos: ${preview.unknownFieldCount}${preview.unknownFields.length?` (${preview.unknownFields.join(', ')})`:''}. ${preview.willPreserveCurrentBytes?'Os bytes atuais são preservados.':'O original é retido antes de publicar a configuração recuperada.'}`));
  if(preview.priorRecoveryProvenanceUncertain)detail.append(h('p',{},'Houve recuperação anterior. A origem por campo pode não ser reconstituível; revê todos os valores efetivos e defaults apresentados.'));
  const all=h('input',{type:'checkbox','data-settings-io-review':'all-values'}),uncertainty=h('input',{type:'checkbox','data-settings-io-review':'uncertainty'}),suspended=h('input',{type:'checkbox','data-settings-io-review':'keep-suspended'}),recovered=h('input',{type:'checkbox','data-settings-io-review':'recovered'});
  const needsRecovered=preview.defaultedFields.length>0||preview.priorRecoveryProvenanceUncertain;
  detail.append(h('label',{},all,' Revi todos os valores, destinos e modelos apresentados.'),h('label',{},uncertainty,' Compreendo que efeitos anteriores podem continuar por confirmar e não serão repetidos.'),h('label',{},suspended,' Aprovo apenas os dados; as componentes continuam suspensas.'));
  if(needsRecovered)detail.append(h('label',{},recovered,' Aceito explicitamente os valores recuperados e defaults apresentados.'));
  const apply=h('button',{type:'button','data-settings-io-review':'apply'},'Aprovar apenas os dados'),cancel=h('button',{type:'button','data-settings-io-review':'cancel'},'Cancelar revisão');
  function update(){apply.disabled=inflight||!all.checked||!uncertainty.checked||!suspended.checked||(needsRecovered&&!recovered.checked);}
  for(const input of [all,uncertainty,suspended,recovered])input.addEventListener('change',update);
  apply.addEventListener('click',()=>{if(apply.disabled||pending!==preview)return;const own=++epoch;inflight=true;update();prepareButton.disabled=true;message.textContent='A publicar a revisão. Cancelar não desfaz uma publicação que já tenha ocorrido.';
   void actions.apply({token:preview.token,fingerprint:preview.fingerprint,reviewedAllValues:all.checked,understandPriorUncertainty:uncertainty.checked,keepComponentsSuspended:suspended.checked,acceptRecoveredValues:!needsRecovered||recovered.checked}).then(receipt=>{if(own===epoch){message.textContent=receipt.message;clear();}},error=>{if(own===epoch){message.textContent=String(error);clear();}}).finally(()=>{inflight=false;prepareButton.disabled=false;update();void refresh();});
  });
  cancel.addEventListener('click',()=>{if(pending!==preview)return;const own=++epoch;cancel.disabled=true;
   void actions.cancel({token:preview.token,fingerprint:preview.fingerprint}).then(()=>{if(own!==epoch)return;message.textContent='Revisão cancelada. Se a gravação já tinha começado, consulta o estado; o cancelamento não desfaz dados publicados.';clear();},error=>{if(own===epoch)message.textContent=String(error);}).finally(()=>{cancel.disabled=false;void refresh();});
  });
  detail.append(apply,cancel);update();
 }
 prepareButton.addEventListener('click',()=>{if(inflight)return;const own=++epoch;prepareButton.disabled=true;message.textContent='A capturar os dados e as revisões, sem consultar contas ou cofres.';
  void actions.prepare().then(preview=>{if(own===epoch){message.textContent='Revisão preparada; ainda não foi publicada nenhuma alteração.';show(preview);}},error=>{if(own===epoch){message.textContent=String(error);clear();}}).finally(()=>{if(own===epoch)prepareButton.disabled=false;});
 });
 return {el,setPhase:(phase:string)=>{prepareButton.hidden=['ready','writing','unavailable','platformNotTracked','unbound'].includes(phase);}};
}
