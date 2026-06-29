## Context

要讓天衡的反應在 AI agent 迴圈裡閉環,需要 agent 能在改碼前讀到法、改碼後撞到反應。`reason`(修復方向)已存在於 `check`/`list` 的 JSON。本 change 補兩個**乾淨、零模型風險**的缺口:`list` 的 agent 可讀 markdown 投影,與一份 agent SOP。

(offending file 經對抗性 review 證實是「新增追蹤」而非「停止丟棄」,已拆為獨立 change `violation-offending-file`,不在此。)

現況關鍵事實(已查證 `crates/tianheng/src/runner.rs`):
- `--format` 在 `:126-135` 對**兩個指令共用**驗證,目前只認 text/json,其餘 exit 2。
- `list`(`:141-166`)是純投影:text 路徑呼叫六個 renderer(`constitution_text` + `semantic_text` + `trait_impl_text` + `visibility_text` + `forbidden_marker_text` + `runtime_text`,`:158-163`);json 路徑呼叫 `list_document`(`:645-692`),以靜態 `constitution_json` 為底,**逐個非空維度**補一個陣列。dimension 為空則不輸出該段。
- usage 字串(`:263-269`)硬寫兩條:`check ... [--format text|json]` 與 `list [--format text|json]`。

## Goals / Non-Goals

**Goals:**
- `list --format markdown` 投影**所有維度**的宣告法(靜態 + 四語意 + runtime),不少於 JSON。
- markdown 為 `list`-only;`check --format markdown` exit 2,明確 spec。
- AGENTS.md 補 agent SOP。
- 只動殼 + 文件;non-breaking;0.1.1 patch。

**Non-Goals:**
- 不碰 `xuanji`/`guibiao`/`hunyi`/`louke`(無模型、無維度引擎改動)。
- 不做 offending file(獨立 change)、repair-category、drift 分類、OpenSpec 工具化。
- markdown 不投影「workspace 實況」——它只投影**宣告的憲法**,與 `list` 整體一致(不觀測、不反應)。

## Decisions

### D1 — markdown 沿用 `list_document`/`*_text` 的多維度遍歷,新增 `list_markdown(...)`
在殼裡新增 `list_markdown(constitution) -> String`,**走與 `list_document`/text 完全相同的維度集合**:靜態憲法 + 四個非空語意 capability + runtime,逐維度渲染 markdown 段落;維度為空則不輸出該段。
- *為何*:對抗性 review 證實「只投影靜態」會少於 JSON、違反「不少於 JSON」的 spec,且讓 agent 讀到的法漏掉語意/runtime 維度。markdown 必須對齊真實的多維度投影,而非 `guibiao::constitution_json`(那只走靜態)。
- *替代*:在 xuanji 型別上加 markdown 方法——否決,文件組裝屬殼/引擎,不入 dimension-agnostic 模型(PROJECT.md)。
- *替代*:只投影靜態憲法摘要——否決,違反「不少於 JSON」且對 agent 不誠實(漏維度)。

### D2 — `--format` 由二向擴成三向,markdown 為 `list`-only
把共用的 `--format` 解析改為認得 `text|json|markdown` 三值;在 `check` 路徑把 `markdown` 當 usage error(exit 2),`list` 路徑三值皆受理。
- *為何*:markdown 一旦成為已知值,就不再落入「unknown format」;若不顯式擋,`check --format markdown` 會語意未定義。`check` 的機器輸出是 JSON report,不是法律摘要,故拒收是正確的不對稱。
- 同步更新兩條 usage 字串:`list` → `[--format text|json|markdown]`,`check` → `[--format text|json]`。

### D3 — AGENTS.md SOP 是 convention,不入憲法
SOP(改碼前 `list`(可 markdown/json)讀法 → 改碼後 `check --format json` → 失敗讀 `reason` 為修復方向 → 改法走 OpenSpec/steward)寫在 AGENTS.md。
- *為何*:不是可觀測架構事實(同 branching 規則的既有判例),drift law 擋在憲法外;明確定位為 orientation——綁定仍由反應完成,不暗示「指望 agent 自律」。

## Risks / Trade-offs

- **markdown 漏維度(少於 JSON)** → D1 強制走 `list_document` 的同一維度集合;新增 scenario 斷言涵蓋每個非空維度。
- **`check --format markdown` 語意未定義 / 靜默接受** → D2 顯式擋為 exit 2 並補 scenario 與測試。
- **usage 文案與既有 exit-2 測試脫節** → tasks 明列更新兩條 usage 字串與 `unknown_format`/`list_unknown_format` 類測試。
- **markdown 不小心觀測 workspace(變成反應)** → 沿用 `list` 既有「不取 `--manifest-path`、exit 0」的結構;新增 scenario 斷言不評估、不產 violation。

## Migration Plan

純加性、無資料遷移:
- 既有 `list` text/json 輸出與預設格式完全不變。
- 新增一個 format 值與一個 renderer;`check` 行為僅多一條明確的 exit-2 拒收。
- Rollback = revert commit,無狀態。

## Open Questions

- **markdown 版面**:每個 boundary 一段標題 vs 表格、維度間的層級——交由實作,spec 僅要求每個 boundary 的 target/rule/reason 俱在、且涵蓋所有非空維度。
