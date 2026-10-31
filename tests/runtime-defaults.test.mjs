import test from 'node:test';
import assert from 'node:assert/strict';
import {Backend} from '../desktop/bridge.ts';

test('default changes require the exact reviewed plan, and errors propagate',async()=>{
 const calls=[];
 const plan={scope:'machine',fingerprint:'reviewed',removing:false};
 const backend=new Backend(async(command,args)=>{
  calls.push([command,args]);
  if(command==='preview_runtime_default')return plan;
  throw new Error('Run Aevrik as Administrator');
 });
 assert.equal(typeof backend.previewDefault,'function');
 assert.equal(await backend.previewDefault('node-22',false),plan);
 await assert.rejects(()=>backend.applyDefault('node-22',plan),/Administrator/);
 assert.deepEqual(calls,[
  ['preview_runtime_default',{id:'node-22',removing:false}],
  ['apply_runtime_default',{id:'node-22',scope:'machine',removing:false,fingerprint:'reviewed'}],
 ]);
});
