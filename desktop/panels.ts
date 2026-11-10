import {state} from './state';
import type {Service,Project,LogEntry} from './bridge';
import {esc,icon,button,badge,empty,iconButton} from './ui';
import {needsNodeScript} from './service-launch';
import {catalog} from './package-model';
export function controls(service:Service) {
 const id=`data-id="${esc(service.id)}"`;
 // data-control keeps keyboard focus on the toggle when Start becomes Stop.
 return button(service.state==='running'?'stop':'start',service.state==='running'?'Stop':'Start',service.state==='running'?'stop':'play',true,`${id} data-control="${esc(service.id)}"`)+
 (service.state==='running'?button('restart','Restart','restart',false,id):'');
}
/** One-click Start/Stop for a list row, without opening the properties first. */
export function quickControl(service:Service) {
 const running=service.state==='running';
 return iconButton(`data-quick="${running?'stop':'start'}" data-id="${esc(service.id)}" data-control="row:${esc(service.id)}"`,`${running?'Stop':'Start'} ${service.name}`,running?'stop':'play');
}
export const browserButton=(service:Service)=>service.port&&service.state==='running'?iconButton(`data-open="service-url" data-id="${esc(service.id)}"`,`Open localhost:${service.port} in browser`,'external'):'';
export function servicePanel(service?:Service):string {
 if(!service)return `<aside class="detail-pane">${empty('Service properties','Select or register a service.')}</aside>`;
 const running=service.state==='running';
 return `<aside class="detail-pane"><h3>Service</h3><div class="detail-identity"><h2>${esc(service.name)}</h2>${badge(service.state,service.exitCode)}</div><div class="header-actions">${controls(service)}${service.port&&running?button('open','Open in browser','external',false,`data-open="service-url" data-id="${esc(service.id)}"`):''}</div>
 ${service.state==='exited'?`<p class="notice failure">This process ended on its own${service.exitCode===null?'':` with exit code ${service.exitCode}`}. Check Output for its last lines, then start it again.</p>`:''}
 <div class="detail-row"><span>Process ID</span><code>${service.pid??'—'}</code></div><div class="detail-row"><span>Expected port</span><code>${service.port??'None'}</code></div><div class="detail-row"><span>Last exit code</span><code>${service.exitCode??'—'}</code></div>
 <h3 class="detail-section">Executable</h3><code class="detail-path">${esc(service.executable)}</code>
 <h3 class="detail-section">Working directory</h3><code class="detail-path">${esc(service.cwd)}</code>
 <h3 class="detail-section">Arguments</h3><code class="detail-path">${service.args.map(esc).join('<br>')||'None'}</code>
 ${needsNodeScript(service)?'<p class="notice">This service starts Node.js without a script, so it exits without serving a project. Open Projects, unlink this service and set up an npm script.</p>':''}
 <p class="detail-note">Running means the process is alive; application readiness is not checked. Stop terminates this process and its children.${running?' Stop it to edit or remove its registration.':''}</p>
 <div class="detail-actions"><button class="button" data-action="edit-service" data-id="${esc(service.id)}" ${running?'disabled':''}>Edit registration</button><button class="button danger" data-action="remove-service" data-id="${esc(service.id)}" ${running?'disabled':''}>Remove registration</button></div></aside>`;
}
export function projectPanel(project?:Project):string {
 if(!project)return `<aside class="detail-pane">${empty('Project properties','Register an existing project to begin.')}</aside>`;
 const services=state.snapshot!.services;
 const linked=services.find(s=>s.id===project.serviceId);
 const missingScript=linked&&needsNodeScript(linked);
 const id=`data-id="${esc(project.id)}"`;
 const runtimes=catalog.items.filter(p=>p.status==='installed').map(p=>`${p.name} ${p.version}`);
 return `<aside class="detail-pane"><h3>Project</h3><div class="detail-identity"><h2>${esc(project.name)}</h2>${badge(project.state,linked?.exitCode??null)}</div>
 ${linked&&(!missingScript||linked.state==='running')?`<div class="header-actions">${controls(linked)}</div>`:''}
 ${missingScript?`<p class="notice" data-project-launch-help>This service starts Node.js without a script. Unlink it, then choose Set up npm script to run this project.</p><button class="button" type="button" data-action="unlink-project-service" ${linked.state==='running'?'disabled':''}>Unlink service</button>`:''}
 ${!linked?button('setup-npm','Set up npm script','terminal',false,id):''}
 <div class="header-actions open-actions">${linked?.port&&linked.state==='running'?button('open','Browser','external',false,`data-open="service-url" data-id="${esc(linked.id)}"`):''}${button('open','Folder','folder',false,`data-open="project-folder" ${id}`)}${button('open','Terminal','terminal',false,`data-open="project-terminal" ${id}`)}</div>
 <h3 class="detail-section">Project directory</h3><code class="detail-path">${esc(project.path)}</code>
 <div class="detail-row"><span>Domain label</span><code>${esc(project.domain)}</code></div><div class="detail-row"><span>HTTPS / routing</span><strong>Not configured</strong></div>
 <form id="project-properties" data-id="${esc(project.id)}"><label class="field">Managed service<select name="serviceId"><option value="">No linked service</option>${services.map(s=>`<option value="${esc(s.id)}" ${s.id===project.serviceId?'selected':''}>${esc(s.name)}</option>`).join('')}</select></label>
 <label class="field">Runtime label<input name="runtime" maxlength="80" value="${esc(project.runtime)}" placeholder="PHP 8.4" list="runtime-labels" autocomplete="off"></label>
 <datalist id="runtime-labels">${runtimes.map(r=>`<option value="${esc(r)}"></option>`).join('')}</datalist>
 <button type="submit" class="button">Save properties</button></form>
 <p class="detail-note">The runtime label describes this project${runtimes.length?'; installed runtimes are suggested as you type':''}. The domain label is saved as metadata: DNS and local certificates are not configured by Aevrik.</p>
 <div class="detail-actions"><button class="button" data-action="edit-project" ${id}>Edit project</button><button class="button danger" data-action="remove-project" ${id}>Remove project</button></div></aside>`;
}
const serviceNames=()=>new Map(state.snapshot!.services.map(s=>[s.id,s.name]));
export function visibleLogs():LogEntry[] {
 const logs=state.snapshot!.logs;
 return (state.logService?logs.filter(l=>l.service===state.logService):logs).slice(-800);
}
/** The same lines the panel shows, as plain text for the clipboard. */
export function logText():string {
 const names=serviceNames();
 return visibleLogs().map(l=>`${new Date(l.timestamp).toLocaleTimeString('en-GB')} ${l.level.padEnd(5)} [${names.get(l.service)??l.service}] ${l.message}`).join('\n');
}
/** The Output page, or (compact) the dock under a list, which follows the selected row's service. */
export function output(compact=false,only=''):string {
 const names=serviceNames();
 const logs=compact?state.snapshot!.logs.filter(l=>!only||l.service===only).slice(-200):visibleLogs();
 const quiet=compact?only:state.logService;
 const filter=compact?'':`<label class="log-filter">Show<select id="log-service" aria-label="Show output from"><option value="">All services</option>${state.snapshot!.services.map(s=>`<option value="${esc(s.id)}" ${s.id===state.logService?'selected':''}>${esc(s.name)}</option>`).join('')}</select></label>`;
 return `<section class="log-panel ${compact?'workspace-log':''}"><div class="log-toolbar"><span>${icon('terminal')}Output <span class="muted">· ${compact?esc(only?names.get(only)??only:'All services'):'Current session'}</span></span><span class="log-actions">${filter}${compact?'':button('copy-logs','Copy','copy')}${button('clear-logs','Clear','close')}</span></div><div class="log-lines" tabindex="0" aria-label="Process output">${logs.map(l=>`<span class="log-time">${new Date(l.timestamp).toLocaleTimeString('en-GB')}</span> <span class="${l.level==='ERROR'?'log-error':l.level==='WARN'?'log-warn':'log-info'}">${esc(l.level.padEnd(5))}</span> ${compact&&only?'':`<span class="muted">[${esc(names.get(l.service)??l.service)}]</span> `}${esc(l.message)}`).join('\n')||`<span class="muted">${quiet?'This service has not written any output in this session.':'Process output will appear here after a service starts.'}</span>`}</div></section>`;
}
