import type { Snapshot } from './bridge';
export type Page = 'projects'|'services'|'packages'|'logs'|'settings';
export const pages: {id:Page;name:string;icon:string}[] = [
 {id:'projects',name:'Projects',icon:'folder'},{id:'services',name:'Services',icon:'server'},
 {id:'packages',name:'Runtimes',icon:'package'},{id:'logs',name:'Output',icon:'terminal'}
];
export type Filter = 'all'|'running'|'exited'|'stopped';
export const state:{page:Page;query:string;filter:Filter;selectedService:string;selectedProject:string;logService:string;startup:boolean|null;snapshot:Snapshot|null;busy:boolean} =
 {page:'projects',query:'',filter:'all',selectedService:'',selectedProject:'',logService:'',startup:false,snapshot:null,busy:false};
const systemDark=window.matchMedia('(prefers-color-scheme: dark)');
/** "system" follows Windows; the page only ever paints "dark" or "light". */
export const resolvedTheme=(theme:string)=>theme==='system'?(systemDark.matches?'dark':'light'):theme;
export function applyTheme() {if(state.snapshot)document.documentElement.dataset.theme=resolvedTheme(state.snapshot.theme);}
systemDark.addEventListener('change',applyTheme);
export function accept(snapshot:Snapshot) {
 state.snapshot=snapshot;
 if(!snapshot.services.some(s=>s.id===state.selectedService)) state.selectedService=snapshot.services[0]?.id??'';
 if(!snapshot.projects.some(p=>p.id===state.selectedProject)) state.selectedProject=snapshot.projects[0]?.id??'';
 if(state.logService&&!snapshot.services.some(s=>s.id===state.logService)) state.logService='';
 applyTheme();
}
