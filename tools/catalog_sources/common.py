"""Official-source fetch/cache and manifest construction helpers (standard library only)."""
import hashlib, json, re, urllib.request
from urllib.parse import urlparse
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
CACHE=ROOT/'.tooling/catalog-source-cache'
CACHE.mkdir(parents=True,exist_ok=True)
PROVENANCE={}

def fetch(url, *, fresh=False):
    if not url.startswith('https://'):raise ValueError('HTTPS required')
    path=CACHE/hashlib.sha256(url.encode()).hexdigest()
    if not fresh and path.exists():return path.read_bytes()
    request=urllib.request.Request(url,headers={'User-Agent':'Aevrik-Catalog/0.3'})
    with urllib.request.urlopen(request,timeout=90) as response:data=response.read()
    path.write_bytes(data)
    return data

def metadata(url):return json.loads(fetch(url))
def version_key(value):return tuple(int(n) for n in re.findall(r'\d+',value))
def selected_versions(values, per_line=2, lines=None):
    counts={};result=[]
    for value in sorted(set(values),key=version_key,reverse=True):
        line='.'.join(value.split('.')[:2])
        if lines and line not in lines:continue
        if counts.get(line,0)>=per_line:continue
        counts[line]=counts.get(line,0)+1;result.append(value)
    return result

def entry(family,name,version,category,description,url,exe,sha=None,source_url=None,architecture='x64',channel='Release'):
    method='publisher-sha256'
    if not sha:
        sha=hashlib.sha256(fetch(url)).hexdigest();method='official-https-archive-sha256'
    if not re.fullmatch(r'[a-fA-F0-9]{64}',sha):raise ValueError(f'Invalid SHA-256: {name} {version}')
    release_id=family+'-'+re.sub(r'[^a-z0-9.-]','-',version.lower())
    p=dict(id=release_id,family=family,name=name,version=version,category=category,description=description,
           url=url,source=urlparse(url).netloc,
           sha256=sha.lower(),executable=exe,architecture=architecture,channel=channel)
    PROVENANCE[(family,version)]={'url':source_url or url,'digestMethod':method}
    return p
