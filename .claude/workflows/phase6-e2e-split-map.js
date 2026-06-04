export const meta = {
  name: 'phase6-e2e-split-map',
  description: 'Read-only line-range domain map of the 18,582-line crates/server/tests/e2e.rs to produce a behaviour-preserving split manifest',
  phases: [
    { title: 'Map modules', detail: 'one agent per fixtures-module: line ranges, tests, fixtures, cross-module refs' },
    { title: 'Map shared head', detail: 'map the top-of-file shared imports/helpers (lines 1-142)' },
    { title: 'Synthesize manifest', detail: 'propose module file structure + common/ + every cross-domain helper to relocate' },
  ],
}

// The 15 fixtures-modules + their [start, end] line ranges (end = next module start - 1; last = EOF).
const FILE = 'crates/server/tests/e2e.rs'
const MODULES = [
  { name: 'governance_fixtures',            start: 143,   end: 6090  },
  { name: 'admin_config_fixtures',          start: 6091,  end: 8592  },
  { name: 'v1_jm_b_fixtures',               start: 8593,  end: 10516 },
  { name: 'v1_jm_e_fixtures',               start: 10517, end: 11729 },
  { name: 'v1_sl_b_fixtures',               start: 11730, end: 12765 },
  { name: 'v1_sl_c_fixtures',               start: 12766, end: 13648 },
  { name: 'v1_sl_d_fixtures',               start: 13649, end: 14514 },
  { name: 'v1_sl_e_fixtures',               start: 14515, end: 15486 },
  { name: 'v1_federation_inbound_a_fixtures', start: 15487, end: 15554 },
  { name: 'v1_ship_2_fixtures',             start: 15555, end: 16289 },
  { name: 'v1_federation_inbound_b_fixtures', start: 16290, end: 16626 },
  { name: 'v1_federation_inbound_e_fixtures', start: 16627, end: 16812 },
  { name: 'v1_ship_3_fixtures',             start: 16813, end: 17214 },
  { name: 'v1_rt_r3_fixtures',              start: 17215, end: 18268 },
  { name: 'v1_rt_r4_fixtures',              start: 18269, end: 18582 },
]

const MODULE_SCHEMA = {
  type: 'object',
  required: ['module', 'line_range', 'tests', 'fixtures', 'cross_module_refs', 'top_level_helpers_referenced', 'proposed_domain'],
  additionalProperties: false,
  properties: {
    module: { type: 'string' },
    line_range: { type: 'string', description: 'e.g. "143-6090"' },
    test_count: { type: 'integer' },
    tests: {
      type: 'array',
      description: 'every #[tokio::test]/#[test] fn in this module',
      items: {
        type: 'object',
        required: ['name', 'line'],
        additionalProperties: false,
        properties: {
          name: { type: 'string' },
          line: { type: 'integer' },
        },
      },
    },
    fixtures: {
      type: 'array',
      description: 'helper fns / structs / consts DEFINED in this module that tests call (the fixture surface)',
      items: {
        type: 'object',
        required: ['name', 'line', 'kind'],
        additionalProperties: false,
        properties: {
          name: { type: 'string' },
          line: { type: 'integer' },
          kind: { type: 'string', description: 'fn|struct|const|type|impl|trait' },
          looks_shared: { type: 'boolean', description: 'true if generic enough that other domains likely need it (e.g. create_test_user, seed_community) vs domain-specific' },
        },
      },
    },
    cross_module_refs: {
      type: 'array',
      description: 'identifiers this module references that are DEFINED IN ANOTHER fixtures module (super::other_mod::foo or use super::...). Empty if none.',
      items: { type: 'string' },
    },
    top_level_helpers_referenced: {
      type: 'array',
      description: 'identifiers this module uses that are defined in the top-of-file shared section (lines 1-142): EnvVarGuard, shared use-aliases, etc.',
      items: { type: 'string' },
    },
    proposed_domain: { type: 'string', description: 'the logical test-domain name this module maps to, e.g. jury, sponsor_liability, federation_inbound, admin_config, ship, reputation, migrations, redaction' },
    notes: { type: 'string', description: 'anything that complicates the split: nested mods, #[path], conditional cfg, shared mutable state, ordering dependency between tests' },
  },
}

