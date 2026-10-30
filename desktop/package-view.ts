import {state} from './state';
import {esc,icon,button,empty,header} from './ui';
import {catalog,groups,type PackageGroup} from './package-model';

const mib=(value:number)=>(value/1048576).toFixed(1)+' MiB';
function row(group:PackageGroup) {
 const {selected:p,all,versions,family}=group;
 const busy=catalog.pending.has(p.id)||p.status==='downloading'||p.status==='verifying';
 const downloading=p.status==='downloading';
 const label=p.status==='verifying'?'Verifying & extracting…':downloading?'Downloading…':busy?'Starting download…':p.status==='installed'?'Installed':'Available';
 const progress=downloading&&p.total?`${Math.min(100,Math.floor(p.bytes/p.total*100))}% · ${mib(p.bytes)} / ${mib(p.total)}`:downloading?mib(p.bytes):label;
 const installed=all.filter(v=>v.status==='installed').length;
 const otherActive=all.filter(v=>v.id!==p.id&&(catalog.pending.has(v.id)||['downloading','verifying'].includes(v.status)));
 const control=p.status==='installed'?`<button class="button" data-configure="${esc(p.id)}" ${busy?'disabled':''}>Configure service</button><button class="button danger" data-uninstall="${esc(p.id)}" ${busy?'disabled':''}>Uninstall</button>`:
  `<button class="button ${busy?'':'primary'}" data-install="${esc(p.id)}" ${busy?'disabled':''}>${busy?'Installing…':p.status==='failed'?'Retry':'Install'}</button>`;
 return `<article class="package-row" data-family="${esc(family)}" data-package="${esc(p.id)}"><span class="resource-icon">${icon(p.category==='Database'?'database':p.category==='Web server'?'server':'package')}</span>
 <div class="package-info"><div class="package-name"><h2>${esc(p.name)}</h2><span class="package-kind">${esc(p.category)}</span><span class="version-count">${all.length} versions${installed?` · ${installed} installed`:''}</span></div>
 <p>${esc(p.description)}</p>
 <div class="package-version-line"><label class="package-version">Version <select data-package-version="${esc(family)}" aria-label="${esc(p.name)} version">${versions.map(v=>`<option value="${esc(v.id)}" ${v.id===p.id?'selected':''}>${esc(v.version)}${v.channel?' · '+esc(v.channel):''}${v.status==='installed'?' · Installed':''}</option>`).join('')}</select></label><span class="package-meta">Windows ${esc(p.architecture??'x64')} · ${esc(p.source)}</span></div>
 ${p.path?`<code class="runtime-path">${esc(p.path)}</code>`:''}
 ${busy?`<div class="package-progress"><progress aria-label="${esc(p.name)} ${esc(p.version)} installation" ${downloading&&p.total?`value="${p.bytes}" max="${p.total}"`:''}></progress><span>${esc(downloading?'Downloading · '+progress:progress)}</span></div>`:''}
 ${otherActive.length?`<p class="other-install">Also installing: ${otherActive.map(v=>esc(v.version)).join(', ')}</p>`:''}
 ${p.error?`<p class="package-error" role="alert">${esc(p.error)}</p>`:''}</div>
 <div class="package-actions"><span class="package-status">${p.status==='installed'?'Installed':p.status==='failed'?'Installation failed':busy?'In progress':'Ready to install'}</span>${control}</div></article>`;
}
export function packagesView() {
 const filtered=groups(state.query);
 const families=new Set(catalog.items.map(p=>p.family||p.name)).size;
 const categories=['All',...new Set(catalog.items.map(p=>p.category))];
 return header('Runtime packages','Choose a product and version. Multiple versions can stay installed together.',button('add-service','Register local executable','plus'))+
 `<div class="toolbar catalog-toolbar"><label class="package-filter">Show <select id="package-category" aria-label="Package category">${categories.map(c=>`<option ${c===catalog.category?'selected':''}>${esc(c)}</option>`).join('')}</select></label>
 <span class="catalog-count">${families} products · ${catalog.items.length} versions</span><label class="search-field">${icon('search')}<input id="search" type="search" value="${esc(state.query)}" placeholder="Find a product or version…" aria-label="Filter packages"></label></div>
 <div class="panel catalog-scroll">${catalog.error?empty('Could not load the catalog',catalog.error,button('reload-packages','Retry','refresh')):!catalog.loaded?empty('Loading catalog','Reading the bundled package list…'):filtered.length?filtered.map(row).join(''):empty('No matching packages','Try another name, version or category.')}</div>
 <p class="page-footnote">Verified downloads · Versions install separately. Existing services keep their executable. Installation does not start services or initialize databases.</p>`;
}
