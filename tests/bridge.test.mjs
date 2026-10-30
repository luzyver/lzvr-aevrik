import test from 'node:test';
import assert from 'node:assert/strict';
import { Backend } from '../desktop/bridge.ts';

test('snapshot failures propagate instead of returning sample running services', async()=>{
  const backend=new Backend(async()=>{throw new Error('database unavailable')});
  await assert.rejects(()=>backend.snapshot(),/database unavailable/);
});
test('service action waits for the backend and reads its authoritative snapshot', async()=>{
  const calls=[];
  const snapshot={projects:[],services:[{id:'s1',state:'stopped'}],logs:[],theme:'dark'};
  const backend=new Backend(async(command,args)=>{
    calls.push([command,args]);
    if(command==='snapshot')return snapshot;
  });
  assert.deepEqual(await backend.service('s1','stop'),snapshot);
  assert.deepEqual(calls,[['control_service',{id:'s1',action:'stop'}],['snapshot',undefined]]);
});
test('a rejected process action never refreshes into fake success', async()=>{
  const calls=[];
  const backend=new Backend(async command=>{calls.push(command);throw new Error('Port in use')});
  await assert.rejects(()=>backend.service('s1','start'),/Port in use/);
  assert.deepEqual(calls,['control_service']);
});

test('package installation propagates rejection and only sends a catalog id',async()=>{
 const calls=[];
 const backend=new Backend(async(command,args)=>{calls.push([command,args]);throw new Error('Checksum mismatch');});
 await assert.rejects(()=>backend.installPackage('caddy'),/Checksum mismatch/);
 assert.deepEqual(calls,[['install_package',{id:'caddy'}]]);
});

test('chosen destination is sent with the release ID and uninstall errors propagate',async()=>{
 const calls=[];
 const backend=new Backend(async(command,args)=>{calls.push([command,args]);if(command==='uninstall_package')throw new Error('Runtime is in use');});
 await backend.installPackage('caddy','D:\\Aevrik\\runtimes');
 assert.deepEqual(calls[0],['install_package',{id:'caddy',destination:'D:\\Aevrik\\runtimes'}]);
 await assert.rejects(()=>backend.uninstallPackage('caddy'),/in use/);
 assert.deepEqual(calls[1],['uninstall_package',{id:'caddy'}]);
});
