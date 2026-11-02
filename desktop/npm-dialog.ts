import type {Backend,Project,ProjectScripts} from './bridge';
import {dialog} from './forms';
import {esc,icon,toast} from './ui';

export function openNpmDialog(project:Project,backend:Backend,save:(operation:()=>ReturnType<Backend['snapshot']>)=>Promise<void>) {
 if(dialog.open)return;
 dialog.innerHTML=`<form id="npm-service"><header class="dialog-header"><h2 id="dialog-title">Set up npm script</h2><button class="icon-button" type="button" data-npm-close aria-label="Close">${icon('close')}</button></header>
 <div class="dialog-body"><p>Create a service for ${esc(project.name)}. Save it here, then press Start from the project.</p>
 <label class="field">Project folder<code class="detail-path">${esc(project.path)}</code></label>
 <p data-npm-status role="status">Reading package.json…</p>
 <label class="field">Script<select name="script" required disabled aria-describedby="npm-script-preview"></select></label>
 <code id="npm-script-preview" class="detail-path" data-script-preview></code>
 <label class="field">Node.js<span class="path-picker"><select name="nodeExecutable" required disabled></select><button class="button" type="button" data-npm-browse>Browse…</button></span><small data-node-path></small></label>
 <label class="field">Expected TCP port (optional)<input name="port" type="number" min="1" max="65535" placeholder="4321"><small>Checks availability; your script configures the listening port.</small></label>
 <p>Start runs the selected script and its npm pre/post hooks. Dependencies must already be installed.</p>
 <p class="form-error" role="alert"></p><button class="button" type="button" data-npm-reload>Reload scripts</button></div>
 <footer class="dialog-footer"><button class="button" type="button" data-npm-close>Cancel</button><button class="button primary" type="submit" disabled>Save service</button></footer></form>`;
 const form=dialog.querySelector<HTMLFormElement>('#npm-service')!;
 const script=form.elements.namedItem('script') as HTMLSelectElement;
 const node=form.elements.namedItem('nodeExecutable') as HTMLSelectElement;
 const submit=form.querySelector<HTMLButtonElement>('[type="submit"]')!;
 const error=form.querySelector<HTMLElement>('[role="alert"]')!;
 const status=form.querySelector<HTMLElement>('[data-npm-status]')!;
 let info:ProjectScripts={scripts:[],nodes:[]};let generation=0;let saving=false;let loading=false;
 const active=()=>dialog.open&&form.isConnected;
 const update=()=>{
  form.querySelector('[data-script-preview]')!.textContent=info.scripts.find(s=>s.name===script.value)?.command??'';
  form.querySelector('[data-node-path]')!.textContent=node.value;
  submit.disabled=saving||loading||!script.value||!node.value;
 };
 const load=async()=>{
  const request=++generation;loading=true;script.disabled=true;node.disabled=true;update();error.textContent='';status.textContent='Reading package.json…';
  try {
   const result=await backend.projectScripts(project.id);
   if(!active()||request!==generation)return;
   info=result;
   script.innerHTML=info.scripts.map(s=>`<option value="${esc(s.name)}">${esc(s.name)} — npm run ${esc(s.name)}</option>`).join('');
   script.value=info.scripts.find(s=>s.name==='dev')?.name??info.scripts.find(s=>s.name==='start')?.name??info.scripts[0]?.name??'';
   node.innerHTML=info.nodes.map(n=>`<option value="${esc(n.executable)}">${esc(n.label)}</option>`).join('');
   script.disabled=!info.scripts.length;node.disabled=!info.nodes.length;
   status.textContent=!info.scripts.length?'No scripts found in package.json. Add a script in your project, then reload.':!info.nodes.length?'No Node.js installation with npm found. Install Node.js from Runtimes or browse to node.exe.':'Choose the script and Node.js version for this service.';
  } catch(e) {if(active()&&request===generation){error.textContent=String(e);status.textContent='Could not load project scripts.';script.innerHTML='';}}
  finally {if(active()&&request===generation){loading=false;update();}}
 };
 form.addEventListener('change',update);
 form.addEventListener('click',async event=>{
  const target=(event.target as Element).closest<HTMLButtonElement>('button');
  if(!target||saving)return;
  if(target.hasAttribute('data-npm-close')){generation++;dialog.close();}
  if(target.hasAttribute('data-npm-reload'))void load();
  if(target.hasAttribute('data-npm-browse')){
   target.disabled=true;
   try {
    const path=await backend.pick('executable');
    if(active()&&path){node.add(new Option(path,path));node.value=path;node.disabled=false;update();}
   }catch(e){if(active())error.textContent=String(e);}
   finally{if(active())target.disabled=false;}
  }
 });
 form.addEventListener('submit',async event=>{
  event.preventDefault();event.stopPropagation();
  if(saving||submit.disabled)return;
  const data=new FormData(form);
  saving=true;form.dataset.submitting='true';error.textContent='';
  form.querySelectorAll<HTMLInputElement|HTMLSelectElement|HTMLButtonElement>('input,select,button').forEach(el=>el.disabled=true);
  try {
   await save(()=>backend.registerNpmService({projectId:project.id,script:String(data.get('script')),nodeExecutable:String(data.get('nodeExecutable')),port:data.get('port')?Number(data.get('port')):null}));
   dialog.close();toast('npm service saved and linked. Press Start to run it.');
  }catch(e){error.textContent=String(e);}
  finally {
   saving=false;delete form.dataset.submitting;
   form.querySelectorAll<HTMLInputElement|HTMLSelectElement|HTMLButtonElement>('input,select,button').forEach(el=>el.disabled=false);update();
  }
 });
 dialog.showModal();void load();
}
