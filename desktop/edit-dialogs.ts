import {dialog} from './forms';
import {esc} from './ui';
import {open} from './package-dialogs';
import type {Project,Service} from './bridge';

const picker=(label:string,name:string,kind:'directory'|'executable',value:string)=>`<label class="field">${label}<span class="path-picker"><input name="${name}" required value="${esc(value)}"><button class="button" type="button" data-pick="${kind}" data-field="${name}">Browse…</button></span></label>`;
export function editProjectDialog(project:Project) {
 open('project-edit',project.id,`Edit ${project.name}`,
 `<label class="field">Name<input name="name" required maxlength="120" autocomplete="off" value="${esc(project.name)}"></label>
 ${picker('Project folder','path','directory',project.path)}
 <label class="field">Domain label<input name="domain" required autocomplete="off" value="${esc(project.domain)}"><small>Saved as metadata. Files are never moved or renamed.</small></label>`,'Save project');
}
export function removeProjectDialog(project:Project) {
 open('project-remove',project.id,`Remove ${project.name}?`,
 `<p class="form-description">Remove this project from Aevrik. Its folder and files stay on disk, and its linked service registration is kept.</p><code class="detail-path">${esc(project.path)}</code>`,'Remove project',true);
}
export function editServiceDialog(service:Service) {
 // An npm service is launched from its project script; only its name and port are editable.
 const launch=service.npmScript?`<p class="form-description">This service runs the npm script <code>${esc(service.npmScript)}</code>. To change the script or Node.js version, remove it and set up the npm script again from its project.</p>`:
 `${picker('Executable (.exe)','executable','executable',service.executable)}${picker('Working directory','cwd','directory',service.cwd)}
 <label class="field">Arguments<textarea name="args">${esc(service.args.join('\n'))}</textarea><small>One argument per line. Do not add shell quotes.</small></label>`;
 open('service-edit',service.id,`Edit ${service.name}`,
 `<label class="field">Name<input name="name" required maxlength="120" autocomplete="off" value="${esc(service.name)}"></label>${launch}
 <label class="field">Expected TCP port (optional)<input name="port" type="number" min="1" max="65535" value="${service.port??''}"></label>`,'Save service');
 dialog.querySelector<HTMLFormElement>('#service-edit')!.dataset.kind='service';
}
export function exitDialog(running:Service[]) {
 open('exit-confirm','',`Stop ${running.length===1?'1 service':`${running.length} services`} and exit?`,
 `<p class="form-description">Exiting ends these processes and their children. Unsaved work inside them is lost; shut down database services using their own tools first.</p><ul class="exit-list">${running.map(s=>`<li>${esc(s.name)}${s.port?` <code>:${s.port}</code>`:''}</li>`).join('')}</ul><p class="form-description">To keep them running, close the window instead: Aevrik stays in the system tray.</p>`,'Stop services and exit',true);
}
