export interface RuntimePackage {
 id:string;family?:string;name:string;version:string;category:string;description:string;source:string;
 architecture?:string;channel?:string;
 status:'available'|'downloading'|'verifying'|'installed'|'failed';
 bytes:number;total:number|null;error:string|null;path:string|null;
 installDirectory?:string|null;
}
export const catalog:{items:RuntimePackage[];loaded:boolean;error:string;category:string;pending:Set<string>;selected:Record<string,string>}={
 items:[],loaded:false,error:'',category:'All',pending:new Set(),selected:{}
};
export interface PackageGroup {family:string;all:RuntimePackage[];versions:RuntimePackage[];selected:RuntimePackage}
const order=new Intl.Collator('en',{numeric:true});
export function availableUpdate(all:RuntimePackage[]):RuntimePackage|undefined {
 const installed=all.filter(p=>p.status==='installed').sort((a,b)=>order.compare(b.version,a.version))[0];
 if(!installed)return;
 return all.filter(p=>p.status!=='installed'&&!/preview|beta|alpha|rc|nightly|dev/i.test(p.channel+' '+p.version)&&order.compare(p.version,installed.version)>0)
  .sort((a,b)=>order.compare(b.version,a.version))[0];
}
export function groups(query=''):PackageGroup[] {
 const families=new Map<string,RuntimePackage[]>();
 for(const p of catalog.items){const key=p.family||p.name;const versions=families.get(key)||[];versions.push(p);families.set(key,versions);}
 const result:PackageGroup[]=[];const search=query.trim().toLowerCase();
 for(const [family,all] of families){
  all.sort((a,b)=>order.compare(b.version,a.version));
  if(catalog.category!=='All'&&all[0]!.category!==catalog.category)continue;
  const versions=all.filter(p=>`${p.name} ${p.version} ${p.category} ${p.channel??''}`.toLowerCase().includes(search));
  if(!versions.length)continue;
  const selected=versions.find(p=>p.id===catalog.selected[family])??versions[0]!;
  result.push({family,all,versions,selected});
 }
 return result;
}
