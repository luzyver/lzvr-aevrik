import {state,pages,type Filter} from './state';
import {esc,icon,button,badge,empty,header,hydrate,iconButton} from './ui';
import {servicePanel,projectPanel,output,quickControl,browserButton} from './panels';
import {packagesView} from './packages';
import {storageRows} from './storage';
import {appUpdateRow} from './updates';
import {appVersion} from './version';
let renderedView='';
let lostFocus='';
const matches=(...values:string[])=>values.join(' ').toLowerCase().includes(state.query.toLowerCase());
const shown=(item:{state:string})=>state.filter==='all'||item.state===state.filter;
/** Start all / Stop all, offered only when they would change something. */
function bulkControls() {
 const all=state.snapshot!.services;const running=all.filter(s=>s.state==='running').length;
 return `<span class="bulk-actions">${running<all.length?button('start-all','Start all','play'):''}${running?button('stop-all','Stop all','stop'):''}</span>`;
}
/** Title, status filters with live counts, name filter, bulk controls and the page's primary action. */
function workHeader(title:string,items:{state:string}[],primary:string) {
 const filters:[Filter,string,string][]=[['all','All',''],['running','Running','<i class="dot active-dot"></i>'],['exited','Exited','<i class="dot failed-dot"></i>'],['stopped','Stopped','<i class="dot"></i>']];
 return `<header class="page-header work-header"><h1>${title}</h1><div class="filters" role="group" aria-label="Filter by status">${filters.map(([id,name,dot])=>`<button class="pill ${state.filter===id?'active':''}" data-filter="${id}" aria-pressed="${state.filter===id}">${dot}${name}<span>${id==='all'?items.length:items.filter(i=>i.state===id).length}</span></button>`).join('')}</div>
 <div class="header-actions"><label class="search-field">${icon('search')}<input id="search" type="search" placeholder="Filter by name…" value="${esc(state.query)}" aria-label="Filter by name"></label>${bulkControls()}${primary}</div></header>`;
}
const command=(s:{executable:string;args:string[]})=>esc([s.executable,...s.args].join(' '));
function services() {
 const all=state.snapshot!.services;const filtered=all.filter(s=>shown(s)&&matches(s.name,s.executable));
 return workHeader('Services',all,button('add-service','Register service','plus',true))+
 `<div class="split-view"><div class="work-pane"><section class="list-pane">
 ${filtered.length?`<div class="table-wrap"><table class="service-table"><thead><tr><th>Name</th><th>Status</th><th>Port</th><th>PID</th><th class="command-cell">Command</th><th><span class="sr-only">Actions</span></th></tr></thead><tbody>${filtered.map(s=>`<tr class="${s.id===state.selectedService?'selected':''}"><td><button class="service-select" data-select-service="${s.id}"><strong>${esc(s.name)}</strong></button></td><td class="status-cell">${badge(s.state,s.exitCode)}</td><td class="port-cell"><code>${s.port??'—'}</code></td><td class="pid-cell"><code>${s.pid??'—'}</code></td><td class="command-cell"><code title="${command(s)}">${command(s)}</code></td><td class="actions-cell"><span class="cell-actions">${browserButton(s)}${quickControl(s)}</span></td></tr>`).join('')}</tbody></table></div>`:
 empty(all.length?'No matching services':'Your local stack starts here',all.length?'Try another name or status filter.':'Install a runtime package or register an executable already on your computer.',all.length?'':button('browse-packages','Browse packages','package',true))}
 </section>${output(true,state.selectedService)}</div>${servicePanel(all.find(s=>s.id===state.selectedService))}</div>`;
}
function projects() {
 const all=state.snapshot!.projects;const filtered=all.filter(p=>shown(p)&&matches(p.name,p.path,p.domain));
 const linked=(id:string|null)=>state.snapshot!.services.find(s=>s.id===id);
 const slot='<span class="icon-slot"></span>';
 // The dock and the properties panel follow the same project, even while a filter hides its row.
 const selected=all.find(p=>p.id===state.selectedProject);
 return workHeader('Projects',all,button('add-project','Add project','plus',true))+
 `<div class="split-view"><div class="work-pane"><section class="list-pane">${filtered.length?`<div class="table-head" aria-hidden="true"><span class="project-grid"><span>Name</span><span>Status</span><span class="project-runtime">Runtime</span><span>Port</span><span class="project-path">Folder</span></span><span class="cell-actions"></span></div>`+filtered.map(p=>{
  const service=linked(p.serviceId);const id=`data-id="${esc(p.id)}"`;
  return `<div class="project-row ${p.id===state.selectedProject?'selected':''}"><button class="project-select project-grid" data-select-project="${p.id}"><span class="project-name"><strong>${esc(p.name)}</strong><small>${esc(p.domain)}</small></span>${badge(p.state,service?.exitCode??null)}<span class="project-runtime">${p.runtime?`<span class="chip">${esc(p.runtime)}</span>`:'—'}</span><code>${service?.port??'—'}</code><span class="project-path" title="${esc(p.path)}">${esc(p.path)}</span></button><span class="cell-actions">${(service&&browserButton(service))||slot}${iconButton(`data-open="project-folder" ${id}`,`Open ${p.name} folder`,'folder')}${iconButton(`data-open="project-terminal" ${id}`,`Open terminal in ${p.name}`,'terminal')}${service?quickControl(service):slot}</span></div>${service?`<div class="service-sub"><i class="dot ${service.state==='running'?'active-dot':service.state==='exited'?'failed-dot':''}"></i><span class="sub-name">${esc(service.name)}</span>${service.pid?`<span>PID ${service.pid}</span>`:''}<code class="sub-command" title="${command(service)}">${command(service)}</code></div>`:''}`;
 }).join(''):empty(all.length?'No matching projects':'Make room for your next project',all.length?'Try another name or status filter.':'Add an existing project folder. Its files stay where they are.',all.length?'':button('add-project','Add project','plus',true))}</section>${output(true,selected?.serviceId??'')}</div>${projectPanel(selected)}</div>`;
}
const toggle=(action:string,label:string,checked:boolean|null)=>`<button class="switch" type="button" role="switch" aria-checked="${checked===true}" aria-label="${esc(label)}" data-action="${action}" ${checked===null?'disabled':''}></button>`;
function settings() {
 const theme=state.snapshot!.theme;
 return header('Settings','Preferences for this computer.')+`<div class="settings-grid"><div class="panel"><div class="settings-row"><div><strong>Appearance</strong><p>Follow Windows, or always use graphite or pearl. Your preference is saved locally.</p></div><select id="theme-select" aria-label="Appearance">${[['system','Follow Windows'],['dark','Graphite'],['light','Pearl']].map(([value,name])=>`<option value="${value}" ${theme===value?'selected':''}>${name}</option>`).join('')}</select></div>
 ${storageRows()}
 ${appUpdateRow()}
 <div class="settings-row"><div><strong>Start with Windows</strong><p>Open Aevrik in the system tray when you sign in to this Windows account.${state.startup===null?' This setting could not be read.':''}</p></div>${toggle('toggle-startup','Start with Windows',state.startup)}</div>
 <div class="settings-row"><div><strong>Restore services</strong><p>When Aevrik opens, start the services that were running when it last exited.</p></div>${toggle('toggle-restore','Restore services',state.snapshot!.restoreServices===true)}</div>
 <div class="settings-row"><div><strong>When the window closes</strong><p>Aevrik stays in the system tray. Services and projects keep running. Click its tray icon to reopen the window.</p></div><span class="badge">Keep running</span></div>
 <div class="settings-row"><div><strong>Exit Aevrik</strong><p>Exit stops all managed processes and their children. Shut down database services using their own tools first.</p></div>${button('exit-app','Exit and stop services','close')}</div>
 </div>
 <p class="page-footnote">Aevrik ${esc(appVersion)} · Rust engine · Tauri desktop shell</p></div>`;
}
/** A selector that finds the same control again after the page is rebuilt. */
function focusKey(node:Element|null):string {
 if(!(node instanceof HTMLElement)||!node.closest('#main'))return '';
 if(node.id)return `#${CSS.escape(node.id)}`;
 const attribute=['data-control','data-filter','data-select-service','data-select-project','data-open','data-action','data-uninstall','data-default','data-clear-default','data-update-package','name'].find(a=>node.hasAttribute(a));
 if(!attribute)return node.classList.contains('log-lines')?'.log-lines':'';
 const scope=node.closest('.detail-pane')?'.detail-pane ':node.closest('.list-pane')?'.list-pane ':'';
 return `${scope}[${attribute}="${CSS.escape(node.getAttribute(attribute)!)}"]${node.dataset.id&&attribute!=='data-control'?`[data-id="${CSS.escape(node.dataset.id)}"]`:''}`;
}
export function render() {
 if(!state.snapshot)return;
 const main=document.querySelector<HTMLElement>('#main')!;
 const scroll=main.scrollTop;const log=main.querySelector('.log-lines');const logScroll=log?.scrollTop??0;
 // Follow new output unless the user scrolled up to read earlier lines.
 const following=!log||log.scrollHeight-log.scrollTop-log.clientHeight<24;
 const viewKey=[state.page,state.query,state.selectedService,state.selectedProject,state.logService].join(':');
 const paneScroll=viewKey===renderedView?Array.from(main.querySelectorAll<HTMLElement>('.catalog-scroll,.list-pane,.detail-pane,.settings-grid')).map(p=>({className:p.className,top:p.scrollTop})):[];
 renderedView=viewKey;
 const active=document.activeElement as HTMLInputElement|null;
 const focus=focusKey(active)||lostFocus;const start=active?.selectionStart;const end=active?.selectionEnd;
 const focusedPackage=active?.dataset.install??active?.dataset.configure;
 const focusedVersion=active?.dataset.packageVersion;
 // The tab bar is rebuilt only when it changes, and keeps keyboard focus when it is.
 const navigation=document.querySelector<HTMLElement>('#navigation')!;
 const focusedNav=navigation.contains(active)?active!.dataset.nav:undefined;
 const counts:Partial<Record<string,number>>={projects:state.snapshot.projects.length,services:state.snapshot.services.length};
 const navigationKey=`${state.page}:${counts.projects}:${counts.services}`;
 if(navigation.dataset.page!==navigationKey) {
  navigation.innerHTML=pages.map(p=>`<button class="nav-item ${p.id===state.page?'active':''}" data-nav="${p.id}" ${p.id===state.page?'aria-current="page"':''}>${p.name}${counts[p.id]===undefined?'':`<span class="nav-count">${counts[p.id]}</span>`}</button>`).join('');
  navigation.dataset.page=navigationKey;
  if(focusedNav)navigation.querySelector<HTMLElement>(`[data-nav="${CSS.escape(focusedNav)}"]`)?.focus({preventScroll:true});
 }
 document.querySelector('[data-nav="settings"]')!.classList.toggle('active',state.page==='settings');
 document.querySelector('#breadcrumb')!.textContent=state.page==='settings'?'Settings':pages.find(p=>p.id===state.page)!.name;
 main.innerHTML=state.page==='services'?services():state.page==='packages'?packagesView():state.page==='settings'?settings():state.page==='logs'?header('Output','Live stdout and stderr from managed processes.')+output():projects();
 main.scrollTop=scroll;
 for(const pane of main.querySelectorAll<HTMLElement>('.catalog-scroll,.list-pane,.detail-pane,.settings-grid'))pane.scrollTop=paneScroll.find(p=>p.className===pane.className)?.top??0;
 const newLog=main.querySelector('.log-lines');if(newLog)newLog.scrollTop=following?newLog.scrollHeight:logScroll;
 main.querySelectorAll<HTMLButtonElement>('[data-action="start"],[data-action="stop"],[data-action="restart"],[data-quick],[data-action="start-all"],[data-action="stop-all"]').forEach(b=>b.disabled=state.busy);
 if(focusedPackage)main.querySelector<HTMLButtonElement>(`[data-install="${CSS.escape(focusedPackage)}"]:not(:disabled),[data-configure="${CSS.escape(focusedPackage)}"]`)?.focus({preventScroll:true});
 if(focusedVersion)main.querySelector<HTMLSelectElement>(`[data-package-version="${CSS.escape(focusedVersion)}"]`)?.focus({preventScroll:true});
 // Rebuilding the page must not drop the keyboard user back to the top of the document.
 const target=focus?main.querySelector<HTMLElement>(focus):null;
 if(target&&!focusedPackage&&!focusedVersion) {
  target.focus({preventScroll:true});
  if(target instanceof HTMLInputElement&&document.activeElement===target&&start!==null&&start!==undefined&&/^(search|text|url|tel|password)$/.test(target.type))target.setSelectionRange(start,end??start);
 }
 // A control disabled while a command runs gets its focus back on the next render.
 lostFocus=focus&&state.busy&&document.activeElement!==target?focus:'';
 const running=state.snapshot.services.filter(s=>s.state==='running').length;const exited=state.snapshot.services.filter(s=>s.state==='exited').length;
 const count=document.querySelector<HTMLElement>('#service-count')!;count.textContent=`${running} services running`;count.classList.toggle('active',running>0);
 const failed=document.querySelector<HTMLElement>('#exited-count')!;failed.hidden=!exited;failed.textContent=`${exited} exited`;
 document.querySelector('#engine-status')!.textContent='Engine connected';
 hydrate();
}
