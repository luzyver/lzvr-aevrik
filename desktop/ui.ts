import './icons.js';
declare global { interface Window { Aevrik: { icon:(name:string)=>string; escape:(value:unknown)=>string; hydrateIcons:(root?:ParentNode)=>void } } }
export const icon = (name:string) => window.Aevrik.icon(name);
export const esc = (value:unknown) => window.Aevrik.escape(value);
export const hydrate = () => window.Aevrik.hydrateIcons();
export const button = (action:string,label:string,glyph:string,primary=false,extra='') => `<button class="button ${primary?'primary':''}" data-action="${action}" ${extra}>${icon(glyph)}${esc(label)}</button>`;
export const badge = (state:string) => `<span class="badge ${state==='running'?'running':''}"><i class="dot ${state==='running'?'active-dot':''}"></i>${esc(state[0].toUpperCase()+state.slice(1))}</span>`;
export const empty = (title:string,description:string,action='') => `<div class="empty">${icon('folder')}<strong>${esc(title)}</strong><p>${esc(description)}</p>${action}</div>`;
export const header = (title:string,description:string,actions='') => `<header class="page-header"><div><h1>${esc(title)}</h1><p>${esc(description)}</p></div><div class="header-actions">${actions}</div></header>`;
export function toast(message:string) {
 const node=document.querySelector<HTMLDivElement>('#toast')!; node.textContent=message;node.classList.add('show');
 window.clearTimeout(toastTimer);toastTimer=window.setTimeout(()=>node.classList.remove('show'),5000);
}
let toastTimer=0;
