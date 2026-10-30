"""Servers and tools with official Windows archives."""
import re
from .common import metadata, fetch, entry, selected_versions

def github():
    result=[]
    products=[
      ('caddy','Caddy','caddyserver/caddy',r'caddy_.*_windows_amd64.zip','caddy.exe','Web server','Web server and reverse proxy. Configure a Caddyfile before starting.'),
      ('bun','Bun','oven-sh/bun',r'bun-windows-x64-baseline.zip','bun-windows-x64-baseline/bun.exe','Language','JavaScript runtime, package manager and bundler. Baseline x64 CPU build.'),
      ('deno','Deno','denoland/deno',r'deno-x86_64-pc-windows-msvc.zip','deno.exe','Language','JavaScript and TypeScript runtime. Configure your script and permissions.'),
      ('mailpit','Mailpit','axllent/mailpit',r'mailpit-windows-amd64.zip','mailpit.exe','Mail','Local SMTP capture and web inbox. Configure listening addresses and ports before starting.')]
    for family,name,repo,pattern,exe,category,description in products:
        url=f'https://api.github.com/repos/{repo}/releases?per_page=12';count=0
        for release in metadata(url):
            version=release['tag_name'].removeprefix('bun-').removeprefix('v')
            if release['prerelease'] or release['draft'] or not re.fullmatch(r'\d+\.\d+\.\d+',version):continue
            archive=next((a for a in release['assets'] if re.fullmatch(pattern,a['name'])),None)
            if not archive:continue
            digest=(archive.get('digest') or '').removeprefix('sha256:') or None
            result.append(entry(family,name,version,category,description,archive['browser_download_url'],exe,digest,url))
            count+=1
            if count==6:break
    return result

def maria():
    url='https://archive.mariadb.org/'
    versions=re.findall(r'href="mariadb-(\d+\.\d+\.\d+)/"',fetch(url).decode())
    versions=selected_versions(versions,2,['10.6','10.11','11.4','11.8','12.0','12.1','12.2'])
    result=[]
    for version in versions:
        base=f'https://archive.mariadb.org/mariadb-{version}/winx64-packages/'
        manifest=base+'sha256sums.txt';filename=f'mariadb-{version}-winx64.zip'
        sha=next((line.split()[0] for line in fetch(manifest).decode().splitlines() if line.split() and line.split()[-1].lstrip('*').removeprefix('./')==filename),None)
        if not sha:
            print(f'MariaDB {version}: publisher has no matching Windows ZIP; omitted',flush=True)
            continue
        result.append(entry('mariadb','MariaDB',version,'Database','Server binaries only. Initialize a data directory separately and use database shutdown before closing Aevrik.',base+filename,f'mariadb-{version}-winx64/bin/mariadbd.exe',sha,manifest))
    return result

def nginx():
    url='https://nginx.org/download/'
    versions=re.findall(r'href="nginx-(\d+\.\d+\.\d+)\.zip"',fetch(url).decode())
    versions=selected_versions(versions,1,['1.24','1.26','1.28','1.30','1.31'])
    return [entry('nginx','nginx',v,'Web server','Official Windows x86 build (runs on x64). Set working directory to the nginx folder and use foreground mode.',f'https://nginx.org/download/nginx-{v}.zip',f'nginx-{v}/nginx.exe',source_url=url,architecture='x86') for v in versions]
