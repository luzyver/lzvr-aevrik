"""Languages, compilers and build tools that ship relocatable Windows x64 ZIP archives."""
import hashlib, io, re, zipfile
from .common import metadata, fetch, entry, version_key
from .runtimes import _releases, _stable


def _tagged(pattern, form=r'\1'):
    def version(tag):
        match=re.fullmatch(pattern,tag)
        return match.expand(form) if match else None
    return version


def erlang():
    return _releases('erlang/otp','erlang','Erlang/OTP',
        'Erlang runtime and compiler (erl, erlc, escript). Needed to run Gleam and Elixir programs on the BEAM.',
        lambda tag:f'otp_win64_{tag.removeprefix("OTP-")}.zip',lambda _:'bin/erl.exe',
        version_of=_tagged(r'OTP-(\d+(?:\.\d+)+)'),per_page=20)


def powershell():
    return _releases('PowerShell/PowerShell','powershell','PowerShell',
        'PowerShell 7 (pwsh), installed beside the Windows PowerShell that ships with Windows.',
        lambda tag:f'PowerShell-{tag.removeprefix("v")}-win-x64.zip',lambda _:'pwsh.exe')


def nushell():
    return _releases('nushell/nushell','nushell','Nushell',
        'Shell and scripting language that works with structured data.',
        lambda tag:f'nu-{tag}-x86_64-pc-windows-msvc.zip',lambda _:'nu.exe')


def babashka():
    return _releases('babashka/babashka','babashka','Babashka',
        'Fast-starting Clojure interpreter for scripts. No JVM required.',
        lambda tag:f'babashka-{tag.removeprefix("v")}-windows-amd64.zip',lambda _:'bb.exe')


def vlang():
    return _releases('vlang/v','vlang','V',
        'V compiler with its bundled TCC backend.',
        lambda _:'v_windows.zip',lambda _:'v/v.exe','SDKs & compilers',per_page=40)


def odin():
    return _releases('odin-lang/Odin','odin','Odin',
        'Odin compiler, monthly release. Linking programs needs the Microsoft C++ Build Tools.',
        lambda tag:f'odin-windows-amd64-{tag}.zip',lambda _:'dist/odin.exe','SDKs & compilers',
        version_of=_tagged(r'dev-(\d{4})-(\d{2})',r'\1.\2'))


def tinygo():
    return _releases('tinygo-org/tinygo','tinygo','TinyGo',
        'Go compiler for microcontrollers and WebAssembly. Needs a Go installation on PATH.',
        lambda tag:f'tinygo{tag.removeprefix("v")}.windows-amd64.zip',lambda _:'tinygo/bin/tinygo.exe','SDKs & compilers')


def llvm_mingw():
    return _releases('mstorsjo/llvm-mingw','llvm-mingw','Clang (LLVM-MinGW)',
        'C and C++ toolchain built on Clang, LLD and mingw-w64 (UCRT). Also answers to gcc and g++.',
        lambda tag:f'llvm-mingw-{tag}-ucrt-x86_64.zip',lambda filename:filename.removesuffix('.zip')+'/bin/clang.exe',
        'SDKs & compilers',version_of=_tagged(r'(\d{8})'))


def cmake():
    return _releases('Kitware/CMake','cmake','CMake',
        'Build system generator for C and C++ projects.',
        lambda tag:f'cmake-{tag.removeprefix("v")}-windows-x86_64.zip',
        lambda filename:filename.removesuffix('.zip')+'/bin/cmake.exe','Development tools')


def ninja():
    return _releases('ninja-build/ninja','ninja','Ninja',
        'Small, fast build runner used by CMake and other generators.',
        lambda _:'ninja-win.zip',lambda _:'ninja.exe','Development tools')


