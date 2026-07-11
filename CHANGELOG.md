<!-- Copyright 2026 John Wilger -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.13](https://github.com/jwilger/emc/compare/v0.1.12...v0.1.13) - 2026-07-11

### Fixed

- recalculate failed trunk release
- retain release configuration during recovery
- recover pending green-trunk releases
- enable release-plz Git releases
- close Tiber tasks from local hooks

### Other

- keep lint preflight fast
- fetch release probe baseline tag
- gate long checks on lint
- preserve published baseline in release probe
- *(release)* v0.1.13
- run release versioning probe in CI
- publish releases from green trunk
- adopt direct trunk development workflow ([#11](https://github.com/jwilger/emc/pull/11))

## [0.1.12](https://github.com/jwilger/emc/compare/v0.1.11...v0.1.12) - 2026-07-05

### Other

- migrate merge queue from Trunk to Mergify ([#8](https://github.com/jwilger/emc/pull/8))
- update repository references after transfer ([#3](https://github.com/jwilger/emc/pull/3))
- submit pull requests to Trunk merge queue ([#4](https://github.com/jwilger/emc/pull/4))
- update gha-workflows references for org rename to slipstream-eng
- Migrate CI/CD from Forgejo Actions to GitHub Actions
- *(workflows)* remove redundant main checks

## [0.1.11](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.10...v0.1.11) - 2026-06-21

### Added

- update workflow entry lifecycle facts
- update workflow transition evidence
- update workflow owned definitions
- update workflow transitions
- update and remove workflow command errors
- update and remove workflow outcomes
- update and remove data flows
- update and remove board connections
- update and remove board elements
- update and remove external payload definitions
- update and remove translation definitions
- update and remove automation definitions ([#182](https://git.johnwilger.com/Slipstream/emc/pulls/182))
- update and remove outcome definitions ([#180](https://git.johnwilger.com/Slipstream/emc/pulls/180))
- update and remove view controls ([#178](https://git.johnwilger.com/Slipstream/emc/pulls/178))
- update and remove view definitions ([#176](https://git.johnwilger.com/Slipstream/emc/pulls/176))
- update and remove read model definitions ([#174](https://git.johnwilger.com/Slipstream/emc/pulls/174))

### Fixed

- update contract scenarios from the CLI

### Other

- finalize modeled element mutation ledger

## [0.1.10](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.9...v0.1.10) - 2026-06-20

### Added

- update and remove event definitions ([#172](https://git.johnwilger.com/Slipstream/emc/pulls/172))

## [0.1.9](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.8...v0.1.9) - 2026-06-20

### Added

- update and remove command definitions ([#170](https://git.johnwilger.com/Slipstream/emc/pulls/170))

## [0.1.8](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.7...v0.1.8) - 2026-06-20

### Added

- update and remove slice scenarios ([#168](https://git.johnwilger.com/Slipstream/emc/pulls/168))

## [0.1.7](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.6...v0.1.7) - 2026-06-20

### Added

- expose modeling guidance as library API ([#166](https://git.johnwilger.com/Slipstream/emc/pulls/166))

## [0.1.6](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.5...v0.1.6) - 2026-06-20

### Added

- expose modeling guidance through CLI and MCP ([#164](https://git.johnwilger.com/Slipstream/emc/pulls/164))

### Other

- draft modeling process guidance ([#162](https://git.johnwilger.com/Slipstream/emc/pulls/162))

## [0.1.5](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.4...v0.1.5) - 2026-06-18

### Added

- *(conflict)* record resolutions as replayable ConflictResolved events ([#151](https://git.johnwilger.com/Slipstream/emc/pulls/151))

### Other

- *(events)* verify the persisted transaction envelope is complete ([#157](https://git.johnwilger.com/Slipstream/emc/pulls/157))
- *(verify)* assert readiness withheld when frontier moves mid-verify ([#155](https://git.johnwilger.com/Slipstream/emc/pulls/155))
- *(runtime)* exercise every command at the eventcore dispatch boundary ([#153](https://git.johnwilger.com/Slipstream/emc/pulls/153))
- *(verify)* prove readiness stays out of the projection fingerprint ([#149](https://git.johnwilger.com/Slipstream/emc/pulls/149))
- *(store)* assert eventcore-fs gitignore keeps the log sole truth ([#147](https://git.johnwilger.com/Slipstream/emc/pulls/147))
- correct event-store references to eventcore-fs source of truth ([#145](https://git.johnwilger.com/Slipstream/emc/pulls/145))

## [0.1.4](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.3...v0.1.4) - 2026-06-18

### Fixed

- *(events)* keep removal projection total so the store can't wedge ([#132](https://git.johnwilger.com/Slipstream/emc/pulls/132))
- *(slice)* validate event command-input source at write time ([#131](https://git.johnwilger.com/Slipstream/emc/pulls/131))
- *(formal-slice)* merge same-name child definitions ([#130](https://git.johnwilger.com/Slipstream/emc/pulls/130))
- *(verify)* prove content invariants with native_decide ([#129](https://git.johnwilger.com/Slipstream/emc/pulls/129))

### Other

- *(lints)* enable pedantic+restriction, enforce no-panic policy ([#143](https://git.johnwilger.com/Slipstream/emc/pulls/143))
- *(emit)* Lean/Quint are write-only projections of the event log ([#141](https://git.johnwilger.com/Slipstream/emc/pulls/141))
- *(events)* reconcile event index from jsonl on load ([#133](https://git.johnwilger.com/Slipstream/emc/pulls/133))

## [0.1.3](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.2...v0.1.3) - 2026-06-15

### Other

- *(deps)* upgrade eventcore workspace to 1.0.0 ([#127](https://git.johnwilger.com/Slipstream/emc/pulls/127))

## [0.1.2](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.1...v0.1.2) - 2026-06-14

### Fixed

- *(release-plz)* describe the release in the PR title and fix body ([#123](https://git.johnwilger.com/Slipstream/emc/pulls/123))
- *(release-plz)* template compliant release PR title and body ([#121](https://git.johnwilger.com/Slipstream/emc/pulls/121))

### Other

- skip auto_review metadata gate for automated release PRs ([#125](https://git.johnwilger.com/Slipstream/emc/pulls/125))

## [0.1.1](https://git.johnwilger.com/Slipstream/emc/compare/v0.1.0...v0.1.1) - 2026-06-13

### Added

- *(events)* migrate event store to eventcore 0.9 with eventcore-fs ([#117](https://git.johnwilger.com/Slipstream/emc/pulls/117))

### Fixed

- *(release-plz)* keep copyright header atop the generated changelog ([#119](https://git.johnwilger.com/Slipstream/emc/pulls/119))
- *(release-plz)* supersede release PR instead of force-pushing ([#115](https://git.johnwilger.com/Slipstream/emc/pulls/115))

## [Unreleased]

## [0.1.0](https://git.johnwilger.com/Slipstream/emc/releases/tag/v0.1.0) - 2026-06-12

### Added

- add verify progress and parallelism
- split navigation transition endpoints
- type exported events and formal root records
- add event-sourced runtime readiness
- remove generated quint json
- verify workflow behavior surface
- verify state-view slice boundary
- verify slice behavior boundary
- verify shared event boundaries
- verify slice coherence
- verify composition structure
- verify bit semantics preservation
- verify data-flow source chains
- verify data-flow source semantics
- verify data-flow transformation semantics ([#25](https://git.johnwilger.com/Slipstream/emc/pulls/25))
- verify data-flow source bits ([#24](https://git.johnwilger.com/Slipstream/emc/pulls/24))
- verify bit encoding consistency ([#23](https://git.johnwilger.com/Slipstream/emc/pulls/23))
- verify displayed source chains
- verify root source-chain completeness
- verify bit-complete data-flow coverage
- verify data-flow targets
- verify model data-flow coverage
- verify state-view command targets
- verify contract scenario coverage
- exempt async lifecycle from entry reachability
- verify invocation argument command inputs
- verify navigation entry views
- verify workflow event participation
- verify generated event source kinds
- verify read model derivation completeness
- enforce root command input provenance
- read formal workflow graphs
- advertise review timestamp schema
- validate review timestamps
- record clean reviews
- validate rejects project drift
- remove modeled workflows
- remove modeled slices
- remove modeled transitions
- update modeled slice names
- update modeled workflow names
- update modeled slice kinds
- list modeled transitions
- update modeled slices
- list modeled slices
- show modeled slice documents
- expose project init over mcp
- add cli help surface
- digest formal slice artifacts
- verify formal slice artifacts
- emit formal slice artifacts
- verify structured workflow transitions
- structure formal workflow transitions
- digest workflow graph artifacts
- strengthen formal artifact digests
- return HTTP errors for malformed MCP JSON
- build container image in flake checks
- constrain project path arguments
- return MCP tool errors as JSON-RPC
- report actionable verify failures
- require indexed workflow reads
- reject noncanonical validation slice paths
- reject noncanonical slice reference paths
- reject duplicate browser workflow slugs
- reject duplicate browser workflow names
- reject duplicate browser index workflows
- check lean module closing ownership
- check formal identity invariants
- check formal module ownership
- canonicalize formal digest markers
- check formal invariant drift
- verify emitted quint invariants
- emit slice detail formal invariant
- preserve all workflow transition kinds
- reject invalid referenced slice json
- reject duplicate referenced slice keys
- support workflow exit transitions
- support external trigger transitions
- reject duplicate browser index keys
- expose project check over mcp
- reject duplicate browser workflow keys
- reject duplicate browser workflow fields
- reject duplicate canonical workflow fields
- reject duplicate canonical slice declarations
- reject duplicate canonical transition declarations
- advertise workflow transition kinds over mcp
- support command and event workflow transitions
- preserve workflow composition on update
- emit canonical slice details
- check canonical artifact ownership
- check slice reference drift
- align generated slice references
- emit valid workflow shape
- check workflow index drift
- validate event model files
- remove import path
- verify lean artifacts through lake
- add verification entrypoint artifacts
- run all gherkin suites
- require auth for exposed mcp http
- accept mcp http bind options
- serve mcp over local http
- execute gherkin runner suites
- report gherkin runner scenario counts
- fail incomplete gherkin runner suites
- list gherkin runner suites
- brand generated site from project
- project view control effects
- project view source chains
- project command definition references
- project browser review overlays
- render error recovery cards
- label retry transitions
- label alternate outcome branches
- label browser workflow transitions
- render browser branch cards
- compose browser workflow lanes and main path
- require follow-up review after corrections
- expose review gate over mcp
- bind review records to workflows
- block current mandatory review findings
- block stale workflow reviews
- add workflow review gate command
- check referenced browser slices
- check workflow slice drift
- check workflow transition drift
- connect workflow steps through cli and mcp
- rewrite canonical artifacts when adding slices
- add slices through cli and mcp
- check workflow field drift
- emit richer lean and quint workflows
- update workflows through cli and mcp
- check browser workflow drift
- add workflows through cli and mcp
- validate event models over mcp
- verify project over mcp
- generate site over mcp
- bundle browser site assets
- expose workflow reads over mcp
- serve mcp stdio
- verify proof artifacts
- generate site data
- preserve workflow descriptions
- show workflow
- list workflows
- qualify control error handling
- require visible decision fields
- require visible actor inputs
- require control input descriptions
- require control input sources
- require control inputs
- validate control commands
- trace command input read models
- require command input provenance
- require command input source chains
- require view source chains
- require view fields to use referenced read models
- require view field sources
- require read model field sources
- require event attribute sources
- require wireframes to show controls
- require wireframes to show fields
- validate wireframe tokens
- require view wireframes
- forbid wireframes in state change slices
- forbid controls in state change slices
- forbid translations in state change slices
- forbid automations in state change slices
- forbid read models in state change slices
- forbid views in state change slices
- require state change slices to emit events
- forbid commands in state view slices
- require state view slices to own views
- reject invalid workflow slice files
- reject missing workflow slice files
- require read model board updates
- validate view command board edges
- validate event read model board edges
- validate command event board edges
- require command board triggers
- reject noncausal board connections
- reject external events updating read models
- reject external events modeled as automations
- reject undeclared external event bridge elements
- reject undeclared automation board elements
- reject unknown board external event references
- reject unknown board automation references
- reject unknown board read model references
- reject unknown board event references
- reject unknown board view references
- reject unknown board command references
- require board events in events lane
- require board commands in actions lane
- require board read models in actions lane
- require board external events in ux lane
- require board automation elements in ux lane
- require board view elements in ux lane
- require canonical board lane names
- reject duplicate board lanes
- require declared board lanes
- reject non-canonical board lanes
- reject disconnected workflow board islands
- require workflow exit rationale
- require workflow external trigger contracts
- require workflow navigation target resolution
- require workflow navigation source controls
- require workflow command target ownership
- require workflow command source controls
- require workflow transition target events
- require workflow transition source events
- require shared workflow transition events
- require application entry state coverage
- require application entry before bootstrap
- reject workflow scenario selection
- reject workflow internal definitions
- reject main async lifecycle steps
- require alternate workflow rationale
- reject unknown workflow transition targets
- require workflow entry reachability
- require incoming workflow transitions
- reject duplicate workflow steps
- require one workflow entry step
- require referenced slices in workflow steps
- require workflow steps to compose slices
- require workflow composition steps
- require workflow outcome coverage
- reject workflow transitions on command errors
- require translation payload variant coverage
- require automation trigger scenario coverage
- require state view empty state coverage
- require external system handoff contract
- require external workflow navigation target
- reject unknown local view state navigation
- reject unknown modeled view navigation
- require navigation control type
- require recovery behavior for error handling
- require controls to handle command errors
- require command error scenario coverage
- reject undeclared scenario command errors
- split outcome event reference diagnostics
- require outcome events in slice
- reject empty outcome event sets
- reject duplicate outcome event sets
- reject conflicting slice events
- reject duplicate slice wireframes
- reject ambiguous slice scenarios
- reject duplicate slice translations
- reject duplicate slice automations
- reject duplicate slice controls
- reject duplicate slice views
- reject duplicate slice read models
- reject duplicate slice commands
- require automation error handling
- reject multi-command automations
- require automation triggers
- reject undeclared board automations
- reject translation owned views
- require translation external contracts
- require singleton repeat behavior
- require state change scenario streams
- reject legacy command reads
- validate command input external sources
- require absence default scenarios
- validate absence default events
- validate transitive read model derivations
- require derived read model scenarios
- validate derived read model provenance
- validate read model event sources
- reject empty generated event sources
- reject read model event attribute sources
- validate external event attribute fields
- validate external event attribute sources
- validate command sourced event attributes
- require event producers
- validate event stream references
- reject duplicate slice outcomes
- require state view projector scenarios
- reject acceptance scenario event references
- reject duplicate slice scenarios
- validate scenario when fields
- reject legacy slice scenarios
- validate slice file slice count
- reject duplicate command definitions
- require explicit event model board
- validate event model structure
- validate event model json syntax
- detect imported artifact drift
- check imported workflow artifacts
- import emc event model artifacts
- add project layout check
- make init idempotent
- add semantic boundary types
- establish emc guardrails and init slice

### Fixed

- use release-plz bot signing identity
- use verified release signing identity
- create signed release-plz PR commits
- propagate release-plz signing config
- sign release-plz release PR commits
- authenticate release-plz git pushes
- decouple release-plz release PR secret gate
- skip release-plz without secrets
- speed up workflow Lean proofs
- prune stale event-store streams after reset
- default event store under home state
- recover artifact-only projects during init
- *(mcp)* satisfy clippy in schema regression test
- support current Codex MCP protocol
- negotiate MCP initialize protocol version
- include docs in nix package source
- return method error for mcp http get
- replace generated site data
- reject duplicate keys during validation
- check quint module closures
- initialize formal slice directories
- check project manifest drift
- check lean project config drift
- check project root formal drift
- verify project root formal artifacts
- brand generated browser sites from project
- stream mcp stdio responses
- validate mcp http origin by request host
- reject connect workflow description drift
- reject add-slice workflow description drift
- reject connect workflow identity drift
- reject add-slice workflow identity drift
- reject unknown workflow transition targets
- reject duplicate workflow transitions
- reject duplicate workflow slugs
- reject duplicate slice slugs
- reject colliding workflow modules
- reject colliding slice modules
- align initialized quint invariants

### Other

- add release-plz automation
- tighten token-efficient Codex rules
- add token-efficient agent SOP
- *(formal)* type command input source kinds
- *(formal)* type formal slice kinds
- *(formal)* type workflow entry lifecycle states
- *(formal)* type workflow step relationships
- *(formal)* type workflow owned-definition kinds
- *(formal)* type workflow transition kinds
- *(events)* type exported event conflicts
- *(formal)* model data-flow source kind as closed type
- *(events)* add typed slice fact event body boundary
- *(events)* add typed review recorded boundary
- *(events)* add typed slice board connection boundary
- *(events)* add typed slice board element boundary
- *(events)* add typed slice automation boundary
- *(events)* add typed slice translation boundary
- *(events)* add typed slice data flow boundary
- *(events)* add typed slice view boundary
- *(events)* add typed slice read model boundary
- *(events)* add typed slice command definition boundary
- *(events)* add typed slice event definition boundary
- *(events)* add typed slice external payload boundary
- type slice outcome event payload
- type slice scenario event payload
- type workflow readiness event payload
- type workflow owned-definition event payload
- type workflow fact event payloads
- type workflow transition event payloads
- type slice event payload boundary
- type workflow event payload boundary
- type command observed stream references
- type command emitted event references
- type slice view references
- type slice read model references
- type slice command references
- type slice event references
- type workflow slice references ([#61](https://git.johnwilger.com/Slipstream/emc/pulls/61))
- serialize slice fact event bodies semantically ([#60](https://git.johnwilger.com/Slipstream/emc/pulls/60))
- render lean workflow facts as records
- render lean automation facts as records
- render lean translation facts as records
- render lean external payload facts as records
- render lean board facts as records
- render lean view facts as records
- render lean read model facts as records
- render lean project event facts as records
- render lean workflow facts as records
- name exported event body model
- type conflict resolution ids
- record exported event metadata
- type exported event metadata
- type runtime exported event import
- internalize unit tests
- mark formal rules complete
- mark slice boundary enforced
- mark duplicate validation removed
- fix README status rules link
- fix README formal rules links
- fix README status rules link
- fix formal rules readme link
- fix README formal rules links
- clarify README status headings
- fix README rules link
- move current status in readme
- request auto_review semantic review
- align Forgejo CI and root scenario proofs
- Update Forgejo CI and data-flow verification
- Add Forgejo CI workflow
- Verify project control input completeness
- Model external payload flows and project licensing
- Verify command input source invariants
- Model session command input sources
- Model generated command input sources
- Model external payload command input sources
- Model event-stream command input sources
- Keep testing policy out of formal rules
- Model read-model derivation source fields
- Model shared event definitions
- Model external workflow navigation
- Model local view navigation state
- Model command observed streams
- Model project root board facts
- Model project root view controls
- Model project root view definitions
- Model project root automation definitions
- Model project root translation definitions
- Model project root external payload fields
- Model project root scenario definitions
- Model project root read model definitions
- Model project root read model fields
- Model project root view fields
- Model project root event attributes
- Model project root command inputs
- Remove JSON navigation helper
- Model project root data flows
- Model project root command errors
- Model project root outcomes
- Model project root scenarios
- Model project root external payloads
- Model project root translations
- Model project root automations
- Model project root views
- Model project root read models
- Model project root commands
- Model project root events
- Model project root streams
- Model root slice modules
- Remove source text audit tests
- List external trigger provenance invariant
- Model workflow slice modules
- Prove project root identity digest
- Model slice membership in formal root
- Model workflows in formal root
- Parallelize formal verification endpoints
- Drop legacy runner absence rule
- Remove stale read model command invariant
- Drop workflow JSON document path
- Prove external system navigation contracts
- Remove legacy presentation rules
- Prove external workflow navigation targets
- Prove local view navigation targets
- Drop legacy transition parsing
- Remove legacy invalidation rules
- Prove modeled view navigation targets
- Prove navigation controls declare types
- Prove control input descriptions
- Prove standalone command input provenance
- Prove session input descriptions
- Prove actor input visibility
- Prove control decision visibility
- Prove workflow command targets
- Prove workflow command control sources
- Explain Lean4 and Quint in README
- Prove automation board elements
- Prove view command board edges
- Prove event projection board edges
- Prove command event board edges
- Prove external board event modeling
- Prove workflow payload provenance
- Prove external boundary provenance
- Prove command input source chains
- Prove displayed data provenance
- Prove stored event fact provenance
- Prove state-view projection ownership
- Prove state-change slices omit controls
- Prove state-change event ownership
- Prove automation slices have one reaction
- Represent formal model version in roots
- Prove state-change outcome ownership
- Prove shared event identity in workflows
- Resolve external trigger payload contracts formally
- Count formal event definitions for state changes
- Prove translations reference observed events
- Prove external triggers match translations
- Prove external events do not update read models
- Require nonempty automation and translation slices
- Sync project Quint workflow invariants
- Prove command transitions resolve owned controls
- Prove navigation transitions resolve owned controls
- Prove contract scenarios target modeled definitions
- Prove control recovery behavior is modeled
- Prove scenario error references resolve
- Prove workflow transition ownership
- Prove workflow reachability from entry
- Document formal model authority split
- Encode workflow entry lifecycle coverage formally
- Make formal artifacts the event model authority
- Update README for QA CLI changes
- Improve CLI usability from QA findings
- Remove browser-derived formal check paths
- Check browser workflows against formal projections
- Represent workflow exit rationale formally
- Mutate workflows from formal projections
- Review formal workflow projections
- Check project from formal artifacts
- Show model data from formal projections
- List model data from formal artifacts
- Project site data from formal artifacts
- Add EMC flake app metadata
- Add repository ignore rules
- Complete documentation accuracy pass
- Fix third-round QA findings
- Expose EMC Nix overlay
- Fix second-round QA findings
- Fix QA findings for generated models and MCP verification
- close completion audit guardrails
- enforce semantic core collections
- wrap browser projection collections
- hide workflow document collections
- wrap effect collections
- wrap modeled layout collections
- hide validation builder internals
- parse validation sources once
- map gherkin scenarios to coverage
- render generated site in package smoke
- audit completion gaps
- smoke review gate in package
- list workflow removal in help
- build mcp payloads with sdk models
- parse project paths at boundaries
- parse init project name at cli boundary
- cover workflow transition boundary parsers
- expand semantic boundary parser coverage
- pin mcp sdk dependency
- share command planning across cli and mcp
- expand user onboarding guide
- model workflow transitions as records
- smoke packaged mcp http tool calls
- reject unmodeled formal slice artifacts
- reject stale slice invariants
- reject stale formal slice artifacts
- emit named transition records
- strengthen formal transition invariants
- guard command input gherkin semantics
- derive browser view definitions semantically
- derive browser command definitions semantically
- derive browser error recovery semantically
- derive browser event elements semantically
- derive browser lanes semantically
- derive browser review overlays semantically
- derive browser transition cards semantically
- derive browser branch cards semantically
- derive browser main path semantically
- parse json object checks semantically
- parse review records semantically
- share browser index path parsing
- share validation slice-file parsing
- share workflow slice-file parsing
- share workflow slice marker parsing
- share workflow transition parsing
- derive digests from workflow documents
- isolate connection workflow document mutation
- isolate slice workflow document mutation
- isolate workflow document parsing
- align emitter digest expectations
- document modeling and mutation workflow
- smoke packaged workflow mutations
- route validation reads through effects
- route mcp project reads through effects
- route cli file reads through effects
- add manual mutation testing gates
- explain emc for non-specialists
- wrap package with verification tools
- smoke test packaged emc commands
- package docker-compatible image
- smoke test nix package checks
- run strict rust gate
- record gherkin fixture parity
- add event model runner meta fixture
- add browser gherkin fixture
- record review gate progress
- add review gate gherkin fixture
- record referenced slice check progress
- record artifact drift guardrail progress
- cover hidden session inputs
- cover direct event view field sources
- pin read model command board edge rejection
- cover event conflict provenance
- allow identical slice events
- import emc validator gherkin fixtures
