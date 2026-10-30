"""Regression: desktop chrome and main never scroll; only designated panes do."""
import json, sys
from pathlib import Path
from playwright.sync_api import sync_playwright, expect
sys.path.insert(0,str(Path(__file__).parent))
from check_desktop_ui import MOCK
with sync_playwright() as p:
 browser=p.chromium.launch()
 page=browser.new_page()
 page.add_init_script(MOCK)
 page.goto("http://127.0.0.1:1420")
 expect(page.locator("#engine-status")).to_have_text("Engine connected")
 page.evaluate("window.fixture.logs=Array.from({length:800},(_,i)=>({timestamp:Date.now(),service:'test',level:'INFO',message:'Line '+i}))")
 findings=[]
 for width,height in [(800,600),(1280,820),(1440,1000)]:
  page.set_viewport_size({"width":width,"height":height})
  for route in ["overview","services","projects","logs","settings","packages","domains"]:
   page.locator(f'[data-nav="{route}"]').click()
   page.mouse.move(width-5,height-60)
   page.mouse.wheel(0,5000)
   page.wait_for_timeout(100)
   metrics=page.evaluate("""()=>({rootTop:document.scrollingElement.scrollTop,
     mainTop:document.querySelector('#main').scrollTop,
     mainOverflow:document.querySelector('#main').scrollHeight-document.querySelector('#main').clientHeight,
     titleTop:document.querySelector('.titlebar').getBoundingClientRect().top,
     statusBottom:document.querySelector('.statusbar').getBoundingClientRect().bottom,
     height:innerHeight})""")
   if metrics["mainTop"] or metrics["rootTop"] or metrics["mainOverflow"]>1 or abs(metrics["statusBottom"]-height)>1 or metrics["titleTop"]!=0:
    findings.append({"size":[width,height],"route":route,**metrics})
 page.locator('[data-nav="logs"]').click()
 expect(page.locator('.log-lines')).to_contain_text('Line 799')
 page.locator('.log-lines').evaluate('(e)=>e.scrollTop=e.scrollHeight')
 assert page.locator('.log-lines').evaluate('(e)=>e.scrollTop>0')
 assert page.locator('#main').evaluate('(e)=>e.scrollTop===0')
 browser.close()
 print(json.dumps(findings,indent=2))
 assert not findings,"Desktop main/root still scroll or overflow"
 print("Fixed desktop viewport regression passed.")
