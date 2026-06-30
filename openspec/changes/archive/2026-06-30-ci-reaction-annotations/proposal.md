## Why

潛移 把法/反應放到 agent 能讀、能仿的位置。**垂象**(反應能見度)是互補的另一面:讓反應**落在 PR / CI 裡 agent 與人都看得到的地方**。目前 `check` 的機器輸出只有 `--format json`(需要工具自己解析)。**SARIF**(OASIS 開放標準)是 GitHub code-scanning 與其他工具/編輯器都會內聯消費的格式 —— 一個 vendor-neutral 的反應投影。

把同一個 `Violation` measure **多投影成 SARIF** —— 不改 law、不改 reaction 語意、不改 exit code、不改既有 text/json。這是 PROJECT.md 裡「三司 wrap the reaction;同一 measure、不同投影」的直接延伸。

**刻意不做 `--format github`(廠商耦合)**:GitHub 的 `::error::` workflow 命令是**單一 CI 廠商的專有協定**,把它做成內建格式會把工具耦合到 GitHub(且開出 gitlab/azure/… 的無底維護面)。SARIF 已 vendor-neutral 且 GitHub 內聯它;要 `::error::` 行內註解,是**harness/CI-step 的 convention**(上傳 SARIF,或用 `jq` 把 JSON report 轉成 `::error::`),不是 tool feature —— 與本專案把 校讎 / branching 視為 convention 同一原則。README 補一段 recipe。

本 change 屬 **0.1.1 patch**(SemVer 分類;加性、非破壞;bump 在 `release: 0.1.1` commit)。

## What Changes

- **`check` 新增一個輸出格式 `--format sarif`**(`crates/tianheng/src/runner.rs`):輸出 SARIF 2.1.0 JSON(`runs[].results[]`,每條非 baselined violation 一筆:`ruleId`=rule、`level`=error(Enforce)/warning(Warn)、`message.text`=reason + finding;rule 不入 message,SARIF 以 `ruleId` 結構化承載)。
- **它是 *reaction* 投影,check-only**:對稱於 markdown 是 list-only —— `list --format sarif` 是 usage error(exit 2)。
- **行號誠實**:violation 帶 `file`(部分維度)但**不帶 line**。SARIF location 只給 `physicalLocation.artifactLocation.uri = file`、**不輸出 `region`**(行號未觀測,不偽造);**無 `file` 的 violation** 的 result 不帶 `locations`(SARIF 允許)。此「file-level、無行號」為 stated bound。
- **內部重構**:check 路徑的 `json: bool` 改為 `ReportFormat { Text, Json, Sarif }`,穿進 `gate()`(兩個輸出點)與最終輸出區塊;每個輸出點 `match` 該格式。`report_json`/`report_violations`/text/coverage 行為不變。
- **outcome 對應**:clean → 空 `results` 的合法 SARIF;violations → 上述;constitution-error → SARIF `runs[0].invocations[0]`(`executionSuccessful=false` + `toolExecutionNotifications` level error);**exit code 一律不變**(0/1/2)。baselined violation 不入 SARIF(與文字報告一致:它們不 fail)。
- **README harness recipe**:補一小段「要 GitHub PR 行內註解,就上傳 SARIF(code-scanning),或在 CI 用 `jq` 把 `--format json` 轉成 `::error::` 行」—— convention,非 tool feature。

**明確排除**:`--format github`(廠商耦合,見上);editor/LSP shift-left(大整合,born-when-built → 0.2.0);改動 JSON/text/markdown 既有輸出;行/列精度(需新觀測,未建);把 sarif 變成會影響 exit code 的東西。

無 **BREAKING** 變更(純加性輸出格式;既有 text/json/markdown、exit code、reaction 不變)。

## Capabilities

### New Capabilities
<!-- 無:既有 check 報告投影面的加性擴充。 -->

### Modified Capabilities
- `cli-check-runner`: **MODIFIED**「Machine-readable report format」—— `check` 額外接受 `--format sarif`,為 `Violation` measure 的加性、vendor-neutral CI-consumable 投影(SARIF 2.1.0);明定它是 check-only 反應投影(`list` 拒收,exit 2)、location 為 file-level 無行號(stated bound)、constitution-error 標 `executionSuccessful=false`、不改 outcome/exit-code、baselined 不輸出。既有 text/json 契約不變,未知 format 仍 exit 2。

## Impact

- **`crates/tianheng/src/runner.rs`**:format 解析增 `sarif`;check 的 `json: bool` → `ReportFormat` enum,穿進 `gate()` 兩個輸出點與最終輸出;新增 `report_sarif` 純函式;`list` 拒收 sarif;usage 字串更新。`report_json`/`report_violations`/exit code/baseline/coverage 不變。一段註解記錄「github 刻意不做」的理由。
- **`README.md`**:補 SARIF/CI harness recipe(含「要 `::error::` 自己在 CI 用 `jq` 轉」)。
- **相依 / 版本**:無新外部相依(SARIF 用既有 `serde_json` 組 `Value`);non-breaking;**0.1.1 patch**(bump 在 release commit)。既有 text/json/markdown 消費者不受影響。
