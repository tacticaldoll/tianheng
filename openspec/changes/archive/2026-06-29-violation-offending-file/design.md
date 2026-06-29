## Context

violation 缺 offending file,agent 知被擋、不知在哪改。對抗性 review 釐清原則:`file` 在**源碼檔是真實觀測副產品**之處帶值;其餘維度要嘛已有定位、要嘛無單一檔、要嘛需新追蹤。按此原則(而非「只限 module-import」),帶值者為 **guibiao module-import** 與 **louke un-auditable-probe**(兩者皆已在手),其餘誠實 `null`。

> 一個 review 修正的前提:原稿宣稱「漏刻無源碼 file、一律 null」是**假的**——un-auditable-probe 違規(louke `:574-586`)建構當下迴圈變數 `file` 就是真實源碼檔,目前被塞進 finding 字串。對它輸出 `null` 是「觀測到了卻謊報 null」,違反「reaction 要誠實」。故按原則它該帶 file;此舉與 module-import 同性質(停止丟棄已觀測的檔),不是新 plumbing。

已查證事實:
- `Violation`(`crates/xuanji/src/lib.rs:98`)`#[non_exhaustive]`,經 `Violation::new`(`:120`)建構(14 處呼叫,無 struct-literal);`ViolationId`(`:192`)= 固定 `(target, rule, finding)`,`id()`(`:140`)、baseline 邏輯只讀此三者;`to_json`(`:151`)顯式列鍵;倉庫**無 on-disk baseline fixture**,亦**無 snapshot/exact-eq 測試**。
- 圭表 module-import:inbound 迴圈 `for (file, current_module) in all_files`(`crates/guibiao/src/lib.rs:310`)違規當下 `file` 在手,目前只 push `current_module`(`:330`)後 `sort+dedup`(`:337`);outbound 同樣持有 file(`:394`)。crate 違規(`:221`)由 `findings()` 產出,無 file。
- 漏刻 `audit_probe_coverage`:un-auditable-probe 迴圈 `for file in unauditable_files`(`crates/louke/src/lib.rs:574`)`file` 在手(現格式化進 finding,`:580`);其餘 3 種 runtime 違規(duplicate `:517` / unprobed `:535` / undeclared `:551`)命名 seam,無檔。
- 渾儀 findings 是 re-export 正規化後的型別路徑字串,建構 scope 只有 `root_file`(crate 根),非 offending 元素檔;`ImplSite`(`:848`)不存 file → `null`(stated bound,per-element 追蹤未建)。
- `to_json` 為所有維度共用,改它會連動 louke 的 default-sink 輸出。

## Goals / Non-Goals

**Goals:**
- `Violation` 帶可選 `file`,投影於 report JSON;**module-import 與 un-auditable-probe 帶值**,crate/semantic/seam-level runtime `null`。
- 不碰 `Violation::new` 簽章、不碰 `ViolationId`/baseline 身份。
- dedup 基數不變(file 收斂後附掛)。
- non-breaking;0.1.1 patch。

**Non-Goals:**
- semantic 的 per-element 來源檔追蹤(`ImplSite` 加欄位、findings 關聯定義檔)——stated bound,born-when-built。
- 行/列(span)精度。
- repair-category、drift 分類。

## Decisions

### D1 — `file: Option<String>` 經 builder 設定,不進 `new()`
`Violation` 增 `file: Option<String>`,以消費式 builder(`with_file(self, Option<String>) -> Self`)設定,**不**加入 `new(...)` 參數列。
- *為何*:`new` 是 pub 且 14 處呼叫;加參數破壞簽章。builder 配 `#[non_exhaustive]` 維持 `new()` 穩定 → 非破壞。
- *型別 String 而非 PathBuf*:`xuanji` 是 dimension-agnostic、`serde_json`-only,不帶 path 語意;以 display 字串呈現,與既有純 String 的 `target`/`finding` 一致。維度持 `PathBuf`,邊界轉字串。

