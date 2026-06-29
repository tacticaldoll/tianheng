## 1. 殼:--format 三向 dispatch(tianheng/src/runner.rs)

- [x] 1.1 把共用的 `--format` 解析(`:126-135`)由 text/json 擴成認得 `text|json|markdown` 三值。
- [x] 1.2 `check` 路徑:`--format markdown` 視為 usage error,exit 2(訊息說明 markdown 為 list-only、check 機器輸出是 JSON report)。
- [x] 1.3 更新兩條 usage 字串(`:263-269`):`list` → `[--format text|json|markdown]`,`check` → `[--format text|json]`。

## 2. 殼:list markdown 投影

- [x] 2.1 新增 `list_markdown(constitution) -> String`,走與 `list_document`/text **相同的維度集合**:靜態憲法 + 非空的 signature/trait_impl/visibility/forbidden_marker 語意 capability + runtime;維度為空不輸出該段。
- [x] 2.2 每個 boundary 渲染 target、rule(禁止/限制什麼)、declared reason;markdown 為純投影、exit 0、不取 `--manifest-path`、不評估 workspace。
- [x] 2.3 `list` 路徑接上 markdown 分支(text/json/markdown 三向)。

## 3. 測試(tianheng)

- [x] 3.1 `list --format markdown`:輸出含憲法名、各 boundary 的 target/rule/reason。
- [x] 3.2 多維度斷言:對同時宣告 靜態 + 語意 + runtime 的憲法,markdown 涵蓋每個非空維度(不少於 JSON)。
- [x] 3.3 純投影斷言:`list --format markdown` exit 0、不評估 workspace、不產 violation。
- [x] 3.4 `check --format markdown` → usage error exit 2。
- [x] 3.5 `list --format <未知>` → exit 2;既有 `unknown_format`/`list_unknown_format` 類測試同步更新(三值)。
- [x] 3.6 回歸:`list --format text` 與 `--format json` 輸出與先前一致(預設格式不變)。

## 4. AGENTS.md SOP(convention,非 spec/憲法)

- [x] 4.1 AGENTS.md 新增 "Agent workflow" 段:改碼前 `list`(可 `--format markdown/json`)讀法 → 改碼後 `check --format json` → 失敗讀 violation 的 `reason`(修復方向)→ 要改法走 OpenSpec propose / steward review。
- [x] 4.2 明寫此為 orientation 而非綁定機制(綁定仍由反應完成),且不進憲法、不新增 drift type。

## 5. 收尾

- [x] 5.1 `cargo test --workspace --all-features` 全綠;`cargo clippy --all-features` 無新警告。
- [x] 5.2 self_governance gate 仍通過(本 change 不動依賴邊界、不動模型)。
- [x] 5.3 確認無 public API 破壞、無新外部相依,維持 0.1.1 patch。
- [ ] 5.4 `openspec validate "agent-actionable-projection"` 通過(✅);PR base 指向 `release/0.1.1`、squash subject 自我描述去除 `(#N)` — 待 review 後 push/PR
