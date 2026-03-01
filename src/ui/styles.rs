pub const STYLES: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=DM+Sans:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap');
:root {
    --bg0:#07080a; --bg1:#0f1114; --bg2:#181a1e; --bg3:#1e2024;
    --bd:#25272b; --bd2:#3a3d42;
    --t1:#ecedef; --t2:#95989e; --t3:#5c6066;
    --c-blue:#58a6ff; --c-green:#3fb950; --c-amber:#d29922;
    --c-purple:#a371f7; --c-red:#f85149; --c-muted:#636970;
    --r-lg:12px; --r-md:8px; --r-sm:6px;
    --ff:'DM Sans',-apple-system,sans-serif;
    --fm:'JetBrains Mono','SF Mono',monospace;
    --ease:cubic-bezier(.4,0,.2,1);
}
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:var(--ff);background:var(--bg0);color:var(--t1);overflow:hidden;height:100vh}

.shell{display:flex;height:100vh;overflow:hidden}

/* Sidebar */
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
.sidebar-foot{margin-top:14px}
.sync{text-align:center;padding:11px;border-radius:var(--r-md);background:rgba(255,255,255,.02);border:1px solid var(--bd);font-size:12px;color:var(--t3)}
.sync.dirty{border-color:var(--c-amber);color:var(--c-amber)}
.save-btn{display:block;width:100%;margin-top:7px;padding:9px;border-radius:var(--r-md);border:none;background:var(--c-green);color:#000;font-weight:700;font-size:13px;font-family:var(--ff);cursor:pointer}
.save-btn:hover{opacity:.85}

/* Main */
.main{flex:1;display:flex;flex-direction:column;overflow:hidden}
.topbar{height:66px;padding:0 36px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--bd);background:rgba(7,8,10,.85);backdrop-filter:blur(20px);flex-shrink:0}
.search-box{width:440px}
.search{width:100%;background:var(--bg2);border:1px solid var(--bd);padding:10px 18px;border-radius:36px;color:var(--t1);font-size:14px;font-family:var(--ff);outline:none;transition:border-color .2s var(--ease)}
.search:focus{border-color:var(--c-blue);background:var(--bg1)}
.search::placeholder{color:var(--t3)}
.count-pill{font-size:12px;font-weight:600;color:var(--t3);padding:5px 13px;background:var(--bg2);border-radius:18px;border:1px solid var(--bd)}
.content{flex:1;overflow-y:auto;scroll-behavior:smooth}

/* Feed */
.feed{padding:36px}
.feed-inner{max-width:840px;margin:0 auto}
.sec-hdr{margin:44px 0 14px;display:flex;align-items:center;gap:14px;font-size:11px;font-weight:800;color:var(--c-purple);text-transform:uppercase;letter-spacing:2px}
.sec-line{flex:1;height:1px;background:var(--bd)}
.sec-ct{font-family:var(--fm);font-size:11px;color:var(--t3)}

/* Card */
.card{margin-bottom:9px;border-radius:var(--r-lg);background:var(--bg1);border:1px solid var(--bd);overflow:hidden;transition:border-color .2s var(--ease)}
.card:hover{border-color:var(--bd2)}
.card.active{border-color:var(--c-blue)}
.card-hdr{padding:16px 22px;display:flex;align-items:center;gap:18px;cursor:pointer;user-select:none}
.card-hdr:hover{background:var(--bg3)}
.cid{font-family:var(--fm);font-size:12px;color:var(--t3);min-width:40px}
.ctitle{flex:1;font-weight:600;font-size:14.5px;color:var(--t1)}

/* Badges */
.badge{font-size:9px;font-weight:800;padding:3px 9px;border-radius:18px;text-transform:uppercase;letter-spacing:.7px;white-space:nowrap}
.b-open{background:rgba(88,166,255,.1);color:var(--c-blue);border:1px solid rgba(88,166,255,.2)}
.b-in-progress{background:rgba(210,153,34,.1);color:var(--c-amber);border:1px solid rgba(210,153,34,.2)}
.b-done{background:rgba(63,185,80,.1);color:var(--c-green);border:1px solid rgba(63,185,80,.2)}
.b-descoped{background:rgba(99,105,112,.1);color:var(--c-muted);border:1px solid rgba(99,105,112,.2)}

/* Detail */
.card-body{padding:0 22px 22px;border-top:1px solid var(--bd)}
.detail-grid{display:grid;grid-template-columns:1fr 240px;gap:28px;margin-top:18px}
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

