import {state,pages} from './state';
import {esc,icon,button,badge,empty,header,hydrate} from './ui';
import {servicePanel,projectPanel,output} from './panels';
import {packagesView} from './packages';
let renderedView='';
const matches=(...values:string[])=>values.join(' ').toLowerCase().includes(state.query.toLowerCase());
function toolbar() {return `<div class="toolbar"><span class="muted">Registered on this computer</span><label class="search-field">${icon('search')}<input id="search" type="search" placeholder="Filter by name…" value="${esc(state.query)}" aria-label="Filter resources"></label></div>`;}
function services() {
 const all=state.snapshot!.services;const filtered=all.filter(s=>matches(s.name,s.executable));
 return header(state.page==='overview'?'Workspace':'Services','Manage the processes you register with Aevrik.',button('add-service','Register service','plus',true))+toolbar()+
 `<div class="split-view"><section class="list-pane"><div class="pane-heading"><h2>Services <span class="count">${all.length}</span></h2><span class="muted">Process status</span></div>
 ${filtered.length?`<div class="table-wrap"><table class="service-table"><thead><tr><th>Name</th><th>Status</th><th>Port</th><th>PID</th></tr></thead><tbody>${filtered.map(s=>`<tr class="${s.id===state.selectedService?'selected':''}"><td><button class="service-select" data-select-service="${s.id}"><span class="resource-icon">${icon('server')}</span><strong>${esc(s.name)}</strong></button></td><td>${badge(s.state)}</td><td><code>${s.port??'—'}</code></td><td><code>${s.pid??'—'}</code></td></tr>`).join('')}</tbody></table></div>`:
 empty(all.length?'No matching services':'Your local stack starts here',all.length?'Try another search.':'Install a runtime package or register an executable already on your computer.',all.length?'':button('browse-packages','Browse packages','package',true))}
 <div class="list-footer"><span>${all.filter(s=>s.state==='running').length} running</span><span>Owned processes only</span></div></section>${servicePanel(all.find(s=>s.id===state.selectedService))}</div>${output(true)}`;
}
function projects() {
 const all=state.snapshot!.projects;const filtered=all.filter(p=>matches(p.name,p.path,p.domain));
 return header('Projects','Register existing folders and link them to managed services.',button('add-project','Add project','plus',true))+toolbar()+
 `<div class="split-view"><section class="list-pane"><div class="pane-heading"><h2>Projects <span class="count">${all.length}</span></h2></div>${filtered.length?filtered.map(p=>`<button class="project-row ${p.id===state.selectedProject?'selected':''}" data-select-project="${p.id}"><span class="project-logo">${esc(p.name[0])}</span><span class="project-name"><strong>${esc(p.name)}</strong><small>${esc(p.domain)}</small></span><span class="project-runtime">${esc(p.runtime||'Unspecified')}</span>${badge(p.state)}</button>`).join(''):empty(all.length?'No matching projects':'Make room for your next project',all.length?'Try another search.':'Add an existing project folder. Its files stay where they are.',all.length?'':button('add-project','Add project','plus',true))}</section>${projectPanel(all.find(p=>p.id===state.selectedProject))}</div>`;
}
function domains() {
 return header('Domains','Domain labels stored with your projects.')+`<div class="panel">${state.snapshot!.projects.length?`<table><thead><tr><th>Project</th><th>Domain label</th><th>Routing / HTTPS</th></tr></thead><tbody>${state.snapshot!.projects.map(p=>`<tr><td>${esc(p.name)}</td><td><code>${esc(p.domain)}</code></td><td>Not configured</td></tr>`).join('')}</tbody></table>`:empty('No domain labels','Add a project to register its .test domain label.')}</div><p class="page-footnote">DNS routing and local certificates are not configured by this build.</p>`;
}
function settings() {
 return header('Settings','Preferences for this computer.')+`<div class="settings-grid"><div class="panel"><div class="settings-row"><div><strong>Appearance</strong><p>Choose graphite or pearl. Your preference is saved locally.</p></div>${button('theme',state.snapshot!.theme==='dark'?'Use light':'Use dark','sun')}</div>
 <div class="settings-row"><div><strong>When Aevrik closes</strong><p>All managed processes and their children are terminated. Close database services using their own shutdown tools first.</p></div><span class="badge">Stop processes</span></div>
 <div class="settings-row"><div><strong>Start with Windows / system tray</strong><p>Not available in this build.</p></div></div>
 <div class="settings-row"><div><strong>Local storage</strong><p>Projects, service registrations and appearance are stored in your Windows application data folder. Output is limited to 800 lines per session.</p></div></div></div>
 <p class="page-footnote">Aevrik 0.1.0-beta-1 · Rust engine · Tauri desktop shell</p></div>`;
}
export function render() {
 if(!state.snapshot)return;
 const main=document.querySelector<HTMLElement>('#main')!;
 const scroll=main.scrollTop;const log=main.querySelector('.log-lines');const logScroll=log?.scrollTop??0;
 const viewKey=[state.page,state.query,state.selectedService,state.selectedProject].join(':');
 const paneScroll=viewKey===renderedView?Array.from(main.querySelectorAll<HTMLElement>('.catalog-scroll,.list-pane,.detail-pane')).map(p=>({className:p.className,top:p.scrollTop})):[];
 renderedView=viewKey;
 const active=document.activeElement as HTMLInputElement|null;
 const focusId=active?.id;const start=active?.selectionStart;const end=active?.selectionEnd;
 const focusedPackage=active?.dataset.install??active?.dataset.configure;
 const focusedVersion=active?.dataset.packageVersion;
 document.querySelector('#navigation')!.innerHTML=pages.map(p=>`<button class="nav-item ${p.id===state.page?'active':''}" data-nav="${p.id}" aria-label="${p.name}" title="${p.name}" ${p.id===state.page?'aria-current="page"':''}>${icon(p.icon)}<span>${p.name}</span></button>`).join('');
 document.querySelector('[data-nav="settings"]')!.classList.toggle('active',state.page==='settings');
 document.querySelector('#breadcrumb')!.textContent=state.page==='settings'?'Settings':pages.find(p=>p.id===state.page)!.name;
 main.innerHTML=state.page==='projects'?projects():state.page==='packages'?packagesView():state.page==='domains'?domains():state.page==='settings'?settings():state.page==='logs'?header('Output','Live stdout and stderr from managed processes.')+output():services();
 main.scrollTop=scroll;
 for(const pane of main.querySelectorAll<HTMLElement>('.catalog-scroll,.list-pane,.detail-pane'))pane.scrollTop=paneScroll.find(p=>p.className===pane.className)?.top??0;
 const newLog=main.querySelector('.log-lines');if(newLog)newLog.scrollTop=logScroll;
 if(focusId==='search') {const input=main.querySelector<HTMLInputElement>('#search');input?.focus();if(input&&start!==null&&start!==undefined)input.setSelectionRange(start,end??start);}
 if(focusedPackage)main.querySelector<HTMLButtonElement>(`[data-install="${CSS.escape(focusedPackage)}"]:not(:disabled),[data-configure="${CSS.escape(focusedPackage)}"]`)?.focus({preventScroll:true});
 if(focusedVersion)main.querySelector<HTMLSelectElement>(`[data-package-version="${CSS.escape(focusedVersion)}"]`)?.focus({preventScroll:true});
 document.querySelector('#service-count')!.textContent=`${state.snapshot.services.filter(s=>s.state==='running').length} services running`;
 document.querySelector('#engine-status')!.textContent='Engine connected';
 main.querySelectorAll<HTMLButtonElement>('[data-action="start"],[data-action="stop"],[data-action="restart"]').forEach(b=>b.disabled=state.busy);
 hydrate();
}
