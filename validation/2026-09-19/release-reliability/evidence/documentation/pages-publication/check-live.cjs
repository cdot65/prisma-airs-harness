const fs=require('node:fs');
const crypto=require('node:crypto');
const {chromium}=require('/home/cdot/development/cdot65/airs-reliability-docs-20260919/node_modules/playwright');
const output='/tmp/airs-autonomous-20260919/evidence/documentation/pages-publication';
const base='https://cdot65.github.io/prisma-airs-reference-architecture/';
const expected={
 '': ['Sign in and connect ServiceNow'],
 'learn/login/': ['airs-harness 0.1.0-alpha.22.mcp.3','is published and ready for local testing','npm install -g airs-harness@0.1.0-alpha.22.mcp.3 --registry=https://npm.example.com','mcp_oauth_credentials_store = "keyring"','does not enforce this setting automatically','Start new conversation','Send connectivity check','remain separate attended checks','does not create a gateway workspace or require matching names','Workspace-key replacement remains a shell operation'],
 'learn/harness/': ['0.1.0-alpha.22.mcp.3','recovery commands scoped to the selected environment','does not enforce it automatically','Start new conversation'],
 'learn/troubleshooting/': ['Verify gateway access','Secret Service'],
 'learn/evidence/': ['0.1.0-alpha.22.mcp.3 is published','Fresh anonymous registry installations passed isolated acceptance on Linux x64, Linux ARM64 and Apple Silicon','they do not establish live-account acceptance','are deferred to attended testing','no full workspace pass is claimed']
};
function requireThat(value,why){if(!value)throw new Error(why);}
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:'/usr/bin/chromium'});
 const context=await browser.newContext({viewport:{width:1440,height:1080}});
 const page=await context.newPage();const browserErrors=[],requestFailures=[],routes=[];
 page.on('pageerror',error=>browserErrors.push(String(error)));
 page.on('console',message=>{if(message.type()==='error')browserErrors.push(message.text());});
 page.on('requestfailed',r=>requestFailures.push({url:r.url(),failure:r.failure()}));
 try{
  for(const [route,phrases] of Object.entries(expected)){
   const url=base+route;const response=await page.goto(url,{waitUntil:'networkidle',timeout:60000});
   requireThat(response.status()===200,`${url}: HTTP ${response.status()}`);
   const body=await response.body();const text=(await page.locator('main').innerText()).replace(/\s+/g,' ');
   for(const phrase of phrases)requireThat(text.includes(phrase),`${url}: missing ${phrase}`);
   requireThat(!text.includes('publication and native acceptance of that exact version are pending'),`${url}: stale pending release`);
   requireThat(!text.includes('publication and three-platform installed acceptance remain pending'),`${url}: stale pending release`);
   requireThat(!text.includes('.cdot.io')&&!text.includes('/home/cdot'),`${url}: private exported data`);
   const name=route?route.split('/')[1]:'root';
   fs.writeFileSync(`${output}/${name}.html`,body);
   routes.push({url,final_url:page.url(),http_status:response.status(),sha256:crypto.createHash('sha256').update(body).digest('hex'),expected_phrases:phrases});
  }
  await page.goto(base,{waitUntil:'networkidle'});
  await page.getByRole('link',{name:'Sign in and connect ServiceNow'}).click();
  await page.waitForURL(base+'learn/login/#sso-to-servicenow-a-complete-first-session');
  await page.locator('#sso-to-servicenow-a-complete-first-session').waitFor({state:'visible',timeout:15000});
  await page.screenshot({path:`${output}/live-login.png`,fullPage:false});
  requireThat(browserErrors.length===0,'Browser errors: '+JSON.stringify(browserErrors));
  requireThat(requestFailures.length===0,'Failed browser requests: '+JSON.stringify(requestFailures));
  fs.writeFileSync(`${output}/LIVE-ROUTES.json`,JSON.stringify({verified_at_utc:new Date().toISOString(),source_commit:'9d328da14b0abf3e8b3823cebb4f92a4bd4a8641',workflow_run:35420045886,routes,login_anchor:page.url(),browser_errors:browserErrors,request_failures:requestFailures,live_identity_checks_performed:false},null,2)+'\n');
  console.log('PASS: 5 live HTTP200 routes, exact release and identity boundaries, homepage login anchor, no browser errors/request failures.');
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exit(1);});
