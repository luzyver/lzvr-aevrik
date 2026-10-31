export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
export interface Project {id:string;name:string;path:string;domain:string;runtime:string;serviceId:string|null;state:string}
export interface Service {id:string;name:string;executable:string;args:string[];cwd:string;port:number|null;state:string;pid:number|null;exitCode:number|null}
export interface LogEntry {timestamp:number;service:string;level:string;message:string}
export interface Snapshot {projects:Project[];services:Service[];logs:LogEntry[];theme:string}
export interface StorageInfo {current:string;next:string|null;application:string;overridden:boolean}
export class Backend {
  private invoke: Invoke;
  constructor(invoke: Invoke) {this.invoke=invoke;}
  snapshot(): Promise<Snapshot> {return this.invoke('snapshot');}
  storage():Promise<StorageInfo> {return this.invoke('storage_info');}
  setDataDirectory(directory:string):Promise<StorageInfo> {return this.invoke('set_data_directory',{directory});}
  packages(): Promise<RuntimePackage[]> {return this.invoke('list_packages');}
  installRoot():Promise<string> {return this.invoke('package_install_root');}
  installPackage(id:string,destination?:string): Promise<void> {return this.invoke('install_package',destination===undefined?{id}:{id,destination});}
  uninstallPackage(id:string):Promise<void> {return this.invoke('uninstall_package',{id});}
  async service(id:string,action:'start'|'stop'|'restart'):Promise<Snapshot> {
    await this.invoke('control_service',{id,action});
    return this.snapshot();
  }
  async mutate(command:string,args?:Record<string,unknown>):Promise<Snapshot> {
    await this.invoke(command,args);
    return this.snapshot();
  }
  pick(kind:'directory'|'executable'):Promise<string|null> {return this.invoke('pick_path',{kind});}
}
import type {RuntimePackage} from './packages';