/* Board */
.board{display:flex;gap:14px;padding:28px;height:100%;overflow-x:auto}
.bcol{flex:1;min-width:240px;max-width:340px;display:flex;flex-direction:column}
.bcol-hdr{display:flex;align-items:center;gap:9px;padding:10px 14px;font-size:13px;font-weight:700;margin-bottom:10px}
.bcol-dot{width:8px;height:8px;border-radius:50%}
.bcol-ct{font-family:var(--fm);font-size:11px;color:var(--t3);margin-left:auto}
.bcol-cards{flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:7px}
.bcard{padding:12px 14px;background:var(--bg1);border:1px solid var(--bd);border-radius:var(--r-md);transition:border-color .15s}
.bcard:hover{border-color:var(--bd2)}
.bcard-id{font-family:var(--fm);font-size:11px;color:var(--t3);margin-bottom:5px}
.bcard-title{font-size:13.5px;font-weight:600;line-height:1.4}
.bcard-meta{margin-top:7px;font-size:11px;color:var(--t3)}

/* Viz common */
.viz{padding:36px;max-width:920px;margin:0 auto}
.viz-hdr{margin-bottom:28px}
.viz-title{font-size:20px;font-weight:800;letter-spacing:-.3px;margin-bottom:5px}
.viz-sub{font-size:13px;color:var(--t2)}

/* Heatmap */
.hm-grid{display:flex;flex-direction:column;gap:5px}
.hm-row{display:grid;grid-template-columns:220px 1fr 36px auto;gap:10px;align-items:center;padding:7px 11px;border-radius:var(--r-md);transition:background .15s}
.hm-row:hover{background:var(--bg2)}
.hm-file{font-family:var(--fm);font-size:12px;color:var(--c-blue);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.hm-track{height:5px;background:var(--bg2);border-radius:3px;overflow:hidden}
.hm-bar{height:100%;background:linear-gradient(90deg,var(--c-amber),var(--c-red));border-radius:3px;transition:width .3s var(--ease)}
.hm-ct{font-family:var(--fm);font-size:12px;font-weight:600;color:var(--t2);text-align:right}
.hm-ids{display:flex;gap:3px;flex-wrap:wrap}
.hm-chip{font-family:var(--fm);font-size:10px;padding:1px 5px;background:var(--bg2);border-radius:4px;color:var(--t3);border:1px solid var(--bd)}

/* Graph */
.graph-sec{margin-bottom:28px}
.graph-sec-title{font-size:11px;font-weight:700;color:var(--t2);text-transform:uppercase;letter-spacing:1px;margin-bottom:10px}
.g-edge{display:flex;align-items:center;gap:9px;padding:9px 12px;border-radius:var(--r-md);margin-bottom:3px;transition:background .15s}
.g-edge:hover{background:var(--bg2)}
.g-node{font-family:var(--fm);font-size:12px;font-weight:600;padding:2px 7px;border-radius:var(--r-sm);background:var(--bg2);border:1px solid var(--bd);color:var(--c-blue)}
.g-from{color:var(--c-green)}
.g-to{color:var(--c-amber)}
.g-arrow{color:var(--t3);font-size:15px}
.g-link{color:var(--c-purple);font-size:15px}
.g-lbl{font-size:12px;color:var(--t3);margin-left:7px}
.g-files{font-family:var(--fm);font-size:11px;color:var(--t3);margin-left:auto}
.empty{text-align:center;padding:50px 18px;color:var(--t3);font-size:14px}

/* Progress / Timeline */
.pbar-wrap{margin-bottom:36px}
.pbar{display:flex;height:26px;border-radius:13px;overflow:hidden;background:var(--bg2);border:1px solid var(--bd)}
.pseg{display:flex;align-items:center;justify-content:center;font-size:10px;font-weight:700;color:rgba(0,0,0,.7);transition:width .4s var(--ease)}
.pseg.done{background:var(--c-green)}
.pseg.wip{background:var(--c-amber)}
.pseg.open{background:rgba(88,166,255,.2);color:var(--c-blue)}
.pbar-lbl{margin-top:9px;font-size:14px;font-weight:700;color:var(--t2)}
.tl-list{display:flex;flex-direction:column;gap:2px}
.tl-item{display:flex;align-items:center;gap:14px;padding:10px 14px;border-radius:var(--r-md);transition:background .15s}
.tl-item:hover{background:var(--bg2)}
.tl-dot{width:7px;height:7px;border-radius:50%;flex-shrink:0}
.st-open .tl-dot{background:var(--c-blue)}
.st-in-progress .tl-dot{background:var(--c-amber)}
.st-done .tl-dot{background:var(--c-green)}
.st-descoped .tl-dot{background:var(--c-muted)}
.tl-body{display:flex;align-items:center;gap:10px;flex:1}
.tl-id{font-family:var(--fm);font-size:11px;color:var(--t3);min-width:32px}
.tl-title{font-size:13.5px;font-weight:500;flex:1}

/* Scrollbar */
::-webkit-scrollbar{width:5px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--bd);border-radius:3px}
::-webkit-scrollbar-thumb:hover{background:var(--bd2)}
"#;
