"""Additional databases using official Windows ZIP archives."""
import re
from .common import metadata, fetch, entry


def mongo():
    url='https://downloads.mongodb.org/full.json';result=[];counts={}
    for release in metadata(url)['versions']:
        version=release['version'];line='.'.join(version.split('.')[:2])
        if not re.fullmatch(r'\d+\.\d+\.\d+',version) or line not in ['7.0','8.0','8.3','9.0'] or counts.get(line,0)>=2:continue
        archive=next((d['archive'] for d in release['downloads'] if d.get('target')=='windows' and d.get('edition')=='base' and d.get('arch')=='x86_64'),None)
        if not archive:continue
        filename=archive['url'].split('/')[-1].removesuffix('.zip')
        result.append(entry('mongodb','MongoDB',version,'Database','Community server binaries. Keep --dbpath outside the runtime installation; configure and shut down the database separately.',archive['url'],filename+'/bin/mongod.exe',archive['sha256'],url))
        counts[line]=counts.get(line,0)+1
    return result


def duckdb():
    url='https://api.github.com/repos/duckdb/duckdb/releases?per_page=20';result=[]
    for release in metadata(url):
        if release['prerelease'] or release['draft']:continue
        asset=next((a for a in release['assets'] if a['name']=='duckdb_cli-windows-amd64.zip'),None)
        if not asset:continue
        digest=(asset.get('digest') or '').removeprefix('sha256:') or None
        result.append(entry('duckdb','DuckDB',release['tag_name'].removeprefix('v'),'Database','Embedded analytics database CLI. Keep database files outside the runtime folder. ZIP-compatible release selection.',asset['browser_download_url'],'duckdb.exe',digest,url))
        if len(result)==6:break
    return result


def sqlite():
    url='https://www.sqlite.org/download.html';page=fetch(url).decode();result=[]
    rows=re.findall(r'PRODUCT,([^,]+),([^,]+sqlite-tools-win-x64-[^,]+),([^,]+),([^\r\n]+)',page)
    for version,relative,_,_ in rows:
        result.append(entry('sqlite','SQLite',version,'Database','Official SQLite command-line tools. Keep database files outside the runtime installation.', 'https://www.sqlite.org/'+relative,'sqlite3.exe',source_url=url))
    # Published earlier stable release, retained for compatibility. Availability/hash are checked when generating.
    result.append(entry('sqlite','SQLite','3.53.0','Database','Official SQLite command-line tools. Keep database files outside the runtime installation.','https://www.sqlite.org/2026/sqlite-tools-win-x64-3530000.zip','sqlite3.exe',source_url='https://www.sqlite.org/releaselog/3_53_0.html'))
    return result