phase('Map modules')
// Fan out: one agent per module. governance_fixtures is 5,948 lines — its agent does the most work.
const moduleMaps = await parallel(MODULES.map(m => () =>
  agent(
    `You are mapping ONE module of a large Rust integration-test file for a behaviour-preserving split into per-domain files.

FILE: ${FILE} (18,582 lines total; do NOT read the whole file).
YOUR MODULE: \`mod ${m.name}\` spans lines ${m.start}-${m.end}. Read ONLY that line range (use Read with offset=${m.start}, limit=${m.end - m.start + 1}). For a very large range, read it in a few chunks.

Produce a precise structural map of this module:
1. Every test function (#[tokio::test] or #[test]) — name + line.
2. Every fixture/helper DEFINED in this module (fn/struct/const/type/impl/trait) that tests use — name, line, kind. Mark looks_shared=true for any helper generic enough that OTHER test domains would plausibly need it (e.g. a generic create_user / seed_community / pool-setup helper) vs domain-specific ones.
3. cross_module_refs: any identifier this module references that is defined in ANOTHER fixtures module (look for \`super::<other_mod>::\`, \`use super::...\`, or calls to helpers not defined locally and not in the top section). This is the critical de-risking data — it tells us what must move to a shared common module. List the exact paths.
4. top_level_helpers_referenced: identifiers used here but defined in the top-of-file shared section (lines 1-142) — e.g. EnvVarGuard, shared use-aliases.
5. proposed_domain: the logical domain name (jury / sponsor_liability / federation_inbound / admin_config / ship / reputation / migrations / redaction / appeal / messaging — pick the best fit from the module name and test contents).
6. notes: anything that complicates a clean extraction — nested mods, #[path] attrs, cfg gates, shared mutable statics, or tests that depend on execution order or other tests' side effects.

This is READ-ONLY. Do not edit anything. Return the structured map.`,
    { label: `map:${m.name}`, phase: 'Map modules', schema: MODULE_SCHEMA }
  )
))

phase('Map shared head')
const headMap = await agent(
  `Read the TOP section of ${FILE} — lines 1 to 142 ONLY (Read with offset=1, limit=142). This is the shared preamble before the first \`mod governance_fixtures\`.

Catalog exactly what lives here, because everything here is implicitly shared by all test modules and must land in a common module after the split:
1. All \`use\` imports (group them: external crates vs lemmy_* crates vs std).
2. Any top-level test functions (e.g. postgres_container_boots, template_dump_capture) — name + line.
3. Any structs/helpers/consts (e.g. EnvVarGuard) — name + line + kind.
4. Any module-level attributes (#![...]) or cfg gates.

Return as structured data.`,
  {
    label: 'map:head',
    phase: 'Map shared head',
    schema: {
      type: 'object',
      required: ['imports', 'top_level_tests', 'helpers', 'module_attrs'],
      additionalProperties: false,
      properties: {
        imports: { type: 'array', items: { type: 'string' } },
        top_level_tests: { type: 'array', items: { type: 'object', required: ['name','line'], additionalProperties: false, properties: { name: {type:'string'}, line: {type:'integer'} } } },
        helpers: { type: 'array', items: { type: 'object', required: ['name','line','kind'], additionalProperties: false, properties: { name:{type:'string'}, line:{type:'integer'}, kind:{type:'string'} } } },
        module_attrs: { type: 'array', items: { type: 'string' } },
      },
    },
  }
)