def gcc():
    """WinLibs: newest revision of each GCC version, UCRT with POSIX threads, without the bundled LLVM."""
    url='https://api.github.com/repos/brechtsanders/winlibs_mingw/releases?per_page=40'
    newest={}
    for release in metadata(url):
        if release['prerelease'] or release['draft']:continue
        for asset in release['assets']:
            match=re.fullmatch(r'winlibs-x86_64-posix-seh-gcc-(\d+\.\d+\.\d+)-mingw-w64ucrt-[\d.]+-r(\d+)\.zip',asset['name'])
            if match and int(match[2])>newest.get(match[1],(0,))[0]:newest[match[1]]=(int(match[2]),asset)
    result=[]
    for version in sorted(newest,key=version_key,reverse=True)[:3]:
        revision,asset=newest[version]
        digest=(asset.get('digest') or '').removeprefix('sha256:') or None
        result.append(entry('gcc','GCC (WinLibs)',f'{version}-r{revision}','SDKs & compilers',
            'GNU C, C++ and Fortran compilers with mingw-w64 (UCRT), GDB and make. Use it as the C compiler for Nim.',
            asset['browser_download_url'],'mingw64/bin/gcc.exe',digest,url))
    if not result:raise ValueError('No WinLibs GCC archives found')
    return result


def perl():
    url='https://api.github.com/repos/StrawberryPerl/Perl-Dist-Strawberry/releases?per_page=12'
    result=[]
    for release in metadata(url):
        if release['prerelease'] or release['draft']:continue
        for asset in release['assets']:
            match=re.fullmatch(r'strawberry-perl-(\d+(?:\.\d+){3})-64bit-portable\.zip',asset['name'])
            if not match:continue
            digest=(asset.get('digest') or '').removeprefix('sha256:') or None
            result.append(entry('perl','Perl (Strawberry)',match[1],'Runtimes',
                'Strawberry Perl portable edition with cpan and its own C toolchain for building modules.',
                asset['browser_download_url'],'perl/bin/perl.exe',digest,url))
        if len(result)==3:break
    if not result:raise ValueError('No Strawberry Perl portable archives found')
    return result


def haxe():
    """Haxe publishes no digests and names the archive folder after the build, so each archive is read."""
    url='https://api.github.com/repos/HaxeFoundation/haxe/releases?per_page=10'
    result=[]
    for release in metadata(url):
        version=_stable(release['tag_name'])
        asset=next((a for a in release['assets'] if a['name']==f'haxe-{version}-win64.zip'),None)
        if release['prerelease'] or release['draft'] or not asset:continue
        data=fetch(asset['browser_download_url'])
        executable=next(n for n in zipfile.ZipFile(io.BytesIO(data)).namelist() if re.fullmatch(r'[^/]+/haxe\.exe',n))
        result.append(entry('haxe','Haxe',version,'SDKs & compilers',
            'Haxe compiler and haxelib. Targets JavaScript, C++, JVM, Python and more.',
            asset['browser_download_url'],executable,hashlib.sha256(data).hexdigest(),url))
        if len(result)==3:break
    if not result:raise ValueError('No Haxe Windows archives found')
    return result


def dart():
    index='https://storage.googleapis.com/storage/v1/b/dart-archive/o?prefix=channels/stable/release/&delimiter=/&maxResults=5000'
    versions=[p.split('/')[-2] for p in metadata(index).get('prefixes',[])]
    versions=sorted((v for v in versions if re.fullmatch(r'\d+\.\d+\.\d+',v)),key=version_key,reverse=True)
    lines=[];result=[]
    for version in versions:
        line=version.rsplit('.',1)[0]
        if line in lines:continue
        lines.append(line)
        base=f'https://storage.googleapis.com/dart-archive/channels/stable/release/{version}/sdk/dartsdk-windows-x64-release.zip'
        digest=fetch(base+'.sha256sum').decode().split()[0]
        result.append(entry('dart','Dart SDK',version,'SDKs & compilers',
            'Dart VM, compiler and pub package manager. Flutter is not included.',
            base,'dart-sdk/bin/dart.exe',digest,base+'.sha256sum'))
        if len(result)==3:break
    return result


def julia():
    url='https://julialang-s3.julialang.org/bin/versions.json'
    data=metadata(url);lines=[];result=[]
    for version in sorted((v for v in data if data[v]['stable']),key=version_key,reverse=True):
        line=version.rsplit('.',1)[0]
        archive=next((f for f in data[version]['files'] if f['os']=='winnt' and f['arch']=='x86_64' and f['extension']=='zip'),None)
        if line in lines or not archive:continue
        lines.append(line)
        result.append(entry('julia','Julia',version,'Runtimes',
            'Julia language with its package manager and REPL.',
            archive['url'],f'julia-{version}/bin/julia.exe',archive['sha256'],url))
        if len(result)==3:break
    return result
