# Tasks

## 1. The shared primitive

- [ ] 1.1 Add `xingbiao::path_label(&Path) -> String`, built on `Path::components()` joined with `/`,
      each component's bytes percent-escaped by the rule `probes::encoded` already states. Document
      why `components()` rather than character substitution, and the two normalizations it implies
      (`./a` → `a`, `a//b` → `a/b`).
- [ ] 1.2 Unit-test it: realistic relative paths byte-identical to the previous encoding; a `%` path;
      a non-UTF-8 path; an absolute path keeping its leading `/`; a unix backslash staying one
      component. Include the platform-invariant assertion (no label contains `MAIN_SEPARATOR` unless
      it is `/`), labelled as vacuous-on-unix / load-bearing-on-Windows.

## 2. 圭表 and 渾儀's compilation unit

- [ ] 2.1 Rewrite `compilation_unit_label` on `path_label`, so its `None` means only "the root is not
      under the manifest directory". Update its doc, including the no-`manifest_path` fallback.
- [ ] 2.2 Verify the two `out_of_package_root_error` texts are now true whenever they fire, and that a
      non-UTF-8 root inside the package directory is governed rather than refused.
- [ ] 2.3 Guard: a non-UTF-8 root inside the package is governed and carries an escaped unit label.
      Run it before 2.1 and record the refusal it produces instead.

## 3. 漏刻's observed file

- [ ] 3.1 Retire the private `encoded` in favour of `path_label`; `labeled` keeps its
      strip-prefix-else-absolute rule, which the shared function does not change.
- [ ] 3.2 Confirm no `unauditable-probe` label changes for any existing test — the injectivity and
      absolute-`#[path]` rules this window established must both still hold.

## 4. Declaration

- [ ] 4.1 Sync the `structured-violation-identity` delta.
- [ ] 4.2 CHANGELOG: the Windows re-key it prevents, the non-UTF-8 input it starts governing, and the
      measured no-op for existing unix baselines.
- [ ] 4.3 Full Definition of Done (13 gates), and record the Windows half as argued rather than
      executed.
