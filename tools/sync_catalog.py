"""Build a reviewable bundled catalog from official metadata. No app-data writes.

python tools/sync_catalog.py [--output <candidate.json>]
Metadata/archive cache: .tooling/catalog-source-cache. Remove selected cache files to refresh upstream indexes.
"""
import argparse, concurrent.futures, datetime, json
from collections import Counter
from pathlib import Path
from catalog_sources import languages, servers, extras
from catalog_sources.common import ROOT, PROVENANCE, version_key

TARGET=ROOT/'crates/aevrik-engine/src/packages/catalog.json'

def run(output):
    legacy=json.loads(TARGET.read_text())
    tasks=[languages.php,languages.node,languages.go,languages.python,languages.java,servers.github,servers.maria,servers.nginx,extras.mongo,extras.duckdb,extras.sqlite]
    results=[]
    with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
        futures={pool.submit(task):task.__name__ for task in tasks}
        for future in concurrent.futures.as_completed(futures):
            rows=future.result();results.extend(rows);print(f'{futures[future]}: {len(rows)} releases',flush=True)
    old={(p['name'],p['version']):p for p in legacy}
    for p in results:
        previous=old.get((p['name'],p['version']))
        if previous:
            # Preserve public release identity, hash, URL and install path of every shipped package.
            for key in ['id','url','sha256','executable']:p[key]=previous[key]
    keys={(p['name'],p['version']) for p in results}
    for p in legacy:
        if (p['name'],p['version']) not in keys:
            p.setdefault('family',{'Node.js':'node','Java (Temurin)':'java'}.get(p['name'],p['name'].lower()))
            p.setdefault('architecture','x64');p.setdefault('channel','Release');results.append(p)
    order=['caddy','nginx','php','node','python','go','java','bun','deno','mariadb','mongodb','duckdb','sqlite','mailpit']
    results.sort(key=lambda p:version_key(p['version']),reverse=True)
    results.sort(key=lambda p:order.index(p['family']))
    assert len({p['id'] for p in results})==len(results),'Duplicate release id'
    output.parent.mkdir(parents=True,exist_ok=True)
    output.write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
    provenance={'generatedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'entries':[{ 'id':p['id'],'version':p['version'],**PROVENANCE.get((p['family'],p['version']),{'url':p['url'],'digestMethod':'preserved-shipped-pin'})} for p in results]}
    output.with_suffix('.sources.json').write_text(json.dumps(provenance,indent=2)+'\n')
    print(json.dumps({'products':dict(Counter(p['name'] for p in results)),'releases':len(results),'output':str(output)},indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,default=TARGET)
    run(parser.parse_args().output)
