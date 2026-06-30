## Why

潛移(PROJECT.md「潛移 (Qiányí)」)的立論:autoregressive agent 模仿 context 裡的 idiom,而 boundary 的 **reason 是重力承載物**(可生成的母則)。但目前 markdown 投影把 reason 渲染在**最後**,當成 kind/severity、rule 之後的一個對等 metadata bullet —— 最該被模仿的內容被機械分類埋住了。

把它前置:reason 應讀作 boundary block 的**原則**,在機械的 rule 與分類之前。這正是 **Contract B** 預留的自由——markdown layout 是 human/agent surface,可在相容版本中演進;它不改 law、不改 reaction、不改 JSON。

本 change 屬 **0.1.1 patch**(SemVer 分類;實際版號 bump 在 `release: 0.1.1` commit,不在本 change)。它是 self-law-projection spec 之 Contract B 所留「Markdown layout may evolve」自由的**第一個 layout 演進**(Contract B 本身在前一個 change 引入)。

## What Changes

採形狀 **B2(reason 當領頭 blockquote)**。每條 boundary,`boundary_markdown()` 渲染為:

```
### `<target>`            (heading 仍是 target — 保持顯眼/可掃)
> <reason>                (reason 存在時:領頭 blockquote — 原則,與 metadata 分離)

- **rule**: <rule> (<params>)
- **kind**: <kind> · **severity**: <severity>[ · **crate**: <crate>]
```

順序 **reason → rule → kind/severity**:reason 是原則,rule 是 reaction 的機械形狀,kind/severity 是分類。無 reason 的 boundary 不輸出 blockquote、也不留孤兒空行(沿用既有 `if !reason.is_empty()` guard)。`crate` 區段**僅 module boundary 才有**(crate boundary 的 JSON 無 `crate` 鍵)—— 故自我治理的七條 crate boundary 重生後都不會有 `crate` 區段。

- **`boundary_markdown()` 改為 B2 形狀**(`crates/tianheng/src/runner.rs`):`list_document`/`list_markdown`/`constitution_markdown` 結構不變,只改 per-boundary block 佈局。byte-exact helper-vs-CLI 測試仍成立(兩邊同 renderer);既有覆蓋測試用 `.contains()`(順序無關)亦存活。
- **doc-comment 更新**:`boundary_markdown` / `list_markdown` / `constitution_markdown` 註明 markdown 前置 declared reason(它是 agent 的 repair/imitation hint);重申 Contract B(layout 可演進;JSON 才是機器契約)。
- **foregrounding 不變式測試**:runner.rs 加單元測試,斷言 reason 文字的 byte-index 早於 rule、早於 kind/severity。此測試**構造上只斷言順序、絕非 golden/byte snapshot**,所以它本身不凍結格式。要釐清的是:全域「不得 byte-snapshot markdown」的綁定守則仍**屬 Contract B(self-law-projection,未改動、review-verified)**,本 change 不接管它;本 requirement 只把「reason 被前置」這個不變式加進來,並明文聲明它是 Contract B 下的一個窄例外,不擴張成新的凍結。
- **重生 `AGENTS.self-law.md`**:`BLESS=1 cargo test -p tianheng self_law_projection_is_fresh` 把 self-law 投影重生成新形狀並 commit(staleness test 在重生前會 fail —— 這是預期且正確的 ripple)。
- **(D,搭便車 docs-only)adopter recipe**:在 README「published binary is a demo」附近(或 crate docs)加一段一行 recipe:`let md = tianheng::constitution_markdown(&constitution()); std::fs::write("AGENTS.<project>-law.md", md)?;` —— adopter 投影自己 agent-context 的 library primitive。**不**做 generator、**不**做 list-self / CLI。

**明確排除(NON-GOALS)**:不改 JSON / text 投影;不改 reaction / law / SemVer 身份;markdown 不做 golden/byte snapshot;不做 generator 或 list-self CLI;不加新 boundary / 維度。

無 **BREAKING** 變更(markdown layout 演進為 Contract B 明示允許之非破壞;JSON 消費者不受影響)。

## Capabilities

### New Capabilities
<!-- 無新 capability:既有投影面的加性演進。 -->

### Modified Capabilities
- `constitution-projection`: **MODIFIED**「List honors the format flag」—— 此 requirement 已是 markdown 投影的擁有者(已列舉 target/rule/reason 欄位集合、順序不拘)。本 change 把「Markdown SHALL foreground the reason(reason 渲染在 rule 與 kind/severity 之前)」這個**順序不變式**併入該 requirement,並明文聲明它**只鎖順序、不凍結 layout**,且全域「不得 byte-snapshot」仍屬未改動的 Contract B(`self-law-projection`),本不變式為其下窄例外。不另開 ADDED requirement(避免兩條 requirement 治理同一個 markdown 面)。

<!-- self-law-projection 不需變更:它已說「same renderer、may evolve、no golden test」;其 snapshot 隨重生跟上。 -->

## Impact

- **`crates/tianheng/src/runner.rs`**:`boundary_markdown()` 改 B2 佈局;新增 foregrounding 不變式單元測試;doc-comment 更新。`list_document`/`list_markdown`/`constitution_markdown` 與 CLI dispatch、`list`/`check` 行為、JSON、exit code **不變**。
- **`AGENTS.self-law.md`**:重生為 reason-前置形狀(內容同源,僅佈局演進)。
- **README / crate docs**:加一段一行 adopter recipe(D)。
- **相依 / 版本**:無新外部相依;non-breaking(Contract B);**0.1.1 patch**(SemVer 分類;bump 在 release commit)。JSON / text 消費者不受影響。
