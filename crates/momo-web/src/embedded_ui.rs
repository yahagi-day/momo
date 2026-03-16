use axum::response::Html;

pub async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>MOMO</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,-apple-system,sans-serif;background:#1a1a2e;color:#e0e0e0;line-height:1.5}
button{cursor:pointer;border:none;padding:8px 16px;border-radius:6px;font-size:.9rem;font-weight:500;transition:background .2s}
button:disabled{opacity:.5;cursor:not-allowed}
.app{max-width:1200px;margin:0 auto;padding:20px}
.status-bar{display:flex;align-items:center;justify-content:space-between;background:#16213e;padding:16px 20px;border-radius:8px;margin-bottom:20px;flex-wrap:wrap;gap:12px}
.status-bar h1{font-size:1.4rem;font-weight:700}
.info{display:flex;align-items:center;gap:12px}
.badge{display:inline-block;padding:4px 12px;border-radius:12px;font-size:.8rem;font-weight:600;text-transform:uppercase}
.badge.stopped{background:#444}.badge.running{background:#2d6a4f}.badge.starting,.badge.stopping{background:#7c5e10}.badge.error{background:#a33}
.btn-start{background:#2d6a4f;color:#fff}.btn-start:hover:not(:disabled){background:#3a8c66}
.btn-stop{background:#a33;color:#fff}.btn-stop:hover:not(:disabled){background:#c44}
.btn-action{background:#334;color:#e0e0e0}.btn-action:hover:not(:disabled){background:#445}
.main{display:grid;grid-template-columns:1fr 1fr;gap:20px}
@media(max-width:768px){.main{grid-template-columns:1fr}}
.panel{background:#16213e;border-radius:8px;padding:16px}
.panel h2{font-size:1.1rem;margin-bottom:12px;padding-bottom:8px;border-bottom:1px solid #333}
.preview-img{width:100%;border-radius:4px;background:#000;aspect-ratio:16/9;object-fit:contain;display:block}
.output-card{background:#0f3460;border-radius:6px;padding:12px;margin-bottom:8px}
.output-card h3{font-size:.95rem;margin-bottom:8px}
.fields{display:grid;grid-template-columns:auto 1fr;gap:4px 12px;font-size:.85rem}
.fields label{color:#aaa}
.flip-row{display:flex;align-items:center;gap:8px;margin-top:8px;font-size:.85rem}
.flip-row input[type=checkbox]{width:auto}
.config-actions{display:flex;gap:8px;margin-top:20px;flex-wrap:wrap}
.error-msg{color:#f88;font-size:.85rem;margin-top:4px}
.msg{color:#6c6;font-size:.85rem}
#fps{font-variant-numeric:tabular-nums}
.no-preview{display:flex;align-items:center;justify-content:center;width:100%;aspect-ratio:16/9;background:#000;border-radius:4px;color:#666;font-size:.9rem}
</style>
</head>
<body>
<div class="app">
  <div class="status-bar">
    <h1>MOMO</h1>
    <div class="info">
      <span id="fps"></span>
      <span class="badge stopped" id="badge">Stopped</span>
      <button class="btn-start" id="startBtn" onclick="doStart()">Start</button>
      <button class="btn-stop" id="stopBtn" onclick="doStop()" style="display:none">Stop</button>
    </div>
  </div>
  <div id="error" class="error-msg" style="margin-bottom:12px"></div>
  <div class="main">
    <div class="panel">
      <h2>Input</h2>
      <p id="inputLabel" style="margin-bottom:12px;font-size:.9rem;color:#aaa">-</p>
      <div id="previewArea"><div class="no-preview">Pipeline stopped</div></div>
    </div>
    <div class="panel">
      <h2>Outputs</h2>
      <div id="outputList"></div>
    </div>
  </div>
  <div class="config-actions">
    <button class="btn-action" onclick="doSave()">Save Config</button>
    <button class="btn-action" onclick="doLoad()">Load Config</button>
    <span id="configMsg" class="msg"></span>
    <span id="configErr" class="error-msg"></span>
  </div>
</div>
<script>
let state='Stopped', config=null;

async function api(url,opts){
  const r=await fetch(url,opts);
  if(!r.ok){const b=await r.json().catch(()=>({error:r.statusText}));throw new Error(b.error||r.statusText)}
  return r.json();
}
function setState(s){
  state=s;
  const b=document.getElementById('badge');
  b.textContent=s;b.className='badge '+s.toLowerCase();
  const run=s==='Running';
  document.getElementById('startBtn').style.display=run?'none':'';
  document.getElementById('stopBtn').style.display=run?'':'none';
  document.getElementById('startBtn').disabled=s==='Starting'||s==='Stopping';
  document.getElementById('stopBtn').disabled=s==='Starting'||s==='Stopping';
  updatePreview();
}
function updatePreview(){
  const area=document.getElementById('previewArea');
  if(state==='Running'){
    area.innerHTML='<img class="preview-img" src="/api/preview/input" alt="Preview">';
  }else{
    area.innerHTML='<div class="no-preview">Pipeline stopped</div>';
    document.getElementById('fps').textContent='';
  }
}
function setInputLabel(){
  if(!config){document.getElementById('inputLabel').textContent='-';return}
  const i=config.input;
  let t='';
  if(i.type==='Mock')t=`Mock (${i.width}x${i.height} @ ${i.fps}fps)`;
  else if(i.type==='DeckLink')t=`DeckLink #${i.device_index}`;
  else if(i.type==='Uvc')t=`UVC (${i.device_path})`;
  document.getElementById('inputLabel').textContent=t;
}
function renderOutputs(){
  const el=document.getElementById('outputList');
  if(!config||!config.outputs.length){el.innerHTML='<p style="color:#666">No outputs</p>';return}
  el.innerHTML=config.outputs.map(o=>`
    <div class="output-card">
      <h3>${esc(o.name)} (${esc(o.id)})</h3>
      <div class="fields">
        <label>Mode:</label><span>${esc(o.display_mode)}</span>
        <label>Format:</label><span>${esc(o.pixel_format)}</span>
        <label>Device:</label><span>#${o.device_index}</span>
      </div>
      <div class="flip-row">
        <input type="checkbox" id="fh_${o.id}" ${o.transform.flip.horizontal?'checked':''}>
        <label>Flip H</label>
        <input type="checkbox" id="fv_${o.id}" ${o.transform.flip.vertical?'checked':''}>
        <label>Flip V</label>
        <button class="btn-action" onclick="applyFlip('${esc(o.id)}')">Apply</button>
        <span id="ferr_${o.id}" class="error-msg"></span>
      </div>
    </div>
  `).join('');
}
function esc(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;')}
async function fetchConfig(){
  try{config=await api('/api/config');setInputLabel();renderOutputs()}catch(e){}
}
async function doStart(){
  try{document.getElementById('error').textContent='';await api('/api/pipeline/start',{method:'POST'})}
  catch(e){document.getElementById('error').textContent=e.message}
}
async function doStop(){
  try{document.getElementById('error').textContent='';document.getElementById('fps').textContent='';await api('/api/pipeline/stop',{method:'POST'})}
  catch(e){document.getElementById('error').textContent=e.message}
}
async function doSave(){
  try{document.getElementById('configErr').textContent='';await api('/api/config/save',{method:'POST'});
    document.getElementById('configMsg').textContent='Saved';setTimeout(()=>document.getElementById('configMsg').textContent='',2000)}
  catch(e){document.getElementById('configErr').textContent=e.message}
}
async function doLoad(){
  const p=prompt('Config file path:','config.json');if(!p)return;
  try{document.getElementById('configErr').textContent='';await api('/api/config/load',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({path:p})});
    document.getElementById('configMsg').textContent='Loaded';fetchConfig();setTimeout(()=>document.getElementById('configMsg').textContent='',2000)}
  catch(e){document.getElementById('configErr').textContent=e.message}
}
async function applyFlip(id){
  const h=document.getElementById('fh_'+id).checked,v=document.getElementById('fv_'+id).checked;
  const errEl=document.getElementById('ferr_'+id);
  try{errEl.textContent='';
    const o=config.outputs.find(x=>x.id===id);
    await api('/api/config/output/'+id,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({crop:o?o.transform.crop:null,flip:{horizontal:h,vertical:v}})});
    fetchConfig()}
  catch(e){errEl.textContent=e.message}
}
// WebSocket
function connectWS(){
  const proto=location.protocol==='https:'?'wss:':'ws:';
  const ws=new WebSocket(proto+'//'+location.host+'/ws/status');
  ws.onmessage=e=>{try{const ev=JSON.parse(e.data);
    if(ev.type==='StateChanged'&&ev.state)setState(ev.state);
    if(ev.type==='FpsUpdate'&&ev.fps!==undefined)document.getElementById('fps').textContent=ev.fps.toFixed(1)+' fps';
    if(ev.type==='ConfigChanged')fetchConfig();
    if(ev.type==='Error')document.getElementById('error').textContent=ev.message||'';
  }catch(ex){}};
  ws.onclose=()=>setTimeout(connectWS,2000);
  ws.onerror=()=>ws.close();
}
// Init
api('/api/status').then(d=>setState(d.state)).catch(()=>{});
fetchConfig();
connectWS();
</script>
</body>
</html>"##;
