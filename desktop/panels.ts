import {state} from './state';
import type {Service,Project} from './bridge';
import {esc,icon,button,badge,empty} from './ui';
export function controls(service:Service) {
 const id=`data-id="${esc(service.id)}"`;
 return button(service.state==='running'?'stop':'start',service.state==='running'?'Stop':'Start',service.state==='running'?'stop':'play',true,id)+
 (service.state==='running'?button('restart','Restart','restart',false,id):'');
}
export function servicePanel(service?:Service):string {
 if(!service)return `<aside class="detail-pane">${empty('Service properties','Select or register a service.')}</aside>`;
 return `<aside class="detail-pane"><h3>Properties</h3><div class="detail-identity"><span class="resource-icon">${icon('server')}</span><div><h2>${esc(service.name)}</h2>${badge(service.state)}</div></div><div class="header-actions">${controls(service)}</div>
 <div class="detail-row"><span>Process ID</span><code>${service.pid??'—'}</code></div><div class="detail-row"><span>Expected port</span><code>${service.port??'None'}</code></div><div class="detail-row"><span>Last exit code</span><code>${service.exitCode??'—'}</code></div>
 <h3 class="detail-section">Executable</h3><code class="detail-path">${esc(service.executable)}</code>
 <h3 class="detail-section">Working directory</h3><code class="detail-path">${esc(service.cwd)}</code>
 <h3 class="detail-section">Arguments</h3><code class="detail-path">${service.args.map(esc).join('<br>')||'None'}</code>
 <p class="detail-note">Running means the process is alive; application readiness is not checked. Stop terminates this process and its children.</p>
 <button class="button danger" data-action="remove-service" data-id="${esc(service.id)}" ${service.state==='running'?'disabled':''}>Remove registration</button></aside>`;
}
export function projectPanel(project?:Project):string {
 if(!project)return `<aside class="detail-pane">${empty('Project properties','Register an existing project to begin.')}</aside>`;
 const services=state.snapshot!.services;
 const linked=services.find(s=>s.id===project.serviceId);
 return `<aside class="detail-pane"><h3>Properties</h3><div class="detail-identity"><span class="project-logo">${esc(project.name[0])}</span><div><h2>${esc(project.name)}</h2>${badge(project.state)}</div></div>
 ${linked?`<div class="header-actions">${controls(linked)}</div>`:''}
 <h3 class="detail-section">Project directory</h3><code class="detail-path">${esc(project.path)}</code>
 <div class="detail-row"><span>Domain label</span><code>${esc(project.domain)}</code></div><div class="detail-row"><span>HTTPS / routing</span><strong>Not configured</strong></div>
 <form id="project-properties" data-id="${esc(project.id)}"><label class="field">Managed service<select name="serviceId"><option value="">No linked service</option>${services.map(s=>`<option value="${esc(s.id)}" ${s.id===project.serviceId?'selected':''}>${esc(s.name)}</option>`).join('')}</select></label>
 <label class="field">Runtime label<input name="runtime" maxlength="80" value="${esc(project.runtime)}" placeholder="PHP 8.4"></label>
 <button type="submit" class="button">Save properties</button></form>
 <p class="detail-note">The runtime label describes this project. Configure the executable and arguments when registering its service.</p></aside>`;
}
export function output(compact=false):string {
 const logs=state.snapshot!.logs.slice(compact?-8:-800);
 const names=new Map(state.snapshot!.services.map(s=>[s.id,s.name]));
 return `<section class="log-panel ${compact?'workspace-log':''}"><div class="log-toolbar"><span>${icon('terminal')}Output <span class="muted">· Current session</span></span>${button('clear-logs','Clear','close')}</div><div class="log-lines" tabindex="0" aria-label="Process output">${logs.map(l=>`<span class="log-time">${new Date(l.timestamp).toLocaleTimeString('en-GB')}</span> <span class="${l.level==='ERROR'?'log-warn':'log-info'}">${esc(l.level.padEnd(5))}</span> <span class="muted">[${esc(names.get(l.service)??l.service)}]</span> ${esc(l.message)}`).join('\n')||'<span class="muted">Process output will appear here after a service starts.</span>'}</div></section>`;
}
