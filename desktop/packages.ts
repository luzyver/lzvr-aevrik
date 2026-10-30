import type {Backend} from './bridge';
import {catalog} from './package-model';
export {catalog} from './package-model';
export type {RuntimePackage} from './package-model';
export {packagesView} from './package-view';
export class PackageController {
 private polling=false;
 constructor(private backend:Backend,private changed:()=>void){}
 async refresh() {
  if(this.polling)return;
  this.polling=true;
  const before=JSON.stringify([catalog.items,catalog.error,catalog.loaded]);
  try {catalog.items=await this.backend.packages();catalog.error='';catalog.loaded=true;}
  catch(error){catalog.error=String(error);}
  finally {this.polling=false;if(before!==JSON.stringify([catalog.items,catalog.error,catalog.loaded]))this.changed();}
 }
 async install(id:string,destination?:string) {
  if(catalog.pending.has(id))return;
  catalog.pending.add(id);this.changed();
  try {await this.backend.installPackage(id,destination);}
  finally {catalog.pending.delete(id);await this.refresh();this.changed();}
 }
 async uninstall(id:string) {
  if(catalog.pending.has(id))return;
  catalog.pending.add(id);this.changed();
  try {await this.backend.uninstallPackage(id);}
  finally {catalog.pending.delete(id);await this.refresh();this.changed();}
 }
}
