## 1. 模型:Violation 帶可選 file(xuanji)

- [x] 1.1 `Violation` 新增 `file: Option<String>` 欄位,預設 `None`;**不改** `Violation::new(...)` 簽章。
- [x] 1.2 新增消費式 builder `with_file(self, Option<String>) -> Self`。
- [x] 1.3 `Violation::to_json` 一律輸出 `file` 鍵:有值為字串,無值為顯式 `null`(D2)。
- [x] 1.4 確認 `ViolationId`、`id()`、`apply_baseline`、baseline (de)serialize 不讀 `file`。
- [x] 1.5 新增 xuanji 單元測試:`to_json` 帶 `file` 鍵(有值/null 兩態);帶 `file` 的 violation 其 `id()` 與不帶時相同(身份隔離)。
- [x] 1.6 `#![deny(missing_docs)]` 下新增項目皆有文件。

## 2. 圭表:module-import 違規帶 file(guibiao)

- [x] 2.1 inbound `must_not_import`:**dedup 前**收集 `(importer 模組, file)` 對(在 `imports_protected` 為真的當下);按 importer 模組收斂;每模組取 sort 後第一個 file 附掛。**不得**把 `offenders` 改成 `(module, file)` tuple 後再 dedup(會同身份多報)。
- [x] 2.2 outbound 規則路徑:同樣 **dedup 前**收集 `(finding, file)` 對,按 finding 收斂、取代表 file 附掛。
- [x] 2.3 crate 邊界(`check_crate_boundary`)維持 `file = None`。
- [x] 2.4 程式註解標明:file 僅檔案層級(非 line/col);module-import 是真觀測副產品。
- [x] 2.5 測試:module 違規 report JSON 帶正確 `file`;crate 違規 `file` 為 `null`;**一模組兩檔皆違規仍只一條 violation**(dedup 基數不變)。

## 3. 漏刻:un-auditable-probe 帶 file,其餘 runtime null(louke)

- [x] 3.1 un-auditable-probe 違規(`crates/louke/src/lib.rs:574-586`):迴圈變數 `file` 以 `.with_file(Some(file.to_string()))` 投影(finding 維持不變);其餘 3 種 runtime 違規(duplicate `:517` / unprobed `:535` / undeclared `:551`)維持 `None`。
- [x] 3.2 測試:un-auditable-probe 違規帶其源碼檔;seam-level runtime 違規 `file` 為 `null`。**承認並斷言** `to_json` 共用導致 louke default-sink JSON 多 `file` 鍵(seam-level 為 `null`)。

## 4. 渾儀:誠實 null + stated bound(hunyi)

- [x] 4.1 hunyi 四個 semantic 建構點維持 `file = None`;程式註解標明 stated bound(per-element 來源檔追蹤未建,born-when-built)。
- [x] 4.2 semantic 違規 `file=null` 由**建構保證**(hunyi 四建構點皆 `Violation::new`、不呼叫 `with_file`,程式註解鎖意圖)+ **xuanji `to_json` null 測試**覆蓋投影。未另建 cargo-metadata workspace fixture e2e(不成比例;hunyi 既有測試亦只測純核心 `findings()`)。

## 5. 端到端與回歸

- [x] 5.1 per-kind file 投影由 **Violation 層測試**覆蓋(guibiao:module 帶 file、crate `null`、兩檔仍一條;louke:un-auditable 帶 file、seam-level `null`;xuanji:`to_json` value/null)。report 的 JSON 經 `Violation::to_json` 投影(`projection.rs`),故 JSON 連動成立。未另作捕捉 stdout 的 runner e2e(runner 測試以 exit code 為主,不捕 stdout)。
- [x] 5.2 回歸:帶 file 後既有 baseline gate 行為不變(身份匹配、dedup 為一條)。
- [x] 5.3 `cargo test --workspace --all-features` 全綠;`cargo clippy --all-features` 無新警告;本地完整 DoD(含 `cargo fmt --check` 與 `cargo doc`)。
- [x] 5.4 self_governance gate 仍通過(本 change 不動依賴邊界)。

## 6. 收尾

- [x] 6.1 確認無 public API 破壞(`new` 簽章不變、無既有 JSON 欄位語意變更)、無新外部相依,維持 0.1.1 patch。
- [ ] 6.2 `openspec validate "violation-offending-file"` 通過;PR base 指向 `release/0.1.1`,squash subject 自我描述、去除 `(#N)`。
