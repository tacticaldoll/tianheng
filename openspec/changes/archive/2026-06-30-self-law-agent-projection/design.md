## Context

Tianheng 的潛移面立論(PROJECT.md「潛移 (Qiányí)」)說:autoregressive agent 是模仿引擎,治理多一根「重力」軸——讓 enforced 的法以可仿的形式進入 agent context,反應仍是兜底。但目前 agent 照 AGENTS.md 跑 `tianheng list --format markdown` 讀到的是 **demo constitution**(`example-core`),而真正的 self-law 在 test-only 的 `tianheng_constitution()`。入口錯位使最有價值的 idiom 不在 context 裡。

本 change 是潛移面的第一個 dogfood,規模小但踩到兩個必須分清的契約與一個 Rust 可見性的硬約束。它接續 release/0.1.1 上已 commit 的兩個契約修訂(79f6249 潛移 thesis、71bb154 收束於周界 + reason 書寫慣例)。

## Goals / Non-Goals

**Goals:**
- 讓 Tianheng 自己 enforced 的 self-law,以**忠實、生成、會被測試追責**的 markdown 投影進入 agent context。
- 投影與 self-governance gate **同源**(同一個 `tianheng_constitution()`),不產生第二事實源。
- 投影走**同一個 renderer**(CLI `list --format markdown` 用的那個),不維護平行投影路徑。

**Non-Goals:**
- `list-self` CLI command(後述,刻意不做)。
- adopter-facing 採用率(本 change 只解 dogfood/contributor 迴圈;adopter 潛移面是後續 phase)。
- 把 self-law 內容公開成 API;發明 reason 範例 / style guide / repair cookbook;新 drift type。
- 凍結 markdown 格式(恰恰相反——見契約 B)。

## Decisions

### 兩條正交契約(本 change 的核心,規格層分立)

混淆它們會讓真風險溜走:

- **契約 A — repo artifact 的 staleness**:受檢查的物件是 `AGENTS.self-law.md` 這個**檔**;enforcer 是本地 `#[test]`;格式可變(改 renderer → regenerate → 測試綠);它**只保「檔不漂」,完全不碰「格式能不能演進」**。
- **契約 B — public API 的格式穩定性**:受檢查的物件是 `constitution_markdown` 的**輸出形狀**;audience 是 repo 外的 adopter;enforcer 是 SemVer;**測試與 `BLESS` 都救不了它**(破壞發生在 adopter 程式碼裡,本 repo CI 看不到)。

反證:改 markdown 排版(潛移想做的 reason-foregrounding),契約 A 的測試恆綠(重生即可),會壞的是某個 call 了 pub fn 的 adopter。所以契約 B 的解藥**不是測試,是 pub fn 上的 doc 承諾**:markdown 是人類/agent surface、可演進;JSON 才是機器契約。少了它,本 change 會無意間凍死下一個 change 想做的潛移演進。

### 落地法選 B-min(expose 最小投影 helper),否決 A 與 C′

可見性硬牆:`list_document`/`list_markdown` 是 src-private,只有 src 內 unit test 看得到;`tianheng_constitution()` 在 integration test(`tests/`),只看得到 `pub`。snapshot test 同時需要兩者,於是被逼三選一,**每個都得割讓一樣專案在乎的東西**:

- **(A) 把 self-law + 自治測試 + snapshot 全搬進 `src/` 當 unit test。** 割讓:`self_governance.rs` 作為 **integration test 透過 public 面證明自治**這個性質——對一個賣點是「adopter 自己接起來」的治理工具,這個狗糧證明很值錢。**否決。**
- **(B-min) 多一個 `pub fn constitution_markdown(&Constitution) -> String`。** 自治測試原地不動、走公開面證明完整;單一來源;同一 renderer。代價:一個對 adopter 而言邊際有限的公開面(adopter 的 binary 綁自己的 constitution,CLI `list --format markdown` 已夠)。**選此。** 正當化:`list --format markdown` 已是公開行為,此 fn 只是把同一投影提供給 library caller;加性、0.1.1(此為 SemVer 分類;workspace 版號 bump 在 `release: 0.1.1` commit 落,不在本 change,與前兩個 0.1.1 change 一致)。
- **(C′) 加 `list-self` command。** 割讓:會讓 binary 同時背 demo-law 與 self-law,與 README「published binary 是 demo、adopter 自帶 constitution」的敘事纏在一起。**否決(至少第一版)。**

