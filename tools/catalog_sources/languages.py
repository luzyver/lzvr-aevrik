"""Language runtimes published by their upstream projects."""
import re
from .common import metadata, fetch, entry, version_key, selected_versions

def php():
    url='https://windows.php.net/downloads/releases/releases.json'
    result=[]
    for line,release in metadata(url).items():
        build=next((v['zip'] for k,v in release.items() if k.startswith('nts-') and k.endswith('-x64')),None)
        if not build:continue
        result.append(entry('php','PHP',release['version'],'Language','NTS CLI and FastCGI. Requires Microsoft Visual C++ runtime.',
            'https://windows.php.net/downloads/releases/'+build['path'],'php.exe',build['sha256'],url,channel='Legacy' if version_key(line)<(8,2) else 'Release'))
    for version,toolset in [('5.6.40','vc11'),('7.0.33','vc14'),('7.1.33','vc14'),('7.2.34','vc15'),('7.3.33','vc15')]:
        archive=f'https://windows.php.net/downloads/releases/archives/php-{version}-nts-Win32-{toolset}-x64.zip'
        result.append(entry('php','PHP',version,'Language','Legacy NTS runtime for compatibility testing. Requires its matching Visual C++ runtime.',archive,'php.exe',channel='Legacy'))
    return result

def node():
    url='https://nodejs.org/dist/index.json';result=[];counts={}
    for release in metadata(url):
        version=release['version'].removeprefix('v');major=version.split('.')[0]
        if not re.fullmatch(r'\d+\.\d+\.\d+',version) or major not in ['12','14','16','18','20','22','24','26'] or counts.get(major,0)>=5 or 'win-x64-zip' not in release['files']:continue
        name=f'node-v{version}-win-x64.zip';manifest=f'https://nodejs.org/dist/v{version}/SHASUMS256.txt'
        sha=next(line.split()[0] for line in fetch(manifest).decode().splitlines() if line.split()[-1]==name)
        result.append(entry('node','Node.js',version,'Language','JavaScript runtime with npm. Configure a project script and working directory.',f'https://nodejs.org/dist/v{version}/{name}',f'node-v{version}-win-x64/node.exe',sha,manifest,channel='Legacy' if int(major)<=20 else 'LTS' if release['lts'] else 'Release'))
        counts[major]=counts.get(major,0)+1
    return result

def go():
    url='https://go.dev/dl/?mode=json&include=all';result=[];counts={}
    for release in metadata(url):
        version=release['version'].removeprefix('go');line='.'.join(version.split('.')[:2])
        if not release['stable'] or line not in [f'1.{i}' for i in range(20,28)] or counts.get(line,0)>=2:continue
        archive=next((f for f in release['files'] if f['os']=='windows' and f['arch']=='amd64' and f['filename'].endswith('.zip')),None)
        if not archive:continue
        result.append(entry('go','Go',version,'Language','Go compiler and toolchain. Use its executable path or configure PATH yourself.',f"https://go.dev/dl/{archive['filename']}",'go/bin/go.exe',archive['sha256'],url))
        counts[line]=counts.get(line,0)+1
    return result

def python():
    url='https://api.nuget.org/v3-flatcontainer/python/index.json'
    versions=selected_versions([v for v in metadata(url)['versions'] if re.fullmatch(r'3\.\d+\.\d+',v)],1,[f'3.{i}' for i in range(8,16)])
    return [entry('python','Python',v,'Language','Official CPython NuGet distribution for isolated development. Use python -m pip for packages.',
        f'https://api.nuget.org/v3-flatcontainer/python/{v}/python.{v}.nupkg','tools/python.exe',source_url=url,channel='Legacy' if version_key(v)<(3,11) else 'Release') for v in versions]

def java():
    result=[]
    for major in [8,11,17,21,25]:
        url=f'https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture=x64&image_type=jdk&os=windows'
        release=metadata(url)[0];archive=release['binary']['package'];name=release['release_name']
        result.append(entry('java','Java (Temurin)',name.removeprefix('jdk-').removeprefix('jdk'),'Language','OpenJDK toolchain from Eclipse Adoptium. JAVA_HOME and global PATH are unchanged.',archive['link'],name+'/bin/java.exe',archive['checksum'],url,channel='LTS'))
    return result