### D2 — `to_json` 顯式輸出 `file` 鍵(無值為 `null`)
`to_json` 一律列 `file`;無檔為 JSON `null`。
- *為何*:spec 要區分「忠實缺省」(crate/semantic/runtime)與「未知」。顯式 `null` 對 agent 是「查過、沒有」,比缺鍵更誠實,schema 偵測穩定。
- *連動承認*:此方法所有維度共用,louke default-sink JSON 會多 `file: null`——行為改變、非破壞,於 louke 測試承認。

### D3 — module-import 違規:身份鍵不變,file 於 dedup 前收集、收斂後取代表
收斂順序**必須**是:
1. 在違規 import 被偵測到的當下,收集 `(身份鍵, file)` 對(inbound 身份鍵 = importer 模組,outbound = finding/import)——**dedup 之前**,因為 dedup 純 `Vec<String>` 後 file 已不在 scope;
2. 按**身份鍵**收斂(sort + dedup by key),violation 基數 = 不同身份鍵數,與現狀一致;
3. 每個身份鍵取**確定性的一個 file**(sort 後第一個),附掛到該 violation。
- *為何*:對抗性 review 指出若把 `offenders` 改成 `(module, file)` tuple 再 `sort+dedup`,同一 importer 由兩檔違規會產生**兩條同身份 violation**(多報、衝擊 baseline)。故 **file 絕不可進 dedup 鍵**;且不可寫成「先 dedup 再附 file」(那時 file 已失)——必須 dedup 前收集對、收斂後取代表。
- *spec 用詞*:用「a source file where the forbidden import occurs」(非「the file」),誠實面對一模組多檔。

### D4 — louke un-auditable-probe 帶 file;其餘 runtime 與 crate/semantic `null`
- **un-auditable-probe**(louke `:574-586`):迴圈變數 `file` 即真實源碼檔,以 builder 投影到 `file` 欄位(同時仍留在 finding,如現狀)。這是「停止丟棄已觀測的檔」,與 module-import 同性質。
- **其餘 3 種 runtime**(duplicate/unprobed/undeclared seam)、**crate**、**semantic 四 capability**:`file = None`,程式註解與 spec 明列理由——crate/seam-level runtime 命名 seam 或依賴邊、**無單一檔**;semantic 的 per-element 來源檔追蹤**未建**(stated bound)。
- *為何*:原則是「源碼檔是真實觀測副產品就投影」+「reaction 要誠實」。un-auditable-probe 的檔在手,輸出 `null` 會是「觀測到卻謊報」;其餘是真實的「無檔」或「未觀測」,`null` 才忠實。drift law「無反應不命名」對應到「未觀測處不假裝有 file」。

### D5 — baseline 身份由結構保證不變 + 回歸測試
`ViolationId` 維持三元組;`file` 在三元組外,無需改身份邏輯即隔離。補回歸測試:帶 file 的 module violation 仍匹配其 baseline entry、仍被 dedup 為一條。

## Risks / Trade-offs

- **`(module, file)` tuple 破壞 dedup → 多報、衝擊 baseline** → D3:dedup 鍵不變,file 附掛於代表者。
- **未來誤把 `file` 併入 `ViolationId`** → D5 回歸測試 + 維持顯式三元組。
- **`to_json` 連動 louke** → D2 於 louke 測試承認其 default-sink 多 `file: null`。
- **un-auditable-probe 的檔被謊報 null(觀測到卻 null)** → D4:它帶 file。其餘 runtime/semantic/crate 的 `null` 是真實「無檔」或「未觀測」,spec + 註解各列理由,不混為一談。
- **路徑形態(絕對 vs 相對)** → 見 Open Questions;因不入身份,不影響正確性。

## Migration Plan

純加性、無資料遷移:JSON 消費者多看到一個可選 `file` 鍵;既有 baseline 不失效;預設 text 與既有 json 其餘欄位不變。Rollback = revert。

## Open Questions

- **`file` 路徑形態**:傾向以引擎觀測形態投影(通常為絕對 src 路徑);是否正規化為 workspace-相對以利跨機器 diff,列為可選後續——因不入身份,本 change 不強制。
