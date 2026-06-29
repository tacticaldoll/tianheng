## Why

天衡的立論是「以反應治理,而非以指令治理」。要讓反應在 **AI agent 的工作迴圈**裡閉環,agent 必須能在改碼前**讀到法**、改碼後**撞到反應**、並知道**為何被擋**。

盤點後,投遞路徑的缺口分成兩類,本 change 只處理**乾淨、零模型風險**的那一類:

- `reason`(修復方向)**已**存在於 `check --format json` 與 `list --format json`,本 change **不重做**。
- 缺一個 agent 友善的 **markdown 法律摘要**:`list` 目前只有 text/json,沒有適合餵進 agent context 的可讀投影。
- 缺一份寫死的 **agent SOP**:改碼前/後該怎麼用天衡。

(另一類——給 violation 加 **offending file**——經對抗性 review 證實是「新增追蹤」而非「停止丟棄已觀測資料」,且只有 module-import 維度真正缺它;它已**拆出**為獨立 change `violation-offending-file`,不在本 change。)

本 change **只動殼(`tianheng`)與文件**,不碰 `xuanji` 模型、不碰任何維度引擎。全部是 non-breaking 投影層,屬 **0.1.1 patch**。

## What Changes

- **`list` 增加 markdown 投影**:`list --format markdown`(與 `--format=markdown`),把**整份宣告的法**投影成 agent 可讀的「架構法律摘要 + 禁忌」。markdown 必須**涵蓋與 text/json 相同的所有維度**(靜態憲法 + 四個語意 capability + runtime),沿用既有的多維度組裝路徑——不得只投影靜態而少於 JSON。純加性、不反應、exit 0;dimension 為空則不輸出該段(與既有 text/json 行為一致)。
- **markdown 是 `list`-only**:`check --format markdown` 是 usage error,exit 2(`check` 的機器輸出是 JSON report,不是法律摘要)。此不對稱明確寫入 spec,避免 agent 困惑。
- **usage 字串更新**:`list` 用法行改為 `[--format text|json|markdown]`,`check` 維持 `[--format text|json]`。
- **AGENTS.md 補一段 agent workflow SOP**:改碼前 `list`(可 `--format markdown/json`)讀法 → 改碼後 `check --format json` → 失敗讀 violation 的 declared `reason` 當修復方向 → 要改法走 OpenSpec propose / steward review。這是 convention,**不進憲法、不新增 drift type**;且明確定位為 orientation——綁定仍由反應完成,不暗示「指望 agent 自律」。

**明確排除**(在其他 change 或被否決):offending file(→ `violation-offending-file`)、repair-category 處方、code/constitution-drift 分類、OpenSpec↔Tianheng 工具化。

無 **BREAKING** 變更。

## Capabilities

### New Capabilities
<!-- 無新 capability:全是 list 既有投影面的加性擴充。 -->

### Modified Capabilities
- `constitution-projection`: 「List honors the format flag」需求——`list` 額外支援 `--format markdown`,投影為涵蓋所有維度的 agent 可讀法律摘要;text/json 不變,未知 format 仍 exit 2。
- `cli-check-runner`: 「Machine-readable report format」需求——新增一條 scenario,明定 `check --format markdown` 是 usage error exit 2(markdown 為 list-only),使既有 format 契約在新增 markdown 值後仍無歧義。

## Impact

- **`tianheng`(天衡)殼**:`runner.rs` 的 `--format` dispatch 由 text/json 擴成三向(text/json/markdown),markdown 僅 `list` 接受、`check` 撞它 exit 2;新增 `list_markdown(...)`,沿用 `list_document`/`*_text` 的多維度遍歷;更新兩條 usage 字串與相關 exit-2 測試。
- **AGENTS.md**:新增 agent workflow SOP 段落(無 spec/憲法變更)。
- **不動**:`xuanji`、`guibiao`、`hunyi`、`louke`——本 change 不碰模型與任何維度引擎。
- **相依 / 版本**:無新外部相依;non-breaking;**0.1.1 patch**。既有 `list` text/json 輸出不變;預設格式不變。