> 「B-min 但不公開」是不可能的:integration test 看不到 `pub(crate)`,所以 helper **必然 `pub`**。誠實地承認它是一個**刻意的、最小的公開能力**,而非粉飾成「非 public」——粉飾會導致漏掉契約 B 的 doc 承諾。

### `From<GnomonConstitution> for Constitution` lift —— 保狗糧證明一字不動

`list_document` 吃 `&Constitution`,self-law 是 `GnomonConstitution`。兩條橋:(x) 讓 `tianheng_constitution()` 改回 `Constitution`,自治測試改用 `.static_boundaries()`(會動到那行); (y) 加 `impl From<GnomonConstitution> for Constitution`,自治測試 `check(&tianheng_constitution(), …)` **完全不動**,投影端才 `.into()`。**選 (y)**:最大化保護那個珍貴的 integration 狗糧證明。lift 是 trivial(`static_` 取入參,`semantic`/`runtime` 取 default/empty)。self-law 全是靜態 boundary,投影出來只有「Static boundaries」段,忠實。

### 整份檔生成,preamble 也是常數

不要半生成半手寫。`SELF_LAW_PREAMBLE` 是 Rust 常數,`render_self_law_doc()` = preamble + projection,**整份 byte-compare**。如此手寫的 preamble 也被追責、不會漸漸漂。preamble 的內容界線(規格 requirement 4):只准講「怎麼讀投影 + 反應迴圈」,**不准** crate-specific 架構主張——那只能來自有反應背書的生成投影(否則就是 PROJECT.md 說的 open-loop prose prescription)。

### staleness test 沿用 repo-only 紀律 + `BLESS`

`workspace_root()` 用 `CARGO_MANIFEST_DIR/../..`:不在 checkout 內則 skip;`TIANHENG_WORKSPACE_TESTS` 已設卻缺 workspace 則 fail-loud(與既有 `workspace_manifest()` 同紀律,絕不默默 skip 守護 false-negative)。`BLESS=1` 時覆寫 `AGENTS.self-law.md` 而非斷言——讓 markdown 的變更是**程式生成**而非人腦同步,「Do not edit by hand」的檔頭才不是謊話。

### 放 AGENTS 鄰近,入口才被吃到

snapshot 放 repo root 的 `AGENTS.self-law.md`;`AGENTS.md`(agent 自然入口)加一行指向它。不內嵌進 AGENTS.md——AGENTS 要短,snapshot 會長。

## Risks / Trade-offs

- **[公開了 markdown renderer → 可能被當穩定契約]** → 契約 B 的 doc 承諾明示「可演進、機器用 JSON」;規格 requirement 3 把它釘成 spec-level 約束。
- **[snapshot byte-compare 的換行/平台脆性]** → 生成與比對走同一 `render_self_law_doc()`;`BLESS` 一鍵重生;CI 在 checkout 內必跑。
- **[preamble 變成 codegen 常數,改 agent 文案要動 Rust + regen]** → 接受的小摩擦,換「手寫段不漂」。
- **[定位過窄:只動 dogfood、不動 adopter 採用]** → 明確承認;本 change 是潛移面原型,adopter-facing 版本是後續 phase,不在此 scope。
- **[搬移風險]** → 本方案**不**搬 `self_governance.rs`(否決 A 的理由),狗糧證明零變動,故無此風險。

## Open Questions

- 無阻塞性未決項。兩個 propose-時微決定已定:(1) 開獨立 capability `self-law-projection`(與 `constitution-projection` 的「list honors format flag」是不同關注點);(2) `render_self_law_doc` + preamble 共置於 `self_governance.rs`(暫不抽 `tests/support/`)。過 apply 時若發現更佳佈局可翻案。
