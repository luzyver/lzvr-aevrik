import {dialog} from './forms';
import {esc,icon} from './ui';
import type {RuntimePackage} from './package-model';
import type {Service} from './bridge';

function open(id:string,resourceId:string,title:string,body:string,action:string,danger=false) {
 dialog.innerHTML=`<form id="${id}" data-id="${esc(resourceId)}"><header class="dialog-header"><h2 id="dialog-title">${esc(title)}</h2><button class="icon-button" type="button" data-action="close-dialog" aria-label="Close">${icon('close')}</button></header><div class="dialog-body">${body}<p id="form-error" class="form-error" role="alert"></p></div><footer class="dialog-footer"><button class="button" type="button" data-action="close-dialog">Cancel</button><button class="button ${danger?'danger':'primary'}" type="submit">${action}</button></footer></form>`;
 dialog.showModal();
}
export function installDialog(package_:RuntimePackage,root:string) {
 open('package-install',package_.id,`Install ${package_.name} ${package_.version}`,
 `<p class="form-description">Choose where this version will be downloaded and installed. The folder is remembered for your next installation.</p><label class="field">Installation folder<span class="path-picker"><input name="destination" required value="${esc(root)}" placeholder="D:\\Aevrik\\runtimes"><button class="button" type="button" data-pick="directory" data-field="destination">Browse…</button></span></label><p class="form-description">A separate <code>${esc(package_.id)}\\${esc(package_.version)}</code> folder will be created inside it. Existing versions stay in their current locations. Database data should be stored outside runtime folders.</p>`,'Install');
}
export function uninstallDialog(package_:RuntimePackage) {
 open('package-uninstall',package_.id,`Uninstall ${package_.name} ${package_.version}?`,
 `<p class="form-description">This permanently removes the version folder below and everything inside it, including any database or configuration files you placed there. Other versions and files outside this folder remain.</p><code class="detail-path">${esc(package_.installDirectory??package_.path??'Location unavailable')}</code><p class="form-description">Removal is blocked while a service registration or project references this folder. Stop its service, then remove the registration first.</p>`,'Uninstall',true);
}
export function removeServiceDialog(service:Service) {
 open('service-remove',service.id,`Remove ${service.name}?`,
 '<p class="form-description">Remove this stopped service registration and unlink it from projects. Executables, runtime installations and project files will stay on disk.</p>','Remove registration',true);
}
