## Why

潛移 已把 reason **前置進「法的投影」**(`list --format markdown`)。但 agent 撞牆時讀的是**反應的輸出** —— `check` 的人類文字報告。那裡 reason 仍排在第 4 位(Boundary → Rule → Found → **Reason** → Reaction),被機械欄位埋住;而且報告**完全沒顯示 `file`**(offending-file change 把它加進了 JSON,但人類報告從不印它,等於把「該去哪改」丟掉)。

把前置原則**從法投影延伸到反應輸出**:violation 文字塊以 reason 領頭、補上 file(修復定位)、並把同一 boundary 的多條 violation 聚在一起,讓 agent 一眼看出母則。純人類面演進:**JSON 機器契約一字不動、無新欄位、無 `repair_hint`(不造第二事實源)**。

本 change 屬 **0.1.1 patch**(SemVer 分類;加性、非破壞;實際 bump 在 `release: 0.1.1` commit)。

## What Changes

- **violation 文字塊 reason 前置**(`crates/tianheng/src/runner.rs` `report_violations`):順序改為 **Reason → Boundary → Rule → Found →(File)→ Reaction** —— reason 是母則/修復方向,領在 header 之後、機械欄位之前。
- **補上 `file`**:violation 帶 `file` 時,文字報告新增 `File:\n  <path>`(「該去哪改」)。`file` 為 `None` 時不印該段(忠實缺省,不偽造)。**這是把既有觀測資料投影到人類報告,非新增資料。**
- **多 violation 依 boundary 聚合**:文字報告把 violations 依 `(target, rule)` 穩定排序,使同一 boundary 的 finding 連續呈現,母則只讀一次。**僅在文字呈現層排序**,`Report`/JSON 的順序與內容不動。
- **(E,搭車)`constitution_markdown` doctest**:把上一個 change 的 README adopter recipe 鎖成 CI 會跑的活範例(`/// ```` 例)。

**明確排除**:JSON 投影任何變更;`repair_hint` 或任何改寫/詮釋 reason 的衍生欄位;good/bad reason 庫;報告改成機器契約。

無 **BREAKING** 變更(人類文字報告為可演進面;JSON 不變)。

## Capabilities

### New Capabilities
<!-- 無 -->

### Modified Capabilities
- `cli-check-runner`: **新增** requirement「Human violation report foregrounds the reason」—— 文字報告 SHALL 以 reason 領頭、SHALL 在 `file` 存在時呈現它、SHALL 依 boundary 聚合 violations;明文聲明這是**呈現層的順序/聚合不變式**,JSON 機器契約不受影響(`file` 為 `None` 時誠實省略)。既有「Machine-readable report format」(JSON)requirement **不變**。

## Impact

- **`crates/tianheng/src/runner.rs`**:`report_violations` 重排欄位、補 `File:` 段、依 `(target, rule)` 排序後輸出。JSON 路徑(`report_json`)、exit code、reaction 語意、baseline 全不動。
- **相依 / 版本**:無新相依;non-breaking;**0.1.1 patch**(bump 在 release commit)。JSON 消費者不受影響。
