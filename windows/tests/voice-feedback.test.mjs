import {test} from 'node:test';
import assert from 'node:assert/strict';
import {voiceFeedback} from '../src/mochi/voice-feedback.ts';
test('a voice receipt survives waiting and belongs only to the Mochi conversation',()=>{
 const view={phase:'received',pet:'agent_codex',heard:'responde apenas olá',answer:''};
 const received=voiceFeedback(view,undefined,'Codex');
 assert.match(received.title,/Codex · Mochi · Pedido recebido/);assert.equal(received.text,'Ouvi: responde apenas olá');
 assert.equal(voiceFeedback(view,{phase:'working',message:'Resposta da tarefa anterior',changedAt:1},'Codex',2).text,received.text);
 const working=voiceFeedback(view,{phase:'working',message:''},'Codex');assert.match(working.title,/A executar/);assert.equal(working.text,received.text);
 assert.match(voiceFeedback(view,{phase:'approval',message:''},'Codex').title,/aprovação/);
 assert.equal(voiceFeedback(view,{phase:'done',message:'Olá'},'Codex').phase,'received');
});
test('listening and transcription provide feedback before a model request',()=>{
 assert.match(voiceFeedback({phase:'listening',heard:'',answer:''},undefined,'Codex').text,/nome do pet/);
 assert.match(voiceFeedback({phase:'transcribing',pet:'agent_codex',heard:'',answer:''},undefined,'Codex').title,/A transcrever/);
 assert.equal(voiceFeedback({phase:'speaking',pet:'agent_codex',heard:'olá',answer:'Olá!'},undefined,'Codex').text,'Olá!');
});
