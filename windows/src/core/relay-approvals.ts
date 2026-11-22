export type RelayApprovalMode='terminal'|'coucou';
export interface RelayApprovalStatus{mode:RelayApprovalMode;revision:string;scope:'processOnly';developmentBuild:boolean;restartMode:RelayApprovalMode}
export function parseRelayApprovalStatus(value:unknown):RelayApprovalStatus{
 const v=value as Partial<RelayApprovalStatus>|null;
 if(!v||typeof v!=='object'||!['terminal','coucou'].includes(v.mode??'')||!['terminal','coucou'].includes(v.restartMode??'')||v.scope!=='processOnly'||typeof v.developmentBuild!=='boolean'||typeof v.revision!=='string'||!(/^[1-9][0-9]{0,19}$/).test(v.revision)||BigInt(v.revision)>18446744073709551615n)throw Error('Não foi possível confirmar o encaminhamento das aprovações.');
 return {mode:v.mode!,revision:v.revision,scope:'processOnly',developmentBuild:v.developmentBuild,restartMode:v.restartMode!};
}
