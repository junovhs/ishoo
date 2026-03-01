pub const STYLES_VIZ: &str = r#"
::-webkit-scrollbar{width:5px}::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--bd);border-radius:3px}::-webkit-scrollbar-thumb:hover{background:var(--bd2)}
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
.viz{padding:36px;max-width:920px;margin:0 auto}
.viz-hdr{margin-bottom:28px}
.viz-title{font-size:20px;font-weight:800;letter-spacing:-.3px;margin-bottom:5px}
.viz-sub{font-size:13px;color:var(--t2)}
.hm-grid{display:flex;flex-direction:column;gap:5px}
.hm-row{display:grid;grid-template-columns:220px 1fr 36px auto;gap:10px;align-items:center;padding:7px 11px;border-radius:var(--r-md);transition:background .15s}
.hm-row:hover{background:var(--bg2)}
.hm-file{font-family:var(--fm);font-size:12px;color:var(--c-blue);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.hm-track{height:5px;background:var(--bg2);border-radius:3px;overflow:hidden}
.hm-bar{height:100%;background:linear-gradient(90deg,var(--c-amber),var(--c-red));border-radius:3px;transition:width .3s var(--ease)}
.hm-ct{font-family:var(--fm);font-size:12px;font-weight:600;color:var(--t2);text-align:right}
.hm-ids{display:flex;gap:3px;flex-wrap:wrap}
.hm-chip{font-family:var(--fm);font-size:10px;padding:1px 5px;background:var(--bg2);border-radius:4px;color:var(--t3);border:1px solid var(--bd)}
.graph-sec{margin-bottom:28px}
.graph-sec-title{font-size:11px;font-weight:700;color:var(--t2);text-transform:uppercase;letter-spacing:1px;margin-bottom:10px}
.g-edge{display:flex;align-items:center;gap:9px;padding:9px 12px;border-radius:var(--r-md);margin-bottom:3px;transition:background .15s}
.g-edge:hover{background:var(--bg2)}
.g-node{font-family:var(--fm);font-size:12px;font-weight:600;padding:2px 7px;border-radius:var(--r-sm);background:var(--bg2);border:1px solid var(--bd);color:var(--c-blue)}
.g-from{color:var(--c-green)}.g-to{color:var(--c-amber)}
.g-arrow{color:var(--t3);font-size:15px}.g-link{color:var(--c-purple);font-size:15px}
.g-lbl{font-size:12px;color:var(--t3);margin-left:7px}
.g-files{font-family:var(--fm);font-size:11px;color:var(--t3);margin-left:auto}
.empty{text-align:center;padding:50px 18px;color:var(--t3);font-size:14px}
.pbar-wrap{margin-bottom:36px}
.pbar{display:flex;height:26px;border-radius:13px;overflow:hidden;background:var(--bg2);border:1px solid var(--bd)}
.pseg{display:flex;align-items:center;justify-content:center;font-size:10px;font-weight:700;color:rgba(0,0,0,.7);transition:width .4s var(--ease)}
.pseg.done{background:var(--c-green)}.pseg.wip{background:var(--c-amber)}
.pseg.open{background:rgba(88,166,255,.2);color:var(--c-blue)}
.pbar-lbl{margin-top:9px;font-size:14px;font-weight:700;color:var(--t2)}
.tl-list{display:flex;flex-direction:column;gap:2px}
.tl-item{display:flex;align-items:center;gap:14px;padding:10px 14px;border-radius:var(--r-md);transition:background .15s}
.tl-item:hover{background:var(--bg2)}
.tl-dot{width:7px;height:7px;border-radius:50%;flex-shrink:0}
.st-open .tl-dot{background:var(--c-blue)}.st-in-progress .tl-dot{background:var(--c-amber)}
.st-done .tl-dot{background:var(--c-green)}.st-descoped .tl-dot{background:var(--c-muted)}
.tl-body{display:flex;align-items:center;gap:10px;flex:1}
.tl-id{font-family:var(--fm);font-size:11px;color:var(--t3);min-width:32px}
.tl-title{font-size:13.5px;font-weight:500;flex:1}
"#;
