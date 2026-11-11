"""Build desktop/stack-logos.ts from an unpacked simple-icons package (CC0-1.0). Usage: python -I tools/generate_stack_logos.py <package dir> <out>"""
import json,re,sys
from pathlib import Path
src,out=Path(sys.argv[1]),Path(sys.argv[2])
MAP={'caddy':'caddy','nginx':'nginx','php':'php','node':'nodedotjs','python':'python','go':'go','java':'openjdk','java-jre':'openjdk','bun':'bun','deno':'deno','dotnet-sdk':'dotnet','dotnet-runtime':'dotnet','aspnet-runtime':'dotnet','zig':'zig','nim':'nim','luau':'luau','wasmtime':'webassembly','mariadb':'mariadb','mongodb':'mongodb','duckdb':'duckdb','sqlite':'sqlite'}
data=json.loads((src/'data'/'simple-icons.json').read_text(encoding='utf-8'))
data=data['icons'] if isinstance(data,dict) else data
def slug(i):
    return i.get('slug') or re.sub(r'[^a-z0-9]','',i['title'].lower().replace('+','plus').replace('.','dot').replace('&','and'))
hexes={slug(i):i['hex'] for i in data}
def lum(h):
    c=[int(h[i:i+2],16)/255 for i in (0,2,4)]
    c=[v/12.92 if v<=.03928 else ((v+.055)/1.055)**2.4 for v in c]
    return .2126*c[0]+.7152*c[1]+.0722*c[2]
def contrast(a,b):
    a,b=sorted((lum(a),lum(b)),reverse=True);return (a+.05)/(b+.05)
shapes={}
for name in sorted(set(MAP.values())):
    svg=(src/'icons'/f'{name}.svg').read_text(encoding='utf-8')
    paths=re.findall(r'<path d="([^"]+)"',svg)
    assert len(paths)==1 and 'viewBox="0 0 24 24"' in svg,name
    d=paths[0];h=hexes[name]
    assert re.fullmatch(r'[MmLlHhVvCcSsQqTtAaZz0-9eE .,+-]+',d) and re.fullmatch(r'[0-9A-Fa-f]{6}',h),name
    plain=[t for t,bg in (('light','f2f2f6'),('dark','222228')) if contrast(h,bg)<1.8]
    shapes[name]=(d,h,plain)
    print(name,h,plain)
lines=["// Generated from simple-icons 16.37.0 (CC0-1.0): single-path brand marks on a 24x24 grid.",
"// `plain` lists the themes where the brand colour is too faint on the icon tile and the text colour is used instead.",
"const shapes:Record<string,{path:string;color:string;plain:string}>={"]
for name,(d,h,plain) in shapes.items():
    lines.append(f" {name}:{{color:'#{h}',plain:'{' '.join(plain)}',path:'{d}'}},")
lines+=["};","const families:Record<string,string>={"+",".join(f"'{k}':'{v}'" for k,v in MAP.items())+"};",
"/** The product's logo for a catalog family, or '' when it has none. */",
"export function stackLogo(family:string):string {",
" const shape=shapes[families[family]??''];",
" if(!shape)return '';",
" return `<svg class=\"stack-logo ${shape.plain.split(' ').filter(Boolean).map(t=>`plain-${t}`).join(' ')}\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path fill=\"${shape.color}\" d=\"${shape.path}\"/></svg>`;",
"}",""]
out.write_text("\n".join(lines),encoding='utf-8',newline='\n')
