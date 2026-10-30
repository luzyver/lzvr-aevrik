import {state} from './state';
import {esc,icon} from './ui';
export const dialog=document.querySelector<HTMLDialogElement>('#dialog')!;
const field=(label:string,name:string,attrs='')=>`<label class="field">${label}<input name="${name}" ${attrs}></label>`;
const pathField=(label:string,name:string,kind:string)=>`<label class="field">${label}<span class="path-picker"><input name="${name}" required placeholder="Select a local ${kind==='directory'?'folder':'executable'}"><button class="button" type="button" data-pick="${kind}" data-field="${name}">Browse…</button></span></label>`;
export function openForm(kind:'project'|'service') {
 const service=kind==='service';
 dialog.innerHTML=`<form id="registration" data-kind="${kind}"><header class="dialog-header"><h2 id="dialog-title">${service?'Register service':'Add project'}</h2><button class="icon-button" type="button" data-action="close-dialog" aria-label="Close">${icon('close')}</button></header><div class="dialog-body">
 <p class="form-description">${service?'Select an existing executable and its launch settings. It will run under your Windows account when you press Start.':'Register an existing folder. Aevrik stores its settings without creating or moving your project files.'}</p>
 ${field('Name','name','required maxlength="120" autocomplete="off"')}
 ${service?pathField('Executable (.exe)','executable','executable')+pathField('Working directory','cwd','directory')+
 `<label class="field">Arguments<textarea name="args" placeholder="run&#10;--config&#10;C:\\Projects\\Caddyfile"></textarea><small>One argument per line. Do not add shell quotes. Commands such as &amp; and | are passed literally.</small></label>`+
 field('Expected TCP port (optional)','port','type="number" min="1" max="65535" placeholder="8080"')+
 '<p class="form-description">Stop terminates the process tree. Use foreground mode; services must not detach or install themselves as Windows services.</p>'
 :pathField('Project folder','path','directory')+field('Domain label','domain','required placeholder="my-project.test" autocomplete="off"')+
 field('Runtime label (optional)','runtime','maxlength="80" placeholder="PHP 8.4"')+
 `<label class="field">Managed service (optional)<select name="serviceId"><option value="">Link later</option>${state.snapshot!.services.map(s=>`<option value="${esc(s.id)}">${esc(s.name)}</option>`).join('')}</select><small>The .test label is saved as metadata. DNS and HTTPS are not configured yet.</small></label>`}
 <p id="form-error" class="form-error" role="alert"></p></div><footer class="dialog-footer"><button type="button" class="button" data-action="close-dialog">Cancel</button><button type="submit" class="button primary">${service?'Register service':'Add project'}</button></footer></form>`;
 dialog.showModal();
}
export function registration(form:HTMLFormElement):{command:string;input:Record<string,unknown>} {
 const data=new FormData(form);const text=(name:string)=>String(data.get(name)??'').trim();
 return form.dataset.kind==='service'?{command:'register_service',input:{
 name:text('name'),executable:text('executable'),cwd:text('cwd'),
 args:String(data.get('args')??'').split(/\r?\n/).filter(line=>line.length>0),port:text('port')?Number(text('port')):null
 }}:{command:'add_project',input:{name:text('name'),path:text('path'),domain:text('domain'),runtime:text('runtime'),serviceId:text('serviceId')||null}};
}
