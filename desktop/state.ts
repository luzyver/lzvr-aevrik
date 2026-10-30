import type { Snapshot } from './bridge';
export type Page = 'overview'|'projects'|'services'|'packages'|'domains'|'logs'|'settings';
export const pages: {id:Page;name:string;icon:string}[] = [
 {id:'overview',name:'Workspace',icon:'overview'},{id:'projects',name:'Projects',icon:'folder'},
 {id:'services',name:'Services',icon:'server'},{id:'packages',name:'Runtimes',icon:'package'},
 {id:'domains',name:'Domains',icon:'globe'},{id:'logs',name:'Output',icon:'terminal'}
];
export const state:{page:Page;query:string;selectedService:string;selectedProject:string;snapshot:Snapshot|null;busy:boolean} =
 {page:'overview',query:'',selectedService:'',selectedProject:'',snapshot:null,busy:false};
export function accept(snapshot:Snapshot) {
 state.snapshot=snapshot;
 if(!snapshot.services.some(s=>s.id===state.selectedService)) state.selectedService=snapshot.services[0]?.id??'';
 if(!snapshot.projects.some(p=>p.id===state.selectedProject)) state.selectedProject=snapshot.projects[0]?.id??'';
 document.documentElement.dataset.theme=snapshot.theme;
}
