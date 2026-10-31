import type {Backend,StorageInfo} from './bridge';
import {dialog} from './forms';
import {esc,icon} from './ui';

export const storage:{info:StorageInfo|null;error:string;loading:boolean}={info:null,error:'',loading:false};

export async function loadStorage(backend:Backend,changed:()=>void) {
 if(storage.loading)return;
 storage.loading=true;storage.error='';changed();
 try {storage.info=await backend.storage();}
 catch(error){storage.info=null;storage.error=String(error);}
 finally {storage.loading=false;changed();}
}

export function storageRows() {
 if(storage.loading)return '<div class="settings-row" id="storage-section"><p role="status">Loading storage settings…</p></div>';
 if(storage.error)return `<div class="settings-row" id="storage-section"><div><strong>Data folder unavailable</strong><p role="alert">${esc(storage.error)}</p></div><button class="button" data-action="reload-storage">Retry</button></div>`;
 const info=storage.info;
 if(!info)return '<div class="settings-row" id="storage-section"><p>Storage settings have not loaded.</p><button class="button" data-action="reload-storage">Load settings</button></div>';
 return `<div class="settings-row"><div><strong>Application folder</strong><code class="runtime-path" id="storage-application">${esc(info.application)}</code><p>Choose this location in Windows Setup.</p></div></div>
 <div class="settings-row storage-row" id="storage-section"><div><strong>Data folder</strong><code class="runtime-path" id="storage-current">${esc(info.current)}</code><p>Settings, service registrations, browser cache and the default runtime folder.</p>
 ${info.overridden?'<p>Controlled by AEVRIK_DATA_DIR. Remove the override to change the saved folder.</p>':''}
 ${info.next?`<div class="storage-pending" id="storage-next" role="status"><strong>Folder selected for next launch</strong><code class="runtime-path">${esc(info.next)}</code><p>Close and reopen Aevrik to use it. Existing files stay where they are.</p><button class="button" data-action="cancel-data-change">Keep current folder</button></div>`:''}
 </div><button class="button" data-action="change-data-folder" ${info.overridden?'disabled':''}>Change folder…</button></div>`;
}

export function openDataFolderDialog() {
 const info=storage.info;
 if(!info||info.overridden||dialog.open)return;
 dialog.innerHTML=`<form id="data-location"><header class="dialog-header"><h2 id="dialog-title">Change data folder</h2><button class="icon-button" type="button" data-action="close-dialog" aria-label="Close">${icon('close')}</button></header>
 <div class="dialog-body"><p>Choose an empty folder for a fresh workspace, or an existing Aevrik data folder to reopen it.</p>
 <label class="field">Data folder<span class="path-picker"><input name="directory" value="${esc(info.next??info.current)}" required autocomplete="off" spellcheck="false"><button class="button" type="button" data-pick="directory" data-field="directory">Browse…</button></span></label>
 <p class="notice">Applies after you close and reopen Aevrik. Existing files stay where they are; this does not move databases, projects or installed runtimes. Select the old folder again to return to that workspace.</p>
 <p id="form-error" class="form-error" role="alert"></p></div><footer class="dialog-footer"><button type="button" class="button" data-action="close-dialog">Cancel</button><button type="submit" class="button primary">Use on next launch</button></footer></form>`;
 dialog.showModal();
}
