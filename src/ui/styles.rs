// src/ui/styles.rs
pub const STYLES: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=DM+Sans:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap');
:root{
--bg0:#07080a;--bg1:#0f1114;--bg2:#181a1e;--bg3:#1e2024;
--bd:#25272b;--bd2:#3a3d42;
--t1:#ecedef;--t2:#95989e;--t3:#5c6066;
--c-blue:#58a6ff;--c-green:#3fb950;--c-amber:#d29922;
--c-purple:#a371f7;--c-red:#f85149;--c-muted:#636970;
--r-lg:12px;--r-md:8px;--r-sm:6px;
--ff:'DM Sans',-apple-system,sans-serif;
--fm:'JetBrains Mono','SF Mono',monospace;
--ease:cubic-bezier(.4,0,.2,1);
}
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:var(--ff);background:var(--bg0);color:var(--t1);overflow:hidden;height:100vh;user-select:none;-webkit-user-select:none}
.shell{display:flex;height:100vh;overflow:hidden}
.sidebar{width:272px;min-width:272px;background:var(--bg1);border-right:1px solid var(--bd);display:flex;flex-direction:column;padding:28px 18px;gap:28px}
.brand{display:flex;align-items:center;gap:10px;font-weight:800;font-size:19px;letter-spacing:-.5px;color:var(--t1)}
.brand svg{color:var(--c-purple)}
.nav{display:flex;flex-direction:column;gap:3px}
.nav-label{font-size:10px;font-weight:800;color:var(--t3);text-transform:uppercase;letter-spacing:1.5px;margin-bottom:10px}
.nav-btn{display:block;width:100%;text-align:left;padding:9px 13px;border-radius:var(--r-md);background:0 0;border:none;color:var(--t2);font-size:13.5px;font-weight:500;font-family:var(--ff);cursor:pointer;transition:all .15s var(--ease)}
.nav-btn:hover{background:var(--bg2);color:var(--t1)}
.nav-btn.on{background:var(--bg2);color:var(--t1);font-weight:600;border:1px solid var(--bd)}
.stats-area{margin-top:auto}
.stat-list{display:flex;flex-direction:column;gap:7px}
.stat{display:flex;justify-content:space-between;align-items:center;padding:9px 13px;background:var(--bg2);border-radius:var(--r-md);border:1px solid var(--bd);font-size:13px}
.stat-lbl{color:var(--t2)}
.stat-val{font-weight:700;font-family:var(--fm);font-size:14px}
.sidebar-foot{margin-top:14px;display:flex;flex-direction:column;gap:7px}
.new-issue-btn{display:block;width:100%;padding:11px;border-radius:var(--r-md);border:1px solid var(--c-purple);background:rgba(163,113,247,.1);color:var(--c-purple);font-weight:700;font-size:13px;font-family:var(--ff);cursor:pointer;transition:all .15s var(--ease)}
.new-issue-btn:hover{background:rgba(163,113,247,.2);border-color:var(--c-purple)}
.sync{text-align:center;padding:11px;border-radius:var(--r-md);background:rgba(255,255,255,.02);border:1px solid var(--bd);font-size:12px;color:var(--t3)}
.sync.dirty{border-color:var(--c-amber);color:var(--c-amber)}
.save-btn{display:block;width:100%;padding:9px;border-radius:var(--r-md);border:none;background:var(--c-green);color:#000;font-weight:700;font-size:13px;font-family:var(--ff);cursor:pointer}
.save-btn:hover{opacity:.85}
.main{flex:1;display:flex;flex-direction:column;overflow:hidden}
.topbar{height:66px;padding:0 36px;display:flex;align-items:center;justify-content:space-between;gap:16px;border-bottom:1px solid var(--bd);background:rgba(7,8,10,.85);backdrop-filter:blur(20px);flex-shrink:0}
.search-box{flex:1;max-width:440px}
.search{width:100%;background:var(--bg2);border:1px solid var(--bd);padding:10px 18px;border-radius:36px;color:var(--t1);font-size:14px;font-family:var(--ff);outline:none;transition:border-color .2s var(--ease)}
.search:focus{border-color:var(--c-blue);background:var(--bg1)}
.search::placeholder{color:var(--t3)}
.topbar-new-btn{padding:8px 16px;border-radius:var(--r-md);border:none;background:var(--c-purple);color:#fff;font-weight:600;font-size:13px;font-family:var(--ff);cursor:pointer;transition:opacity .15s var(--ease)}
.topbar-new-btn:hover{opacity:.85}
.count-pill{font-size:12px;font-weight:600;color:var(--t3);padding:5px 13px;background:var(--bg2);border-radius:18px;border:1px solid var(--bd)}
.content{flex:1;overflow-y:auto;scroll-behavior:smooth}
.feed{padding:36px;touch-action:none}
.feed-inner{max-width:840px;margin:0 auto}
.sec-hdr{margin:44px 0 14px;display:flex;align-items:center;gap:14px;font-size:11px;font-weight:800;color:var(--c-purple);text-transform:uppercase;letter-spacing:2px}
.sec-line{flex:1;height:1px;background:var(--bd)}
.sec-ct{font-family:var(--fm);font-size:11px;color:var(--t3)}
"#;

pub const STYLES_CARD: &str = r#"
.card{margin-bottom:9px;border-radius:var(--r-lg);background:var(--bg1);border:1px solid var(--bd);overflow:hidden;transition:border-color .2s var(--ease);cursor:grab;transform-origin:50% -30%;will-change:transform}
.card:hover{border-color:var(--bd2)}
.card.active{border-color:var(--c-blue)}
.card-hdr{padding:16px 22px;display:flex;align-items:center;gap:18px;cursor:pointer;user-select:none}
.card-hdr:hover{background:var(--bg3)}
.cid{font-family:var(--fm);font-size:12px;color:var(--t3);min-width:40px}
.ctitle{flex:1;font-weight:600;font-size:14.5px;color:var(--t1)}
.badge{font-size:9px;font-weight:800;padding:3px 9px;border-radius:18px;text-transform:uppercase;letter-spacing:.7px;white-space:nowrap}
.b-open{background:rgba(88,166,255,.1);color:var(--c-blue);border:1px solid rgba(88,166,255,.2)}
.b-in-progress{background:rgba(210,153,34,.1);color:var(--c-amber);border:1px solid rgba(210,153,34,.2)}
.b-done{background:rgba(63,185,80,.1);color:var(--c-green);border:1px solid rgba(63,185,80,.2)}
.b-descoped{background:rgba(99,105,112,.1);color:var(--c-muted);border:1px solid rgba(99,105,112,.2)}
.card-body{padding:0 22px 22px;border-top:1px solid var(--bd)}
.detail-grid{display:grid;grid-template-columns:1fr 240px;gap:28px;margin-top:18px}
.detail-l,.detail-r{}
.fgroup{margin-bottom:16px}
.flbl{font-size:9px;font-weight:800;color:var(--t3);text-transform:uppercase;letter-spacing:1.5px;margin-bottom:7px;display:block}
.desc-block{color:var(--t2);font-size:13px;line-height:1.7;white-space:pre-wrap;background:var(--bg0);padding:14px;border-radius:var(--r-md);border:1px solid var(--bd);max-height:280px;overflow-y:auto}
.res-input{width:100%;background:var(--bg2);border:1px solid var(--bd);color:var(--t1);padding:11px;border-radius:var(--r-md);font-family:var(--ff);font-size:13px;line-height:1.6;resize:vertical;outline:none}
.res-input:focus{border-color:var(--c-blue)}
.sel{width:100%;background:var(--bg2);border:1px solid var(--bd);color:var(--t1);padding:9px 11px;border-radius:var(--r-md);font-family:var(--ff);font-size:13px;cursor:pointer;outline:none}
.sel:focus{border-color:var(--c-blue)}
.chips{display:flex;flex-wrap:wrap;gap:5px}
.chip-file{display:inline-block;padding:2px 7px;background:#1a1f27;border-radius:var(--r-sm);font-family:var(--fm);font-size:11px;color:var(--c-blue);border:1px solid var(--bd)}
.chip-dep{display:inline-block;padding:2px 7px;background:rgba(163,113,247,.08);border-radius:var(--r-sm);font-family:var(--fm);font-size:11px;color:var(--c-purple);border:1px solid rgba(163,113,247,.2)}
"#;

pub const STYLES_DRAG: &str = r#"
.item{position:relative;touch-action:none;will-change:transform;z-index:1}
.item.dragging{z-index:500}
.item.settling{z-index:400}
.item.dragging .card{cursor:grabbing;border-color:var(--c-blue);background:#1a1c1e;box-shadow:0 20px 50px rgba(0,0,0,0.5)}
"#;

pub const STYLES_TOAST: &str = r#"
.toast-container{position:fixed;top:20px;right:20px;z-index:9999;display:flex;flex-direction:column;gap:8px;pointer-events:none}
.toast{padding:12px 20px;border-radius:var(--r-md);font-size:13px;font-weight:500;color:#fff;pointer-events:auto;cursor:pointer;animation:toast-in .2s var(--ease);box-shadow:0 4px 20px rgba(0,0,0,.3)}
.toast-success{background:var(--c-green)}
.toast-error{background:var(--c-red)}
.toast-info{background:var(--c-blue)}
@keyframes toast-in{from{opacity:0;transform:translateX(20px)}to{opacity:1;transform:translateX(0)}}
"#;

pub const STYLES_MODAL: &str = r#"
.modal-overlay{position:fixed;inset:0;background:rgba(0,0,0,.7);display:flex;align-items:center;justify-content:center;z-index:1000;animation:fade-in .15s var(--ease)}
.modal{background:var(--bg1);border:1px solid var(--bd);border-radius:var(--r-lg);width:100%;max-width:480px;animation:modal-in .2s var(--ease)}
.modal-header{display:flex;align-items:center;justify-content:space-between;padding:20px 24px;border-bottom:1px solid var(--bd)}
.modal-header h2{font-size:18px;font-weight:700;color:var(--t1)}
.modal-close{background:none;border:none;color:var(--t3);font-size:24px;cursor:pointer;padding:0;line-height:1}
.modal-close:hover{color:var(--t1)}
.modal-body{padding:24px}
.modal-footer{display:flex;justify-content:flex-end;gap:12px;padding:16px 24px;border-top:1px solid var(--bd)}
.modal-input{width:100%;background:var(--bg2);border:1px solid var(--bd);color:var(--t1);padding:12px 14px;border-radius:var(--r-md);font-family:var(--ff);font-size:14px;outline:none}
.modal-input:focus{border-color:var(--c-blue)}
.modal-input::placeholder{color:var(--t3)}
.btn-primary{padding:10px 20px;border-radius:var(--r-md);border:none;background:var(--c-purple);color:#fff;font-weight:600;font-size:13px;font-family:var(--ff);cursor:pointer}
.btn-primary:hover{opacity:.85}
.btn-primary:disabled{opacity:.4;cursor:not-allowed}
.btn-secondary{padding:10px 20px;border-radius:var(--r-md);border:1px solid var(--bd);background:transparent;color:var(--t2);font-weight:500;font-size:13px;font-family:var(--ff);cursor:pointer}
.btn-secondary:hover{background:var(--bg2);color:var(--t1)}
.btn-lg{padding:14px 28px;font-size:15px}
@keyframes fade-in{from{opacity:0}to{opacity:1}}
@keyframes modal-in{from{opacity:0;transform:scale(.95)}to{opacity:1;transform:scale(1)}}
"#;

pub const STYLES_WELCOME: &str = r#"
.welcome-screen{display:flex;align-items:center;justify-content:center;height:100vh;background:var(--bg0)}
.welcome-card{text-align:center;padding:60px;max-width:500px}
.welcome-icon{color:var(--c-purple);margin-bottom:24px}
.welcome-card h1{font-size:32px;font-weight:800;margin-bottom:16px;color:var(--t1)}
.welcome-desc{font-size:15px;color:var(--t2);margin-bottom:12px;line-height:1.6}
.welcome-path{font-family:var(--fm);font-size:12px;color:var(--t3);padding:8px 14px;background:var(--bg2);border-radius:var(--r-md);display:inline-block;margin-bottom:32px}
.welcome-error{color:var(--c-red);font-size:13px;margin-bottom:16px;padding:12px;background:rgba(248,81,73,.1);border-radius:var(--r-md);border:1px solid rgba(248,81,73,.2)}
.welcome-hint{font-size:12px;color:var(--t3);margin-top:16px}
"#;
