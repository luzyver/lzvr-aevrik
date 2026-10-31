import type {Backend,AppUpdate} from './bridge';
import {esc} from './ui';

export const runtimeUpdates={checking:false,message:'Check for new versions in the published catalog.',error:''};
export const appUpdates:{checking:boolean;installing:boolean;launched:boolean;error:string;result:AppUpdate|null}={checking:false,installing:false,launched:false,error:'',result:null};

export async function checkRuntimeUpdates(backend:Backend,reload:()=>Promise<void>,changed:()=>void) {
 if(runtimeUpdates.checking)return;
 runtimeUpdates.checking=true;runtimeUpdates.error='';changed();
 try {
  const result=await backend.refreshCatalog();
  await reload();
  runtimeUpdates.message=result.added?`${result.added} new ${result.added===1?'version':'versions'} added · ${result.total} versions in the catalog. Existing installations are unchanged.`:'Catalog checked. No new versions were added. Existing installations are unchanged.';
 } catch(error){runtimeUpdates.error=String(error);}
 finally {runtimeUpdates.checking=false;changed();}
}

export async function checkAppUpdate(backend:Backend,changed:()=>void) {
 if(appUpdates.checking||appUpdates.installing||appUpdates.launched)return;
 appUpdates.checking=true;appUpdates.error='';appUpdates.result=null;changed();
 try {appUpdates.result=await backend.checkAppUpdate();}
 catch(error){appUpdates.error=String(error);}
 finally {appUpdates.checking=false;changed();}
}

export async function installAppUpdate(backend:Backend,changed:()=>void) {
 if(appUpdates.checking||appUpdates.installing||appUpdates.launched||appUpdates.result?.status!=='available')return;
 appUpdates.installing=true;appUpdates.error='';changed();
 try {await backend.installAppUpdate(appUpdates.result.latestCommit);appUpdates.launched=true;}
 catch(error){appUpdates.error=String(error);}
 finally {appUpdates.installing=false;changed();}
}

export function appUpdateRow() {
 const {checking,installing,launched,error,result}=appUpdates;
 const busy=checking||installing||launched;
 const message=launched?'Opening Windows Setup. Aevrik will close and reopen after installation.':installing?'Downloading and verifying Windows Setup… Keep Aevrik open until the download finishes.':checking?'Checking the published release…':error||result?.message||'Check for updates on this build’s release channel.';
 return `<div class="settings-row app-update-row" id="app-update-section"><div><strong>Aevrik updates</strong>
 <p id="app-update-status" role="${error?'alert':'status'}" class="${error?'update-error':''}">${esc(message)}</p>
 ${result?.status==='available'?'<p>Stop services and finish runtime installations first. Aevrik will restart; your data folder will be kept.</p>':''}
 ${result?`<p>Channel: ${esc(result.channel)} · Installed: ${esc(result.currentVersion)} · Published: ${esc(result.latestVersion)}</p><p>Build: ${esc(result.currentCommit.slice(0,7)||'local')} → ${esc(result.latestCommit.slice(0,7)||'unknown')}</p>`:''}</div>
 <div class="update-actions"><button class="button" data-action="check-app-update" ${busy?'disabled':''}>${checking?'Checking…':'Check for updates'}</button>
 ${result?.status==='available'?`<button class="button primary" data-action="install-app-update" ${busy?'disabled':''}>${installing?'Downloading…':launched?'Opening Setup…':'Download &amp; install update'}</button>`:''}
 ${result&&['available','unknown'].includes(result.status)?`<button class="button" data-action="open-app-release" ${busy?'disabled':''}>Open release page</button>`:''}</div></div>`;
}
