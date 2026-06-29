## Why

當天衡的反應擋下一個 agent 的改動,agent 需要知道**在哪改**。`reason` 已給出修復方向,但 violation 沒有指向**來源檔**。

對抗性 review 釐清了一個關鍵事實:`file` **不是**「到處都該有、只是有些在手」。逐維度看,「該去哪改」的定位資訊多半已在 violation 裡,或根本不存在 file 這個概念:

- **module-import 違規(圭表)**:target 是模組路徑,但一個模組可能由多檔組成——**這裡才真正缺 file**,且 file 在違規偵測的當下就在手(scanner 為判 import 本就讀該檔),是真觀測副產品。
- **un-auditable-probe 違規(漏刻)**:CI 探針掃描對「非字面量 seam 的 `assert_boundary!`」反應時,**手上就有那個源碼檔**(`louke` 已從掃描捕獲、目前塞在 finding 字串裡),同屬真觀測副產品——所以它也該帶 file。
- **crate-dependency 違規(圭表)**:是依賴圖的邊,改的是 `Cargo.toml`,定位早在 `(target, finding)`,**無單一來源檔**。
- **semantic 違規(渾儀)**:finding 是型別路徑,**本身即 anchor**;且 finding 經 re-export 正規化,忠實對應到「定義檔」需要新追蹤,不是停止丟棄。
- **其餘 runtime 違規(漏刻)**:duplicate / undeclared / unprobed seam 命名的是 **seam 名**,不是源碼位置——**無單一檔**。

所以按 drift law「無反應不命名」、且按「reaction 本身要誠實」:`file` 在**源碼檔是真實觀測副產品**之處帶值——即 **module-import 違規** 與 **un-auditable-probe 違規**(兩者皆「停止丟棄已觀測的檔」)。其餘(crate、semantic、seam-level runtime)誠實留 `null` 並列為 stated bound:semantic 的 per-element 來源檔追蹤未建(born-when-built),crate/seam-level runtime 則本就無單一檔。

本 change 屬 **0.1.1 patch**(加性、不破壞公開 API、不碰 baseline 身份)。

## What Changes

- **`Violation` 模型加可選 `file`**(`crates/xuanji`):新增 `file: Option<String>` 欄位,以 builder/setter 設定,**不改 `Violation::new(...)` 簽章**(維持對 14 處呼叫端與 adopter 的非破壞);`file` **不進 `ViolationId`**(身份仍是 `(target, rule, finding)`),故既有 baseline 不失效。
- **`Violation::to_json` 投影 `file`**:一律輸出 `file` 鍵(有值為字串,無值為顯式 `null`),使「忠實的缺省」與「未知」可區分。此方法為所有維度共用,故所有維度的 violation JSON 都會新增此鍵(下述)。
- **圭表 module-import 違規帶 file**(`crates/guibiao`):在違規 import 被偵測到的精確檔擷取並附掛;**收斂順序為「dedup *前*收集 (身份鍵, file) 對 → 按身份鍵收斂 → 每鍵取確定性的一個 file(sort 後第一個)」**——身份鍵(importer 模組 / finding)不變,故 violation 基數不變(絕不可用 `(module, file)` tuple 當 dedup 鍵,那會同身份多報)。
- **漏刻 un-auditable-probe 違規帶 file**(`crates/louke`):該違規建構當下 `file` 已在迴圈變數中(現塞於 finding),以 builder 一併投影到 `file` 欄位;其餘 3 種 runtime 違規(duplicate / undeclared / unprobed seam)維持 `None`。
- **其餘維度誠實缺省**:crate-dependency、semantic(渾儀四 capability)、seam-level runtime 的 violation `file` 為 `null`,程式註解與 spec 明列理由(crate/seam-level=無單一檔;semantic=per-element 追蹤未建的 stated bound)。
- **xuanji `to_json` 回歸測試**:目前 `to_json` 形狀未被測;補測試斷言 `file` 鍵的有值/null 行為。

**明確排除**:semantic 的 per-finding 來源檔追蹤(需新 plumbing:`ImplSite` 加欄位、findings 關聯回定義檔)——stated bound,born-when-built;行/列(span)精度;repair-category;drift 分類。

無 **BREAKING** 變更。

## Capabilities

### New Capabilities
<!-- 無新 capability:既有 report 投影面的加性擴充。 -->

### Modified Capabilities
- `cli-check-runner`: 「Machine-readable report format」需求——每條 violation 的 JSON 投影增加一個可選 `file`;**module-import 違規與 un-auditable-probe 違規帶值,其餘(crate、semantic、seam-level runtime)為 `null`**(各帶理由);不改 outcome/exit-code 語意,不進 baseline 身份。

<!-- violation-baseline 刻意不在此:身份 (target, rule, finding) 不變,file 被排除於身份外,既有 baseline 不失效。 -->

## Impact

- **`xuanji`(璇璣)**:`Violation` 增 `file: Option<String>` + builder;`to_json` 投影該鍵;`Violation::new`、`ViolationId`、`id()`、baseline 邏輯不變。所有維度的 violation JSON 連動新增 `file` 鍵。
- **`guibiao`(圭表)**:module-import(inbound + outbound)違規建構帶上來源檔(dedup 前收集 (鍵, file)、收斂後取代表 file);crate 維持 `None`。dedup 身份鍵不變。
- **`louke`(漏刻)**:un-auditable-probe 違規帶上其源碼檔;其餘 3 種 seam-level runtime 違規維持 `None`。louke default-sink JSON 因共用 `to_json` 會多出 `file` 鍵(seam-level 為 `null`)——行為加性、非破壞,明確承認並測試。
- **`hunyi`(渾儀)**:四個 semantic 違規維持 `file = None`(stated bound,per-element 來源檔追蹤未建)。
- **相依 / 版本**:無新外部相依;non-breaking;**0.1.1 patch**。既有 JSON 消費者向後相容(僅新增可選鍵);既有 baseline 不失效。
