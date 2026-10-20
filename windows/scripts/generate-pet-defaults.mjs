// Compatibility snapshot only; production defaults resolve pets.factory.json.
// Run after reviewing a deliberate bundled-default change. Never reads HOME.
import {readFileSync,writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
const source=JSON.parse(readFileSync(new URL('../src-tauri/pets.factory.json',import.meta.url),'utf8'));
function merge(base,extra){const out=structuredClone(base);for(const[key,value]of Object.entries(extra)){if(['__proto__','prototype','constructor'].includes(key))throw Error('Invalid factory key');out[key]=value&&typeof value==='object'&&!Array.isArray(value)&&out[key]&&typeof out[key]==='object'&&!Array.isArray(out[key])?merge(out[key],value):structuredClone(value);}return out;}
const protectedKeys=['name','aliases','provider','permissions','localExecution'];
for(const recipe of [source.base,...Object.values(source.molds),...Object.values(source.extensions)]){
 if(recipe.preferences&&protectedKeys.some(k=>Object.hasOwn(recipe.preferences,k)))throw Error('Identity and access policy must be explicit per pet');
}
const pets=Object.fromEntries(Object.entries(source.pets).map(([id,pet])=>{
 if(!['llm','application'].includes(pet.mold))throw Error('Invalid mold');
 if(!['name','aliases','provider','permissions'].every(k=>Object.hasOwn(pet.preferences,k)))throw Error('Missing explicit identity/access policy');
 let value=merge(source.base,source.molds[pet.mold]).preferences;
 if(pet.extension){if(!Object.hasOwn(source.extensions,pet.extension))throw Error('Unknown extension');value=merge(value,source.extensions[pet.extension].preferences??{});}
 return[id,merge(value,pet.preferences)];
}));
const result=JSON.stringify({...source.configuration,pets},null,2)+'\n';
const path=fileURLToPath(new URL('../src-tauri/pets.default.json',import.meta.url));
if(process.argv.includes('--check')){
 const old=JSON.parse(readFileSync(path,'utf8'));
 if(JSON.stringify(old)!==JSON.stringify(JSON.parse(result))){
  // Compare canonical keys, not authoring order.
  const canonical=v=>v&&typeof v==='object'?Array.isArray(v)?v.map(canonical):Object.fromEntries(Object.keys(v).sort().map(k=>[k,canonical(v[k])])):v;
  if(JSON.stringify(canonical(old))!==JSON.stringify(canonical(JSON.parse(result))))throw Error('Pet default snapshot differs; review the factory change and regenerate.');
 }
 console.log('All pet defaults match factory');
}else{writeFileSync(path,result);console.log('Updated bundled compatibility snapshot');}
