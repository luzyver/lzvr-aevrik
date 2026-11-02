"""Verify severity colors, escaping and wrapped output using the desktop IPC fixture."""
from playwright.sync_api import sync_playwright, expect
from check_desktop_ui import MOCK, ROOT

with sync_playwright() as p:
 browser=p.chromium.launch()
 page=browser.new_page(viewport={"width":1280,"height":820})
 errors=[]
 page.on('pageerror',lambda error:errors.append(str(error)))
 page.add_init_script(MOCK+"""
 window.fixture.logs=[
  ['INFO','[vite] connected.'],
  ['WARN','(node:17680) [DEP0205] DeprecationWarning: module.register() is deprecated.'],
  ['INFO','┃ Local http://localhost:4321/'],
  ['WARN','[WARN] [astro-icon] Failed to load icons from "src/icons": ENOENT'],
  ['INFO','[200] / 78ms'],
  ['INFO','[404] /api/github-contributions 3196ms'],
  ['ERROR','Error: <script>unsafe</script> '+ 'long/path/'.repeat(25)],
  ['ERROR','    at handler (D:\\\\project\\\\main.js:12)']
 ].map(([level,message])=>({timestamp:Date.now(),service:'Portofolio · dev',level,message}));
 """)
 page.goto('http://127.0.0.1:1420')
 page.locator('[data-nav="logs"]').click()
 lines=page.locator('.log-lines')
 expect(lines).to_contain_text('[404] /api/github-contributions')
 expect(lines).to_contain_text('<script>unsafe</script>')
 assert lines.locator('script').count()==0
 expect(lines.locator('.log-warn')).to_have_count(2)
 expect(lines.locator('.log-error')).to_have_count(2)
 for theme in ['dark','light']:
  page.evaluate('(theme)=>{window.fixture.theme=theme;document.documentElement.dataset.theme=theme}',theme)
  colors=[lines.locator(selector).first.evaluate('(e)=>getComputedStyle(e).color') for selector in ['.log-info','.log-warn','.log-error']]
  assert len(set(colors))==3,colors
  for width in [1280,800]:
   page.set_viewport_size({'width':width,'height':820})
   assert lines.evaluate('(e)=>e.scrollWidth<=e.clientWidth+1')
 page.evaluate("window.fixture.theme='dark';document.documentElement.dataset.theme='dark'")
 page.set_viewport_size({'width':1280,'height':820})
 out=ROOT/'target'/'log-ui.png'
 out.parent.mkdir(parents=True,exist_ok=True)
 page.screenshot(path=str(out))
 page.locator('[data-action="clear-logs"]').click()
 expect(lines).to_contain_text('Process output will appear here')
 assert not errors,errors
 browser.close()
 print('Log UI: severity colors, escaping, wrapping and Clear passed; screenshot:',out)

