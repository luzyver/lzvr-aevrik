import '../styles/tokens.css';
import '../styles/layout.css';
import '../styles/components.css';
import '../styles/dialogs.css';
import '../styles/refinements.css';
import './native.css';
import markUrl from '../assets/mark.svg';
import {invoke,isTauri} from '@tauri-apps/api/core';
import {getCurrentWindow} from '@tauri-apps/api/window';
import {Backend} from './bridge';
import {state,accept,pages,type Page} from './state';
import {render} from './views';
import {hydrate,toast,esc} from './ui';
import {dialog,openForm,registration} from './forms';
import {catalog,PackageController} from './packages';
import {installDialog,uninstallDialog,removeServiceDialog} from './package-dialogs';
import {storage,loadStorage,openDataFolderDialog} from './storage';

const backend=new Backend(invoke);
const refreshStorage=()=>loadStorage(backend,()=>{if(state.page==='settings'&&!dialog.open)render();});
const packages=new PackageController(backend,()=>{if(state.page==='packages'&&!dialog.open)render();});
document.querySelector<HTMLImageElement>('.window-title img')!.src=markUrl;
let polling=false;
let lastSnapshot='';
let propertiesDirty=false;
const connection=document.querySelector<HTMLDivElement>('#connection-error')!;
function failure(error:unknown) {return error instanceof Error?error.message:String(error);}
function connected() {connection.hidden=true;}
async function refresh(force=false) {
 if(polling||state.busy)return;
 polling=true;
 try {
  const snapshot=await backend.snapshot();
  const serialized=JSON.stringify(snapshot);
  accept(snapshot);connected();
  if(force||serialized!==lastSnapshot) {
   if(!dialog.open&&!propertiesDirty){render();lastSnapshot=serialized;}
  }
 } catch(error) {
  const message=`Engine unavailable: ${failure(error)}`;
  connection.hidden=false;if(connection.textContent!==message)connection.textContent=message;
  document.querySelector('#engine-status')!.textContent='Engine disconnected';
 } finally {polling=false;}
}
async function mutate(operation:()=>ReturnType<Backend['snapshot']>) {
 if(state.busy)return;
 state.busy=true;
 const main=document.querySelector('#main')!;
 main.setAttribute('aria-busy','true');
 main.querySelectorAll<HTMLButtonElement>('button').forEach(b=>b.disabled=true);
 try {accept(await operation());connected();lastSnapshot='';propertiesDirty=false;render();}
 finally {state.busy=false;main.removeAttribute('aria-busy');render();}
}
document.addEventListener('click',async event=>{
 const target=(event.target as Element).closest<HTMLElement>('button');
 if(!target)return;
 try {
  if(target.dataset.window) {
   const win=getCurrentWindow();
   if(target.dataset.window==='minimize')await win.minimize();
   if(target.dataset.window==='maximize')await win.toggleMaximize();
   if(target.dataset.window==='close')await win.close();
   return;
  }
  if(!state.snapshot)return;
  if(target.dataset.nav) {
   const page=target.dataset.nav as Page;
   if(page==='settings'||pages.some(p=>p.id===page)){state.page=page;state.query='';propertiesDirty=false;render();document.querySelector('#main')!.scrollTop=0;}
  }
  if(target.dataset.selectService){state.selectedService=target.dataset.selectService;render();}
  if(target.dataset.nav==='packages')void packages.refresh();
  if(target.dataset.nav==='settings')void refreshStorage();
  if(target.dataset.install){
   const item=catalog.items.find(p=>p.id===target.dataset.install);
   if(item&&!dialog.open){const root=await backend.installRoot();if(!dialog.open)installDialog(item,root);}
  }
  if(target.dataset.uninstall){const item=catalog.items.find(p=>p.id===target.dataset.uninstall);if(item)uninstallDialog(item);}
  if(target.dataset.configure){
   const item=catalog.items.find(p=>p.id===target.dataset.configure);
   if(item?.path){
    openForm('service');
    const values={name:`${item.name} ${item.version}`,executable:item.path,cwd:item.path.slice(0,Math.max(item.path.lastIndexOf('\\'),item.path.lastIndexOf('/')))};
    for(const [key,value] of Object.entries(values))dialog.querySelector<HTMLInputElement>(`[name="${key}"]`)!.value=value;
   }
  }
  if(target.dataset.selectProject){state.selectedProject=target.dataset.selectProject;propertiesDirty=false;render();}
  if(target.dataset.pick) {
   target.setAttribute('disabled','');
   try {const path=await backend.pick(target.dataset.pick as 'directory'|'executable');
    if(path){const input=dialog.querySelector<HTMLInputElement>(`[name="${target.dataset.field}"]`);if(input)input.value=path;}
   } finally {target.removeAttribute('disabled');}
  }
  const action=target.dataset.action;
  if(action==='reload-storage')await refreshStorage();
  if(action==='change-data-folder')openDataFolderDialog();
  if(action==='cancel-data-change'&&storage.info){
   target.setAttribute('disabled','');
   try {storage.info=await backend.setDataDirectory(storage.info.current);render();}
   finally {target.removeAttribute('disabled');}
  }
  if(action==='remove-service'){const service=state.snapshot.services.find(s=>s.id===target.dataset.id);if(service)removeServiceDialog(service);}
  if(action==='browse-packages'){state.page='packages';state.query='';render();void packages.refresh();}
  if(action==='reload-packages')await packages.refresh();
  if(action==='add-service')openForm('service');
  if(action==='add-project')openForm('project');
  if(action==='close-dialog')dialog.close();
  if(action==='search') {
   if(!['projects','services','overview','packages'].includes(state.page)){state.page='services';render();}
   document.querySelector<HTMLInputElement>('#search')?.focus();
  }
  if(action==='start'||action==='stop'||action==='restart') {
   await mutate(()=>backend.service(target.dataset.id!,action));toast(`${action==='start'?'Start':action==='stop'?'Stop':'Restart'} command completed.`);
  }
  if(action==='theme')await mutate(()=>backend.mutate('set_theme',{theme:state.snapshot!.theme==='dark'?'light':'dark'}));
  if(action==='clear-logs')await mutate(()=>backend.mutate('clear_logs'));
 } catch(error) {toast(failure(error));}
});
document.addEventListener('input',event=>{
 const input=event.target as HTMLInputElement;
 if(input.id==='search'){state.query=input.value;render();}
 if(input.closest('#project-properties'))propertiesDirty=true;
});
document.addEventListener('change',event=>{
 const select=event.target as HTMLSelectElement;
 if(select.id==='package-category'){catalog.category=select.value;render();document.querySelector<HTMLElement>('#package-category')?.focus();}
 if(select.dataset.packageVersion){catalog.selected[select.dataset.packageVersion]=select.value;render();document.querySelector<HTMLElement>(`[data-package-version="${CSS.escape(select.dataset.packageVersion)}"]`)?.focus({preventScroll:true});}
});
document.addEventListener('submit',async event=>{
 const form=event.target as HTMLFormElement;
 if(!['registration','project-properties','package-install','package-uninstall','service-remove','data-location'].includes(form.id))return;
 event.preventDefault();
 if(state.busy)return;
 const submit=form.querySelector<HTMLButtonElement>('[type="submit"]')!;submit.disabled=true;
 try {
  if(form.id==='data-location') {
   storage.info=await backend.setDataDirectory(String(new FormData(form).get('directory')??'').trim());
   dialog.close();render();toast(storage.info.next?'Data folder saved. Close and reopen Aevrik to apply it.':'Current data folder retained.');
   document.querySelector('#storage-next')?.scrollIntoView({block:'nearest'});
  } else if(form.id==='package-install') {
   const destination=String(new FormData(form).get('destination')??'').trim();
   dialog.close();
   try {await packages.install(form.dataset.id!,destination);toast('Package installed. Configure a service when you are ready.');}
   catch(error){toast(failure(error));}
   render();
  } else if(form.id==='package-uninstall') {
   await packages.uninstall(form.dataset.id!);dialog.close();render();toast('Runtime uninstalled.');
  } else if(form.id==='service-remove') {
   await mutate(()=>backend.mutate('remove_service',{id:form.dataset.id}));dialog.close();render();toast('Service registration removed. Files preserved.');
  } else if(form.id==='registration'){
   const request=registration(form);
   await mutate(()=>backend.mutate(request.command,{input:request.input}));dialog.close();toast(form.dataset.kind==='service'?'Service registered. Select Start to launch it.':'Project registered.');
  } else {
   const data=new FormData(form);
   await mutate(()=>backend.mutate('update_project',{id:form.dataset.id,serviceId:data.get('serviceId')||null,runtime:data.get('runtime')||''}));toast('Project properties saved.');
  }
 } catch(error) {
  const node=form.querySelector('#form-error');
  if(node)node.textContent=failure(error);else toast(failure(error));
 } finally {submit.disabled=false;}
});
document.addEventListener('keydown',event=>{
 if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){
  event.preventDefault();if(dialog.open)return;
  document.querySelector<HTMLButtonElement>('[data-action="search"]')!.click();
 }
});
hydrate();
if(isTauri()) {
 void refresh(true);
 void packages.refresh();
 window.setInterval(()=>{if(!document.hidden&&(state.page==='packages'||catalog.pending.size))void packages.refresh();},750);
 window.setInterval(()=>{if(!document.hidden)void refresh();},1000);
 document.addEventListener('visibilitychange',()=>{if(!document.hidden)void refresh();});
} else {
 document.querySelector('#main')!.innerHTML=`<div class="empty"><strong>Open Aevrik as a desktop application</strong><p>This interface requires the Rust engine. Start it with <code>${esc('npm run desktop:dev')}</code>, or open the built Aevrik executable.</p></div>`;
 document.querySelector('#engine-status')!.textContent='Desktop engine unavailable';
 document.querySelector('#service-count')!.textContent='No engine connection';
}
