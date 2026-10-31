"""Check every catalog URL resolves to a ZIP/NuGet archive within installer limits."""
import concurrent.futures, json, sys, time, urllib.request
from pathlib import Path

def check(package):
 for attempt in range(3):
  try:
   request=urllib.request.Request(package['url'],headers={'User-Agent':'Aevrik-Catalog/0.4','Range':'bytes=0-3','Accept-Encoding':'identity'})
   with urllib.request.urlopen(request,timeout=30) as response:
    magic=response.read(4)
    assert magic==b'PK\x03\x04',f'Not a ZIP archive: {magic!r}'
    size=int(response.headers.get('Content-Range','').split('/')[-1]) if response.headers.get('Content-Range') else int(response.headers.get('Content-Length','0'))
    assert not size or size<=2*1024*1024*1024,f'Archive exceeds installer limit: {size}'
    return {'id':package['id'],'version':package['version'],'url':package['url'],'status':'passed','archiveBytes':size,'digest':package.get('sha512') or package['sha256']}
  except Exception as error:
   if attempt==2:return {'id':package['id'],'url':package['url'],'status':'failed','error':str(error)}
   time.sleep(1)

if __name__=='__main__':
 path=Path(sys.argv[1] if len(sys.argv)>1 else 'crates/aevrik-engine/src/packages/catalog.json')
 data=json.loads(path.read_text());report=[]
 with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
  for result in pool.map(check,data):
   report.append(result);print(result['id'],result['status'],result.get('error',''),flush=True)
 target=Path('previews/desktop/catalog-url-verification-0.1.0-beta-1.json')
 target.write_text(json.dumps(report,indent=2))
 failed=[r for r in report if r['status']!='passed']
 print(f'{len(report)-len(failed)}/{len(report)} archive URLs passed')
 if failed:sys.exit(1)
