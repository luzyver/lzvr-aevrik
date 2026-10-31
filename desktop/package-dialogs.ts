import {dialog} from './forms';
import {esc,icon} from './ui';
import type {RuntimePackage} from './package-model';
import type {DefaultPlan,Service} from './bridge';

export const defaultPlans=new WeakMap<HTMLFormElement,DefaultPlan>();
export function defaultDialog(item:RuntimePackage,plan:DefaultPlan) {
 const action=plan.removing?'Remove from PATH':'Set as default';
 open('package-default',item.id,`${action}: ${item.name} ${item.version}`,
 `<p class="form-description">${plan.removing?'Remove this version’s folders from the selected Windows PATH. Another version already on PATH may take over. If it is listed in both scopes, repeat to remove the remaining entry.':'Choose this installed version for terminal commands. Other runtime folders stay on PATH as fallbacks.'}</p>
 <dl class="default-summary"><dt>Applies to</dt><dd>${plan.scope==='machine'?'All Windows users · Administrator required':'This Windows account'}</dd><dt>Runtime</dt><dd><code>${esc(plan.executable)}</code></dd><dt>Current PATH command</dt><dd><code>${esc(plan.currentExecutable??'No runtime found on PATH')}</code></dd></dl>
 ${plan.scope==='machine'?'<p class="form-description">The system PATH takes priority. This change updates it for all users; reopen Aevrik with Run as administrator if Windows denies access.</p>':''}
 <p class="notice">Restart your terminal or editor after applying. Shell aliases and explicit service/project paths keep their own version.${item.family==='python'?' Active virtual environments and Python’s <code>py</code> launcher have their own version selection. Virtual environment packages stay there. Use <code>python -m pip</code> for this interpreter.':''}</p>`,action);
 const form=dialog.querySelector<HTMLFormElement>('#package-default')!;
 // Keep permission/stale-plan errors visible even when a compact window scrolls the details.
 form.insertBefore(form.querySelector('#form-error')!,form.querySelector('.dialog-footer'));
 defaultPlans.set(form,plan);
}

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
