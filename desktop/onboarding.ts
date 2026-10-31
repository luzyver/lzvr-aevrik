import '../styles/tokens.css';
import './onboarding.css';
import markUrl from '../assets/mark.svg';
import {invoke,isTauri} from '@tauri-apps/api/core';

interface SetupInfo {preferred:string;application:string}
const form=document.querySelector<HTMLFormElement>('#setup-form')!;
const directory=document.querySelector<HTMLInputElement>('#setup-directory')!;
const browse=document.querySelector<HTMLButtonElement>('#setup-browse')!;
const useDefault=document.querySelector<HTMLButtonElement>('#setup-default')!;
const open=document.querySelector<HTMLButtonElement>('#setup-open')!;
const exit=document.querySelector<HTMLButtonElement>('#setup-exit')!;
const retry=document.querySelector<HTMLButtonElement>('#setup-retry')!;
const error=document.querySelector<HTMLParagraphElement>('#setup-error')!;
const status=document.querySelector<HTMLParagraphElement>('#setup-status')!;
let info:SetupInfo|null=null;
let busy=false;

document.documentElement.dataset.theme=matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light';
document.querySelector<HTMLImageElement>('.setup-heading img')!.src=markUrl;
function showError(value:unknown) {
 error.textContent=value instanceof Error?value.message:String(value);
 error.hidden=false;
}
function clearError() {error.textContent='';error.hidden=true;}
function controls() {
 for(const control of [directory,browse,useDefault,open])control.disabled=busy||!info;
 exit.disabled=busy;retry.disabled=busy;
 form.setAttribute('aria-busy',String(busy));
}
async function load() {
 if(busy)return;
 busy=true;clearError();retry.hidden=true;status.textContent='Loading storage settings…';controls();
 try {
  if(!isTauri())throw new Error('Open Aevrik on Windows to choose your data folder.');
  info=await invoke<SetupInfo>('setup_info');
  directory.value=info.preferred;
  document.querySelector('#setup-application')!.textContent=info.application;
 } catch(value){showError(value);retry.hidden=false;}
 finally {busy=false;status.textContent='';controls();}
}
browse.addEventListener('click',async()=>{
 if(busy||!info)return;
 busy=true;clearError();controls();
 try {
  const path=await invoke<string|null>('pick_path',{kind:'directory'});
  if(path!==null)directory.value=path;
 } catch(value){showError(value);}
 finally {busy=false;controls();directory.focus();}
});
useDefault.addEventListener('click',()=>{
 if(busy||!info)return;
 directory.value=info.preferred;clearError();directory.focus();
});
directory.addEventListener('input',clearError);
retry.addEventListener('click',()=>void load());
exit.addEventListener('click',async()=>{
 if(busy)return;
 try {await invoke('exit_setup');}catch(value){showError(value);}
});
form.addEventListener('submit',async event=>{
 event.preventDefault();
 if(busy||!info)return;
 const selected=directory.value.trim();
 if(!selected){showError('Choose a data folder to continue.');directory.focus();return;}
 busy=true;clearError();open.textContent='Opening…';status.textContent='Opening your workspace…';controls();
 try {await invoke('complete_setup',{directory:selected});}
 catch(value){
  showError(value);busy=false;open.textContent='Open Aevrik';status.textContent='';controls();directory.focus();
 }
});
void load();