phase('Synthesize manifest')
const synthesis = await agent(
  `You are designing a behaviour-preserving split of a 18,582-line Rust integration test (${FILE}) into per-domain module files. You have structural maps of all 15 fixtures-modules and the shared head section.

HARD CONSTRAINTS (this is a TEST refactor — behaviour must be identical):
- Cargo auto-discovers tests/e2e.rs as a single integration-test binary (NO [[test]] entry in Cargo.toml). The canonical behaviour-preserving pattern is: keep \`tests/e2e.rs\` as a THIN ROOT that declares submodules (\`mod common; mod jury; mod sponsor; ...\`), and move each domain into \`tests/e2e/<domain>.rs\`, with shared fixtures in \`tests/e2e/common/mod.rs\` (or \`tests/e2e/common.rs\`). All in ONE test binary → same compile unit, same test discovery, identical pass-count.
- Rust test-module visibility: helpers shared across submodules must be \`pub(crate)\` or \`pub\` and re-exported from \`common\`; submodules reference them via \`use crate::common::...\` or \`super::common::...\`. (cf. the project lesson on cross-crate/cross-module visibility.)
- Every cross_module_ref found in the maps MUST be resolved: the referenced helper moves to \`common\` (if shared) or the two domains merge into one file (if tightly coupled). Enumerate each one and its resolution.
- The shared head (imports + EnvVarGuard + the 2 sanity tests + any statics) goes into \`common\` (imports) and a \`tests/e2e.rs\` root or \`common\` (the sanity tests + EnvVarGuard).

MODULE MAPS:
${JSON.stringify(moduleMaps.filter(Boolean), null, 1)}

SHARED HEAD MAP:
${JSON.stringify(headMap, null, 1)}

Produce a PRECISE split manifest:
1. target_files: the exact set of new files (tests/e2e.rs root content sketch, tests/e2e/common/mod.rs, and one tests/e2e/<domain>.rs per domain) — group the 15 modules into logical domains (some small modules may merge, e.g. the 3 federation_inbound_* into one federation.rs; the ship_2/ship_3 into ship.rs; sl_* into sponsor_liability.rs; jm_* into jury_mechanics.rs; rt_* into reputation.rs). For each target file: which source modules/line-ranges it absorbs, and its test count.
2. common_surface: every helper/struct/const that must live in common/ — name, original location, why (which domains use it). Driven by looks_shared flags + cross_module_refs.
3. cross_ref_resolutions: one row per cross_module_ref from the maps → how it's resolved (move-to-common | merge-domains | leave-as-is-same-file).
4. visibility_changes: which helpers need pub(crate)/pub added when they move to common.
5. risks: anything that could change behaviour or break compile — execution-order deps, shared statics, cfg gates, nested mods, name collisions when modules merge.
6. suggested_subphase_batching: how to sequence the split as several reviewable sub-phases (one domain-group per sub-phase) so each diff is small and each cargo run stays green — recommend the order (start with the domain that has the FEWEST cross_module_refs = lowest risk first).
7. verification_command: the exact laptop command to confirm identical pass-count before/after (cargo-test.bat --workspace --test e2e --features full).

Return structured.`,
  {
    label: 'synthesize:manifest',
    phase: 'Synthesize manifest',
    schema: {
      type: 'object',
      required: ['target_files', 'common_surface', 'cross_ref_resolutions', 'visibility_changes', 'risks', 'suggested_subphase_batching', 'verification_command', 'total_tests_accounted'],
      additionalProperties: false,
      properties: {
        target_files: {
          type: 'array',
          items: {
            type: 'object',
            required: ['path', 'absorbs', 'test_count', 'domain'],
            additionalProperties: false,
            properties: {
              path: { type: 'string' },
              domain: { type: 'string' },
              absorbs: { type: 'array', items: { type: 'string' }, description: 'source modules + line ranges' },
              test_count: { type: 'integer' },
            },
          },
        },
        common_surface: {
          type: 'array',
          items: {
            type: 'object',
            required: ['name', 'origin', 'used_by'],
            additionalProperties: false,
            properties: {
              name: { type: 'string' },
              origin: { type: 'string' },
              used_by: { type: 'array', items: { type: 'string' } },
            },
          },
        },
        cross_ref_resolutions: {
          type: 'array',
          items: {
            type: 'object',
            required: ['ref', 'resolution'],
            additionalProperties: false,
            properties: {
              ref: { type: 'string' },
              resolution: { type: 'string' },
            },
          },
        },
        visibility_changes: { type: 'array', items: { type: 'string' } },
        risks: { type: 'array', items: { type: 'string' } },
        suggested_subphase_batching: {
          type: 'array',
          items: {
            type: 'object',
            required: ['order', 'domain', 'rationale'],
            additionalProperties: false,
            properties: {
              order: { type: 'integer' },
              domain: { type: 'string' },
              rationale: { type: 'string' },
            },
          },
        },
        verification_command: { type: 'string' },
        total_tests_accounted: { type: 'integer', description: 'sum of test_count across target_files — must equal 41 (the current total)' },
      },
    },
  }
)

return { moduleMaps: moduleMaps.filter(Boolean), headMap, synthesis }
