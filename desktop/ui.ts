import './icons.js';
declare global { interface Window { Aevrik: { icon:(name:string)=>string; escape:(value:unknown)=>string; hydrateIcons:(root?:ParentNode)=>void } } }
export const icon = (name:string) => window.Aevrik.icon(name);
export const esc = (value:unknown) => window.Aevrik.escape(value);
export const hydrate = () => window.Aevrik.hydrateIcons();
export const button = (action:string,label:string,glyph:string,primary=false,extra='') => `<button class="button ${primary?'primary':''}" data-action="${action}" ${extra}>${icon(glyph)}${esc(label)}</button>`;
/** An exited process is a failure the user must notice; stopped is the quiet resting state. */
export const badge = (state:string,exitCode:number|null=null) => {
 const tone=state==='running'?'running':state==='exited'?'failed':'';
 const label=state[0].toUpperCase()+state.slice(1)+(state==='exited'&&exitCode!==null?` · code ${exitCode}`:'');
 return `<span class="badge ${tone}"><i class="dot ${state==='running'?'active-dot':''}"></i>${esc(label)}</span>`;
};
export const iconButton = (attrs:string,label:string,glyph:string) => `<button class="icon-button" type="button" ${attrs} aria-label="${esc(label)}" title="${esc(label)}">${icon(glyph)}</button>`;
export const empty = (title:string,description:string,action='') => `<div class="empty">${icon('folder')}<strong>${esc(title)}</strong><p>${esc(description)}</p>${action}</div>`;
export const header = (title:string,description:string,actions='') => `<header class="page-header"><div><h1>${esc(title)}</h1><p>${esc(description)}</p></div><div class="header-actions">${actions}</div></header>`;
export function toast(message:string) {
 const node=document.querySelector<HTMLDivElement>('#toast')!; node.textContent=message;node.classList.add('show');
 window.clearTimeout(toastTimer);toastTimer=window.setTimeout(()=>node.classList.remove('show'),5000);
}
let toastTimer=0;
