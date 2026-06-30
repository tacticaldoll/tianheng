## Why

`AGENTS.md` 叫 agent 改碼前讀法:`tianheng list --format markdown`。但**已發佈的 binary 投影的是 demo constitution**(`crates/tianheng/src/constitution.rs`,治理一個叫 `example-core` 的範例),而 Tianheng**真正 enforced 的 self-law** 活在 test-only 的 `tianheng_constitution()`(`crates/tianheng/tests/self_governance.rs`)。所以 Tianheng 最該被模仿的 idiom——璇璣 bedrock 在所有維度之下、heavy `syn` 被隔離在渾儀、functional core ⊥ shell、維度只由 shell 組合、漏刻以 dependency boundary 維持 production-light——**不在 agent 的 context 裡**。

這不是「缺文件」,是**入口錯位**:LLM 第一口 context 吃到的是 example,不是 self-law。本 change 是 PROJECT.md 新立的 **潛移(Qiányí)面**的第一個 dogfood:把 Tianheng 自己 enforced 的法,做成一份**忠實、agent-loaded、會被測試追責**的投影。

**定位誠實:本 change 只改善 dogfood / contributor 迴圈**(在本 repo 工作的 agent 讀到 self-law 而非 demo);它**不動 adopter 的採用率**(adopter-facing 的潛移面是後續 phase,本 change 是其原型)。

本 change 屬 **0.1.1 patch**(全加性:文件 + 生成物 + 測試 + 一個加性 `pub fn` + 一個加性 `From` impl;既有公開面一字不動)。此為 **SemVer 分類**,非「本 change 交付 bump」:workspace 版號的實際 bump 落在 `release: 0.1.1` snapshot commit(與前兩個 0.1.1 change 一致),**不在本 change 內**,故 tasks 不含 `Cargo.toml` 改動。

## What Changes

本 change 守住**兩條正交的契約**,規格層必須分開陳述:

- **A. repo artifact 契約**:`AGENTS.self-law.md` == live 生成輸出,由本地 staleness 測試 enforce,renderer 變動後以 regenerate(`BLESS`)修正。
- **B. public API 格式契約**:markdown helper 的輸出形狀**不是** machine-stable;由 `pub fn` 上的 **doc 契約**守護,**不是**由測試守護。Markdown 是人類/agent surface,JSON 才是機器契約。

具體變更(皆加性):

- **`Constitution::from(GnomonConstitution)` lift**(`crates/tianheng/src/lib.rs`):新增 `impl From<GnomonConstitution> for Constitution`(`static_` 取入參,`semantic`/`runtime` 取 default/empty)。讓 `self_governance.rs` 的狗糧證明 `check(&tianheng_constitution(), &manifest)` **一字不動**;投影端才以 `.into()` lift。
- **`pub fn constitution_markdown(&Constitution) -> String`**(`crates/tianheng/src/runner.rs`):組合既有的 **private** `list_document` + `list_markdown`(兩者維持 private 實作細節)。**必帶 doc 契約**:此 Markdown 排版為人類/agent 可讀,供 display / review / LLM context;其**排版可能在任何相容版本中演進**以改善可讀性/可仿性;需要穩定機器契約者改用 JSON 投影。(此即契約 B 的解藥——格式**不得被凍結**,因為潛移的 reason-foregrounding 演進針對的正是這個面。)
- **self-law 投影生成 + staleness 測試**(共置於 `crates/tianheng/tests/self_governance.rs`):
  - `SELF_LAW_PREAMBLE` 常數 + `render_self_law_doc() -> String` = preamble + `constitution_markdown(&Constitution::from(tianheng_constitution()))`。**整份檔由生成器產生**(preamble 也是 Rust 常數,故手寫段同樣進 byte-compare,不會漸漂)。preamble **只准**描述「如何讀投影」與反應迴圈(declare intent in Rust / observe only what has a source / react 0·1·2 / repair toward the declared reason / never weaken law to pass / 三儀 measure, 三司 administer);**不得**做 crate-specific 架構主張——那只能來自生成的投影。
  - `self_law_projection_is_fresh()`:byte-compare checked-in `AGENTS.self-law.md` 與 `render_self_law_doc()`;以 `workspace_root()`(`CARGO_MANIFEST_DIR/../..`)gate——不在 checkout 內(如 packaged crate)則 **skip**,`TIANHENG_WORKSPACE_TESTS` 已設卻缺 workspace 則 **fail-loud**(沿用既有 `workspace_manifest()` 的 repo-only 紀律)。支援 `BLESS=1` 覆寫檔案而非斷言(一鍵 regenerate)。
- **`AGENTS.self-law.md`**(repo root):生成、checked-in。
- **`AGENTS.md`**:加一行指向 `AGENTS.self-law.md`(「Tianheng 自己 enforced 的法讀這份;它由 `self_governance.rs` 生成並被 staleness-check」)。

**明確排除(NON-GOALS)**:`list-self` CLI command(會與 README「已發佈 binary 是 demo、adopter 自帶 constitution」的敘事纏在一起);發明 reason 範例;手寫 reason style guide;curated repair cookbook;新 drift type / 觀測維度。self-law 內容**不**公開為 API(無 `pub self_law_markdown`、無 `pub tianheng_constitution()`)——helper 泛化於任何 `Constitution`,self-law 只是測試裡的第一個 caller。

無 **BREAKING** 變更。

## Capabilities

### New Capabilities
- `self-law-projection`: repo 攜帶一份 agent 可讀的 markdown 投影,**源自與 self-governance gate 同一個 enforced self-constitution**;一個測試在 checked-in 投影與 live 投影不一致時反應(staleness)。涵蓋契約 A,並明定 markdown helper 的契約 B(格式可演進、非機器契約)。

### Modified Capabilities
<!-- 無:既有 `constitution-projection`(list honors the format flag)與 `cli-check-runner` 的 requirements 不變。新 `pub fn` 重用其 private renderer,但既有 CLI 行為、format 契約、exit code 一字不改;故不是 spec-level 行為變更,放 design/Impact 而非此處。 -->

## Impact

- **`crates/tianheng/src/lib.rs`(天衡)**:新增 `impl From<GnomonConstitution> for Constitution`(加性 trait impl);`Constitution` struct / `new` / 既有 builder 不變。
- **`crates/tianheng/src/runner.rs`(天衡)**:新增 `pub fn constitution_markdown`,內部重用 private `list_document`/`list_markdown`(維持 private);CLI dispatch、`list`/`check` 行為、format 契約、usage、exit code **一字不變**。
- **`crates/tianheng/tests/self_governance.rs`**:`tianheng_constitution()` 與 `tianheng_governs_itself()` 的狗糧證明**不動**(仍以 public 面 `check(&tianheng_constitution(), …)` 自治);新增 `SELF_LAW_PREAMBLE`、`render_self_law_doc()`、`workspace_root()`、`self_law_projection_is_fresh()`。
- **`AGENTS.self-law.md`(新檔,repo root)**:生成的 self-law 投影。
- **`AGENTS.md`**:加一行入口指引。
- **相依 / 版本**:無新外部相依;non-breaking;**0.1.1 patch**(SemVer 分類;實際版號 bump 在 `release: 0.1.1` commit,不在本 change)。既有 JSON / CLI 消費者不受影響;新 `pub fn` 為加性公開面,其 markdown 格式明示為「可演進、非機器契約」。
