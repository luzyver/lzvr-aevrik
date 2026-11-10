import {dialog} from './forms';
import {esc,icon} from './ui';

export interface Command {label:string;hint:string;glyph:string;run:()=>void|Promise<void>}
let commands:Command[]=[];
let shown:Command[]=[];
let active=0;
const isOpen=()=>dialog.open&&dialog.classList.contains('command-dialog');
function list(query:string) {
 const words=query.toLowerCase().split(/\s+/).filter(Boolean);
 shown=commands.filter(c=>words.every(w=>`${c.label} ${c.hint}`.toLowerCase().includes(w)));
 active=0;
 dialog.querySelector('#command-results')!.innerHTML=shown.length?shown.map((c,i)=>`<button class="command-result" type="button" role="option" id="command-${i}" data-command="${i}" tabindex="-1" aria-selected="${i===0}">${icon(c.glyph)}<span>${esc(c.label)}</span><small>${esc(c.hint)}</small></button>`).join(''):'<p class="command-empty">Nothing matches. Try a project, service or page name.</p>';
 mark();
}
function mark() {
 dialog.querySelectorAll<HTMLElement>('.command-result').forEach((node,i)=>{
  node.setAttribute('aria-selected',String(i===active));
  if(i===active)node.scrollIntoView({block:'nearest'});
 });
 const input=dialog.querySelector('#command-query');
 if(shown.length)input?.setAttribute('aria-activedescendant',`command-${active}`);else input?.removeAttribute('aria-activedescendant');
}
/** Closes the palette and drops its marker at once: the command may open another dialog before the close event. */
function dismiss() {dialog.close();dialog.classList.remove('command-dialog');}
async function run(command?:Command) {
 if(!command)return;
 dismiss();
 await command.run();
}
/** Ctrl+K: jump to any page, project or service, or run a command, from the keyboard. */
export function openPalette(items:Command[]) {
 commands=items;
 dialog.classList.add('command-dialog');
 dialog.innerHTML=`<h2 id="dialog-title" class="sr-only">Search and commands</h2><div class="command-input">${icon('search')}<input id="command-query" type="text" role="combobox" aria-expanded="true" aria-controls="command-results" aria-autocomplete="list" aria-label="Search projects, services and commands" placeholder="Search projects, services and commands…" autocomplete="off" spellcheck="false"></div><div id="command-results" class="command-results" role="listbox" aria-label="Results"></div>`;
 list('');
 dialog.showModal();
}
// The close event is queued: by the time it arrives the palette may already be open again.
dialog.addEventListener('close',()=>{if(!dialog.open)dialog.classList.remove('command-dialog');});
dialog.addEventListener('input',event=>{if(isOpen()&&(event.target as Element).id==='command-query')list((event.target as HTMLInputElement).value);});
dialog.addEventListener('click',event=>{
 if(!isOpen())return;
 // A click on the backdrop lands on the dialog element itself.
 if(event.target===dialog){dismiss();return;}
 const index=(event.target as Element).closest<HTMLElement>('[data-command]')?.dataset.command;
 if(index!==undefined)void run(shown[Number(index)]);
});
dialog.addEventListener('keydown',event=>{
 if(!isOpen())return;
 if(event.key==='ArrowDown'||event.key==='ArrowUp') {
  event.preventDefault();
  if(shown.length)active=(active+(event.key==='ArrowDown'?1:shown.length-1))%shown.length;
  mark();
 }
 if(event.key==='Enter'){event.preventDefault();void run(shown[active]);}
});
