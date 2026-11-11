"""Build desktop/stack-logos.ts from an unpacked simple-icons package (CC0-1.0). Usage: python -I tools/generate_stack_logos.py <package dir> <out>"""
import json,re,sys
from pathlib import Path
src,out=Path(sys.argv[1]),Path(sys.argv[2])
MAP={'caddy':'caddy','nginx':'nginx','php':'php','node':'nodedotjs','python':'python','go':'go','java':'openjdk','java-jre':'openjdk','bun':'bun','deno':'deno','dotnet-sdk':'dotnet','dotnet-runtime':'dotnet','aspnet-runtime':'dotnet','zig':'zig','nim':'nim','luau':'luau','wasmtime':'webassembly','mariadb':'mariadb','mongodb':'mongodb','duckdb':'duckdb','sqlite':'sqlite','traefik':'traefikproxy','frankenphp':'php','gleam':'gleam','pocketbase':'pocketbase','uv':'uv','pnpm':'pnpm','gh':'github','hugo':'hugo','typst':'typst','erlang':'erlang','perl':'perl','julia':'julia','nushell':'nushell','babashka':'clojure','gcc':'gnu','llvm-mingw':'llvm','dart':'dart','vlang':'v','odin':'odin','haxe':'haxe','tinygo':'go','cmake':'cmake'}
# Marks simple-icons lacks, copied from the project's own artwork. A path without a colour takes the theme's tone.
# Mailpit: server/ui/favicon.svg in axllent/mailpit (MIT, (c) Ralph Slooten); its envelope is slate on light and white on dark.
CUSTOM={'mailpit':('0 0 132.292 121.708',[(None,'M12.321 0l53.861 53.918L120.365 0zM5.155 9.025l60.842 59.673 61.211-59.489-.185 36.835L66.921 70.54l15.164 12.616-8.137 5.986-41.609.184c-4.838-.022-25.877-18.34-27.185-41.255z'),
 ('00b786','M78.385 72.049l53.907-21.679-8.031 57.318-11.845-9.132c-21.727 23.171-45.255 26.289-67.997 20.837S12.281 98.39 5.155 83.8-.67 53.116 2.843 38.769c1.13 10.511-1.313 16.316 6.38 33.612 6.31 11.399 14.413 20.417 25.89 24.956 13.9 6.195 32.247 3.357 41.701-3.039l14.24-12.156z')])}
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
PATH=r'[MmLlHhVvCcSsQqTtAaZz0-9eE .,+-]+'
shapes={}
for name in sorted(set(MAP.values())):
    svg=(src/'icons'/f'{name}.svg').read_text(encoding='utf-8')
    paths=re.findall(r'<path d="([^"]+)"',svg)
    assert len(paths)==1 and 'viewBox="0 0 24 24"' in svg,name
    d=paths[0];h=hexes[name]
    assert re.fullmatch(PATH,d) and re.fullmatch(r'[0-9A-Fa-f]{6}',h),name
    plain=[t for t,bg in (('light','f2f2f6'),('dark','222228')) if contrast(h,bg)<1.8]
    shapes[name]=(d,h,plain)
    print(name,h,plain)
lines=["// Generated from simple-icons 16.37.0 (CC0-1.0): single-path brand marks on a 24x24 grid.",
"// `custom` holds marks simple-icons lacks (Mailpit: its own MIT-licensed logo, (c) Ralph Slooten).",
"// `plain` lists the themes where the brand colour is too faint on the icon tile and the text colour is used instead.",
"const shapes:Record<string,{path:string;color:string;plain:string}>={"]
for name,(d,h,plain) in shapes.items():
    lines.append(f" {name}:{{color:'#{h}',plain:'{' '.join(plain)}',path:'{d}'}},")
lines+=["};","const custom:Record<string,string>={"]
for name,(box,paths) in CUSTOM.items():
    assert re.fullmatch(r'[0-9. ]+',box) and all(re.fullmatch(PATH,d) and (h is None or re.fullmatch(r'[0-9a-f]{6}',h)) for h,d in paths),name
    lines.append(f" {name}:'<svg class=\"stack-logo\" viewBox=\"{box}\" aria-hidden=\"true\">"+''.join(f'<path class=\"stack-logo-tone\" d=\"{d}\"/>' if h is None else f'<path fill=\"#{h}\" d=\"{d}\"/>' for h,d in paths)+"</svg>',")
lines+=["};","const families:Record<string,string>={"+",".join(f"'{k}':'{v}'" for k,v in MAP.items())+"};",
"/** The product's logo for a catalog family, or '' when it has none. */",
"export function stackLogo(family:string):string {",
" if(Object.hasOwn(custom,family))return custom[family];"," const shape=shapes[families[family]??''];",
" if(!shape)return '';",
" return `<svg class=\"stack-logo ${shape.plain.split(' ').filter(Boolean).map(t=>`plain-${t}`).join(' ')}\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path fill=\"${shape.color}\" d=\"${shape.path}\"/></svg>`;",
"}",""]
out.write_text("\n".join(lines),encoding='utf-8',newline='\n')
