import test from 'node:test';
import assert from 'node:assert/strict';
import {catalog,groups} from '../desktop/package-model.ts';

const item=(family,name,category,version='1.0.0')=>({id:family+'-'+version,family,name,category,version,status:'available',channel:'Release'});
test('category filter reclassifies cached compiler and embedded database entries',()=>{
 catalog.items=[item('go','Go','Language'),item('node','Node.js','Language'),item('sqlite','SQLite','Database'),item('mariadb','MariaDB','Database')];
 catalog.category='SDKs & compilers';
 assert.deepEqual(groups().map(g=>g.family),['go']);
 catalog.category='Embedded databases';
 assert.deepEqual(groups().map(g=>g.family),['sqlite']);
 catalog.category='Database servers';
 assert.deepEqual(groups().map(g=>g.family),['mariadb']);
 catalog.category='All';
});
test('search uses corrected category and preserves version selection and unknown products',()=>{
 catalog.items=[item('go','Go','Language'),item('go','Go','Language','2.0.0'),item('future','Future','Special tools')];
 catalog.category='All';catalog.selected={go:'go-1.0.0'};
 assert.equal(groups('compilers')[0]?.selected.id,'go-1.0.0');
 catalog.category='Special tools';
 assert.deepEqual(groups().map(g=>g.family),['future']);
 catalog.category='All';catalog.selected={};
});
