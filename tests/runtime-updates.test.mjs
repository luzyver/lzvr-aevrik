import test from 'node:test';
import assert from 'node:assert/strict';
import * as model from '../desktop/package-model.ts';

const item=(id,version,status='available',family='node',channel='Release')=>({id,version,status,family,channel});
test('runtime update chooses a newer stable version and preserves installed selection',()=>{
 assert.equal(typeof model.availableUpdate,'function','Missing runtime update selection');
 const old=item('old','9.9.0','installed');
 const latest=item('new','10.0.0');
 const preview=item('preview','11.0.0-beta.1','available','node','Preview');
 assert.equal(model.availableUpdate([old,latest,preview]),latest);
 assert.equal(old.status,'installed');
 assert.equal(model.availableUpdate([old,item('older','9.8.0')]),undefined);
 assert.equal(model.availableUpdate([item('new','10.0.0','installed'),old]),undefined);
 assert.equal(model.availableUpdate([latest]),undefined);
});
test('runtime update comparison handles Java build and release suffixes numerically',()=>{
 assert.equal(typeof model.availableUpdate,'function');
 assert.equal(model.availableUpdate([item('a','21.0.9+9','installed','java'),item('b','21.0.10+7','available','java')])?.id,'b');
 assert.equal(model.availableUpdate([item('a','8.4.9-1','installed','php'),item('b','8.4.10-1','available','php')])?.id,'b');
});
