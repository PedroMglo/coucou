import {h} from './dom';
/** Present literal argv values, retaining the complete original request. */
export function commandReview(detail:string,local=false):HTMLElement{
 let value:unknown;try{value=local?JSON.parse(detail):null;}catch{/* Unstructured native requests remain readable verbatim. */}
 const spec=value as {program?:unknown;args?:unknown;cwd?:unknown;timeoutMs?:unknown;reason?:unknown}|null;
 if(!spec||typeof spec.program!=='string'||typeof spec.cwd!=='string'||!Array.isArray(spec.args)||!spec.args.every(a=>typeof a==='string'))return h('pre',{text:detail,style:'white-space:pre-wrap;max-height:120px;overflow:auto'});
 const fields=h('dl'),program=h('pre',{text:spec.program}),cwd=h('pre',{text:spec.cwd}),args=h('ol');
 for(const argument of spec.args)args.append(h('li',{},h('code',{text:JSON.stringify(argument)})));
 fields.append(h('dt',{text:'Programa'}),h('dd',{},program),h('dt',{text:'Argumentos literais'}),h('dd',{},spec.args.length?args:h('span',{text:'Sem argumentos'})),h('dt',{text:'Pasta de trabalho'}),h('dd',{},cwd));
 if(typeof spec.timeoutMs==='number'&&Number.isFinite(spec.timeoutMs))fields.append(h('dt',{text:'Tempo máximo'}),h('dd',{text:`${spec.timeoutMs} ms`}));
 const original=h('details',{},h('summary',{text:'Ver pedido completo'}),h('pre',{text:detail}));
 return h('div',{class:'command-review'},fields,...(typeof spec.reason==='string'?[h('p',{text:spec.reason})]:[]),original);
}
