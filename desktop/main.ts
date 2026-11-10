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
import {state,accept,pages,resolvedTheme,type Page,type Filter} from './state';
import {render} from './views';
import {hydrate,toast,esc} from './ui';
import {dialog,openForm,registration} from './forms';
import {catalog,PackageController} from './packages';
import {installDialog,uninstallDialog,removeServiceDialog,defaultDialog,defaultPlans} from './package-dialogs';
import {storage,loadStorage,openDataFolderDialog} from './storage';
import {checkRuntimeUpdates,checkAppUpdate,installAppUpdate,appUpdates} from './updates';
import {openNpmDialog} from './npm-dialog';
import {editProjectDialog,removeProjectDialog,editServiceDialog,exitDialog} from './edit-dialogs';
import {openPalette,type Command} from './palette';
import {logText} from './panels';
import type {OpenTarget} from './bridge';
import {appVersion} from './version';

const backend=new Backend(invoke);
const refreshStorage=()=>loadStorage(backend,()=>{if(state.page==='settings'&&!dialog.open)render();});
const packages=new PackageController(backend,()=>{if(state.page==='packages'&&!dialog.open)render();});
document.querySelector<HTMLImageElement>('.window-title img')!.src=markUrl;
document.querySelector('#app-version')!.textContent=`${appVersion} · Desktop`;
let polling=false;
let lastSnapshot='';
let propertiesDirty=false;
let defaultPreviewPending=false;
dialog.addEventListener('cancel',event=>{if(dialog.querySelector('[data-submitting="true"]'))event.preventDefault();});
const connection=document.querySelector<HTMLDivElement>('#connection-error')!;
function failure(error:unknown) {return error instanceof Error?error.message:String(error);}
async function refreshStartup() {
 try {state.startup=await backend.startupEnabled();} catch {state.startup=null;}
 if(state.page==='settings'&&!dialog.open)render();
}
function go(page:Page) {
 state.page=page;state.query='';state.filter='all';propertiesDirty=false;render();document.querySelector('#main')!.scrollTop=0;
 if(page==='packages')void packages.refresh();
 if(page==='settings'){void refreshStorage();void refreshStartup();}
}
async function controlAll(action:'start'|'stop') {
 let failures:string[]=[];
 await mutate(async()=>{failures=await backend.controlAll(action);return backend.snapshot();});
 toast(failures.length?`Could not start ${failures.join(' · ')}`:action==='start'?'All services started.':'All services stopped.');
}
async function exitApp() {
 const running=state.snapshot?.services.filter(s=>s.state==='running')??[];
 if(running.length)exitDialog(running);else await backend.exit();
}
// Tray Exit while services run: confirm here, against the engine's current state.
let exitPending=false;
async function requestExit() {
 if(dialog.open) {
  if(dialog.querySelector('#exit-confirm'))return;
  if(dialog.classList.contains('command-dialog')) {
   // The palette holds no unsaved input; close it so its keys stop applying.
   dialog.close();dialog.classList.remove('command-dialog');
  } else {
   // Another dialog may hold unsaved input: ask once it has closed.
   if(!exitPending){exitPending=true;dialog.addEventListener('close',()=>{exitPending=false;void requestExit();},{once:true});}
   return;
  }
 }
 try {accept(await backend.snapshot());connected();await exitApp();} catch(error) {toast(failure(error));}
}
window.addEventListener('aevrik-exit-requested',()=>void requestExit());
/** Everything reachable from Ctrl+K: pages, projects, services and bulk commands. */
function commands():Command[] {
 const snapshot=state.snapshot!;
 const guarded=(run:()=>void|Promise<void>)=>async()=>{try {await run();} catch(error) {toast(failure(error));}};
 const item=(label:string,hint:string,glyph:string,run:()=>void|Promise<void>):Command=>({label,hint,glyph,run:guarded(run)});
 const running=snapshot.services.filter(s=>s.state==='running').length;
 return [
  ...snapshot.projects.map(p=>item(p.name,`Project · ${p.domain}`,'folder',()=>{state.selectedProject=p.id;go('projects');})),
  ...snapshot.services.flatMap(s=>[
   item(s.name,`Service · ${s.state}`,'server',()=>{state.selectedService=s.id;go('services');}),
   item(`${s.state==='running'?'Stop':'Start'} ${s.name}`,'Command',s.state==='running'?'stop':'play',async()=>{await mutate(()=>backend.service(s.id,s.state==='running'?'stop':'start'));})
  ]),
  ...pages.map(p=>item(`Go to ${p.name}`,'Page',p.icon,()=>go(p.id))),
  item('Go to Settings','Page','settings',()=>go('settings')),
  item('Add project','Command','plus',()=>openForm('project')),
  item('Register service','Command','plus',()=>openForm('service')),
  ...(running<snapshot.services.length?[item('Start all services','Command','play',()=>controlAll('start'))]:[]),
  ...(running?[item('Stop all services','Command','stop',()=>controlAll('stop'))]:[]),
  item(`Use ${resolvedTheme(snapshot.theme)==='dark'?'pearl (light)':'graphite (dark)'} appearance`,'Command','sun',async()=>{await mutate(()=>backend.mutate('set_theme',{theme:resolvedTheme(snapshot.theme)==='dark'?'light':'dark'}));}),
  item('Exit and stop services','Command','close',exitApp)
 ];
}
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
   if(page==='settings'||pages.some(p=>p.id===page))go(page);
  }
  if(target.dataset.filter){state.filter=target.dataset.filter as Filter;render();}
  if(target.dataset.selectService){state.selectedService=target.dataset.selectService;render();}
  if(target.dataset.install||target.dataset.updatePackage){
   const item=catalog.items.find(p=>p.id===(target.dataset.install||target.dataset.updatePackage));
   if(item&&!dialog.open){const root=await backend.installRoot();if(!dialog.open)installDialog(item,root);}
  }
  if(target.dataset.uninstall){const item=catalog.items.find(p=>p.id===target.dataset.uninstall);if(item)uninstallDialog(item);}
  if((target.dataset.default||target.dataset.clearDefault)&&!defaultPreviewPending&&!dialog.open){
   const item=catalog.items.find(p=>p.id===(target.dataset.default||target.dataset.clearDefault));
   if(item){
    defaultPreviewPending=true;target.setAttribute('disabled','');
    try {const plan=await backend.previewDefault(item.id,!!target.dataset.clearDefault);if(!dialog.open)defaultDialog(item,plan);}
    finally {defaultPreviewPending=false;target.removeAttribute('disabled');}
   }
  }
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
  if(target.dataset.open&&target.dataset.id) {
   target.setAttribute('disabled','');
   try {await backend.open(target.dataset.open as OpenTarget,target.dataset.id);}
   finally {target.removeAttribute('disabled');}
  }
  const quick=target.dataset.quick;
  if(quick==='start'||quick==='stop') {
   await mutate(()=>backend.service(target.dataset.id!,quick));toast(`${quick==='start'?'Start':'Stop'} command completed.`);
  }
  const action=target.dataset.action;
  if(action==='exit-app'){await exitApp();return;}
  if(action==='start-all'||action==='stop-all')await controlAll(action==='start-all'?'start':'stop');
  if(action==='edit-project'){const project=state.snapshot.projects.find(p=>p.id===target.dataset.id);if(project)editProjectDialog(project);}
  if(action==='remove-project'){const project=state.snapshot.projects.find(p=>p.id===target.dataset.id);if(project)removeProjectDialog(project);}
  if(action==='edit-service'){const service=state.snapshot.services.find(s=>s.id===target.dataset.id);if(service)editServiceDialog(service);}
  if(action==='toggle-startup') {
   target.setAttribute('disabled','');
   try {state.startup=await backend.setStartup(target.getAttribute('aria-checked')!=='true');}
   finally {render();}
   toast(state.startup?'Aevrik will open in the system tray when you sign in.':'Aevrik will no longer start with Windows.');
  }
  if(action==='toggle-restore')await mutate(()=>backend.mutate('set_restore_services',{enabled:target.getAttribute('aria-checked')!=='true'}));
  if(action==='copy-logs'){await navigator.clipboard.writeText(logText());toast('Output copied to the clipboard.');}
  if(action==='setup-npm'){
   const project=state.snapshot.projects.find(p=>p.id===target.dataset.id);
   if(project)openNpmDialog(project,backend,mutate);
  }
  if(action==='unlink-project-service'){
   const form=document.querySelector<HTMLFormElement>('#project-properties');
   if(form&&!state.busy){(form.elements.namedItem('serviceId') as HTMLSelectElement).value='';form.requestSubmit();}
  }
  if(action==='check-runtime-updates')await checkRuntimeUpdates(backend,()=>packages.refresh(),()=>{if(state.page==='packages'&&!dialog.open)render();});
  if(action==='check-app-update')await checkAppUpdate(backend,()=>{if(state.page==='settings'&&!dialog.open)render();});
  if(action==='install-app-update')await installAppUpdate(backend,()=>{if(state.page==='settings'&&!dialog.open)render();});
  if(action==='open-app-release'&&appUpdates.result)await backend.openReleasePage(appUpdates.result.tag);
  if(action==='reload-storage')await refreshStorage();
  if(action==='change-data-folder')openDataFolderDialog();
  if(action==='cancel-data-change'&&storage.info){
   target.setAttribute('disabled','');
   try {storage.info=await backend.setDataDirectory(storage.info.current);render();}
   finally {target.removeAttribute('disabled');}
  }
  if(action==='remove-service'){const service=state.snapshot.services.find(s=>s.id===target.dataset.id);if(service)removeServiceDialog(service);}
  if(action==='browse-packages')go('packages');
  if(action==='reload-packages')await packages.refresh();
  if(action==='add-service')openForm('service');
  if(action==='add-project')openForm('project');
  if(action==='close-dialog')dialog.close();
  if(action==='search'&&!dialog.open)openPalette(commands());
  if(action==='start'||action==='stop'||action==='restart') {
   await mutate(()=>backend.service(target.dataset.id!,action));toast(`${action==='start'?'Start':action==='stop'?'Stop':'Restart'} command completed.`);
  }
  if(action==='theme')await mutate(()=>backend.mutate('set_theme',{theme:resolvedTheme(state.snapshot!.theme)==='dark'?'light':'dark'}));
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
 if(select.id==='theme-select')void mutate(()=>backend.mutate('set_theme',{theme:select.value})).catch(error=>toast(failure(error)));
 if(select.id==='log-service'){state.logService=select.value;render();}
 if(select.id==='package-category'){catalog.category=select.value;render();document.querySelector<HTMLElement>('#package-category')?.focus();}
 if(select.dataset.packageVersion){catalog.selected[select.dataset.packageVersion]=select.value;render();document.querySelector<HTMLElement>(`[data-package-version="${CSS.escape(select.dataset.packageVersion)}"]`)?.focus({preventScroll:true});}
});
document.addEventListener('submit',async event=>{
 const form=event.target as HTMLFormElement;
 if(!['registration','project-properties','package-install','package-uninstall','package-default','service-remove','data-location','project-edit','project-remove','service-edit','exit-confirm'].includes(form.id))return;
 event.preventDefault();
 if(state.busy||form.dataset.submitting==='true')return;
 const submit=form.querySelector<HTMLButtonElement>('[type="submit"]')!;submit.disabled=true;
 try {
  if(form.id==='package-default') {
   const plan=defaultPlans.get(form);
   if(!plan)throw new Error('Close this dialog and select the runtime again.');
   form.dataset.submitting='true';form.setAttribute('aria-busy','true');
   form.querySelectorAll<HTMLButtonElement>('button').forEach(b=>b.disabled=true);
   const error=form.querySelector('#form-error');if(error)error.textContent='';
   try {
    await backend.applyDefault(form.dataset.id!,plan);
    await packages.refresh();dialog.close();render();
    toast(plan.removing?'PATH entry removed. Restart your terminal or editor.':'Terminal default saved. Restart your terminal or editor.');
   } finally {
    delete form.dataset.submitting;form.removeAttribute('aria-busy');
    form.querySelectorAll<HTMLButtonElement>('button').forEach(b=>b.disabled=false);
   }
  } else if(form.id==='data-location') {
   storage.info=await backend.setDataDirectory(String(new FormData(form).get('directory')??'').trim());
   dialog.close();render();toast(storage.info.next?'Data folder saved. Choose Exit and stop services, then reopen Aevrik to apply it.':'Current data folder retained.');
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
  } else if(form.id==='exit-confirm') {
   await backend.exit();
  } else if(form.id==='project-edit') {
   const data=new FormData(form);const text=(name:string)=>String(data.get(name)??'').trim();
   await mutate(()=>backend.mutate('edit_project',{id:form.dataset.id,name:text('name'),path:text('path'),domain:text('domain')}));dialog.close();toast('Project saved.');
  } else if(form.id==='project-remove') {
   await mutate(()=>backend.mutate('remove_project',{id:form.dataset.id}));dialog.close();render();toast('Project removed. Files preserved.');
  } else if(form.id==='service-edit') {
   const service=state.snapshot!.services.find(s=>s.id===form.dataset.id);
   if(!service)throw new Error('Service no longer exists.');
   const input=registration(form).input;
   // An npm service keeps the launch settings its project script produced.
   if(service.npmScript)Object.assign(input,{executable:service.executable,cwd:service.cwd,args:service.args});
   await mutate(()=>backend.mutate('update_service',{id:service.id,input}));dialog.close();render();toast('Service registration saved.');
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
  if(state.snapshot)openPalette(commands());
 }
});
hydrate();
if(isTauri()) {
 void refresh(true);
 void packages.refresh();
 void refreshStartup();
 window.setInterval(()=>{if(!document.hidden&&(state.page==='packages'||catalog.pending.size))void packages.refresh();},750);
 window.setInterval(()=>{if(!document.hidden)void refresh();},1000);
 document.addEventListener('visibilitychange',()=>{if(!document.hidden)void refresh();});
} else {
 document.querySelector('#main')!.innerHTML=`<div class="empty"><strong>Open Aevrik as a desktop application</strong><p>This interface requires the Rust engine. Start it with <code>${esc('npm run desktop:dev')}</code>, or open the built Aevrik executable.</p></div>`;
 document.querySelector('#engine-status')!.textContent='Desktop engine unavailable';
 document.querySelector('#service-count')!.textContent='No engine connection';
}
