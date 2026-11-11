"""Standalone language and WebAssembly runtimes with official Windows ZIP releases."""
import re
from .common import metadata, entry


def _stable(tag):
    version=tag.removeprefix('v')
    return version if re.fullmatch(r'\d+(?:\.\d+)+',version) else None


def _releases(repo, family, name, description, asset_name, executable, category='Runtimes', version_of=_stable, per_page=10):
    url=f'https://api.github.com/repos/{repo}/releases?per_page={per_page}'
    result=[]
    for release in metadata(url):
        version=version_of(release['tag_name'])
        if release['prerelease'] or release['draft'] or not version:continue
        filename=asset_name(release['tag_name'])
        asset=next((a for a in release['assets'] if a['name']==filename),None)
        if not asset:continue
        digest=(asset.get('digest') or '').removeprefix('sha256:') or None
        result.append(entry(family,name,version,category,description,
            asset['browser_download_url'],executable(filename),digest,url))
        if len(result)==3:break
    if not result:raise ValueError(f'No stable Windows ZIP releases found for {name}')
    return result


def wasmtime():
    return _releases('bytecodealliance/wasmtime','wasmtime','Wasmtime',
        'Run WebAssembly and WASI programs from the command line. Global PATH is unchanged.',
        lambda tag:f'wasmtime-{tag}-x86_64-windows.zip',
        lambda filename:filename.removesuffix('.zip')+'/wasmtime.exe')


def luau():
    return _releases('luau-lang/luau','luau','Luau',
        'Standalone Luau scripting runtime and analysis tools. Luau is derived from Lua 5.1; it is not a drop-in Lua replacement.',
        lambda _:'luau-windows.zip',lambda _:'luau.exe')
