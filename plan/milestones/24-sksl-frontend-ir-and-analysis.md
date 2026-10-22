# Milestone 24: SkSL frontend, IR and analysis

## Outcome and entry gate

Implement lexical input, parser, types, symbol tables, IR, semantic analysis, optimizations and diagnostics. Preserve language version/capability rules and program-kind distinctions. Port generated lexer/table behavior with a reproducible generation path.

Begin implementation only after milestone 23 is marked ACCEPTED by Deepseek after running the tests and completing the audit and the user has handed over this milestone. All earlier accepted tests remain required.

Read `../WORKFLOW.md`, `../MACOS-SCOPE.md` and the inventories before starting. All source paths below are relative to the pinned C++ Skia checkout; all Rust work belongs in the separate rust-skia repository. Never edit the reference checkout to make a test pass.

## Deepseek: implementation contract

Implement lexical input, parser, types, symbol tables, IR, semantic analysis, optimizations and diagnostics. Preserve language version/capability rules and program-kind distinctions. Port generated lexer/table behavior with a reproducible generation path.

- Port the observable behavior of every owned source/header below, including applicable error paths and public declarations. A header is part of the contract, not documentation to skip. Existing Rust files are integration work, not automatic completion.
- Map original symbols to Rust modules in the handoff. Keep production logic under `src/` (or workspace library crates); use explicit Cargo features for optional macOS backends. Preserve widths, ownership, layout when observable, and binary formats where required by original tests.
- Keep Skia-owned algorithmic/runtime code in Rust. External codec/shaping/GPU libraries and Apple framework FFI are allowed only as named, versioned dependencies. Never delegate the implementation to a pre-existing Skia Rust port or C++ Skia.
- Inspect reached dependencies before coding. `forward-includes.csv` contains syntactic warnings, not proof of runtime dependency. Implement a required earlier prerequisite fully and record it in the handoff; if it materially changes this milestone, request a plan amendment before continuing. Never use fake return values, ignored tests or unimplemented runtime branches to clear a gate.
- Do not author or change Codex-owned acceptance tests, expected values, tolerances, fixtures. Deepseek owns execution logs, audit records and acceptance status. Development checks may be separate and must be labeled as such. Fix production code when acceptance tests fail.
- Supply reproducible build commands, feature flags, source/symbol mapping, external dependencies, and known unfinished work using `../HANDOFF-TEMPLATE.md`. Stop after the handoff; do not begin the next milestone.

### Port these production files (Deepseek)

- `include/private/SkSLSampleUsage.h` — port to Rust.
- `include/sksl/SkSLDebugTrace.h` — port to Rust.
- `include/sksl/SkSLVersion.h` — port to Rust.
- `src/core/SkSLTypeShared.cpp` — port to Rust.
- `src/core/SkSLTypeShared.h` — port to Rust.
- `src/sksl/GLSL.std.450.h` — port to Rust.
- `src/sksl/SkSLAnalysis.cpp` — port to Rust.
- `src/sksl/SkSLAnalysis.h` — port to Rust.
- `src/sksl/SkSLBuiltinTypes.cpp` — port to Rust.
- `src/sksl/SkSLBuiltinTypes.h` — port to Rust.
- `src/sksl/SkSLCompiler.cpp` — port to Rust.
- `src/sksl/SkSLCompiler.h` — port to Rust.
- `src/sksl/SkSLConstantFolder.cpp` — port to Rust.
- `src/sksl/SkSLConstantFolder.h` — port to Rust.
- `src/sksl/SkSLContext.cpp` — port to Rust.
- `src/sksl/SkSLContext.h` — port to Rust.
- `src/sksl/SkSLDefines.h` — port to Rust.
- `src/sksl/SkSLErrorReporter.cpp` — port to Rust.
- `src/sksl/SkSLErrorReporter.h` — port to Rust.
- `src/sksl/SkSLFileOutputStream.h` — port to Rust.
- `src/sksl/SkSLGLSL.h` — port to Rust.
- `src/sksl/SkSLGraphiteModules.cpp` — port to Rust.
- `src/sksl/SkSLGraphiteModules.h` — port to Rust.
- `src/sksl/SkSLInliner.cpp` — port to Rust.
- `src/sksl/SkSLInliner.h` — port to Rust.
- `src/sksl/SkSLIntrinsicList.cpp` — port to Rust.
- `src/sksl/SkSLIntrinsicList.h` — port to Rust.
- `src/sksl/SkSLLexer.cpp` — port to Rust.
- `src/sksl/SkSLLexer.h` — port to Rust.
- `src/sksl/SkSLMangler.cpp` — port to Rust.
- `src/sksl/SkSLMangler.h` — port to Rust.
- `src/sksl/SkSLMemoryLayout.h` — port to Rust.
- `src/sksl/SkSLMemoryPool.h` — port to Rust.
- `src/sksl/SkSLModule.cpp` — port to Rust.
- `src/sksl/SkSLModule.h` — port to Rust.
- `src/sksl/SkSLModuleDataDefault.cpp` — port to Rust.
- `src/sksl/SkSLModuleDataFile.cpp` — port to Rust.
- `src/sksl/SkSLModuleLoader.cpp` — port to Rust.
- `src/sksl/SkSLModuleLoader.h` — port to Rust.
- `src/sksl/SkSLOperator.cpp` — port to Rust.
- `src/sksl/SkSLOperator.h` — port to Rust.
- `src/sksl/SkSLOutputStream.cpp` — port to Rust.
- `src/sksl/SkSLOutputStream.h` — port to Rust.
- `src/sksl/SkSLParser.cpp` — port to Rust.
- `src/sksl/SkSLParser.h` — port to Rust.
- `src/sksl/SkSLPool.cpp` — port to Rust.
- `src/sksl/SkSLPool.h` — port to Rust.
- `src/sksl/SkSLPosition.cpp` — port to Rust.
- `src/sksl/SkSLPosition.h` — port to Rust.
- `src/sksl/SkSLProgramKind.h` — port to Rust.
- `src/sksl/SkSLProgramSettings.h` — port to Rust.
- `src/sksl/SkSLSampleUsage.cpp` — port to Rust.
- `src/sksl/SkSLString.cpp` — port to Rust.
- `src/sksl/SkSLString.h` — port to Rust.
- `src/sksl/SkSLStringStream.h` — port to Rust.
- `src/sksl/SkSLUtil.cpp` — port to Rust.
- `src/sksl/SkSLUtil.h` — port to Rust.
- `src/sksl/analysis/SkSLCanExitWithoutReturningValue.cpp` — port to Rust.
- `src/sksl/analysis/SkSLCheckProgramStructure.cpp` — port to Rust.
- `src/sksl/analysis/SkSLCheckSymbolTableCorrectness.cpp` — port to Rust.
- `src/sksl/analysis/SkSLFinalizationChecks.cpp` — port to Rust.
- `src/sksl/analysis/SkSLGetLoopControlFlowInfo.cpp` — port to Rust.
- `src/sksl/analysis/SkSLGetLoopUnrollInfo.cpp` — port to Rust.
- `src/sksl/analysis/SkSLGetReturnComplexity.cpp` — port to Rust.
- `src/sksl/analysis/SkSLHasSideEffects.cpp` — port to Rust.
- `src/sksl/analysis/SkSLIsConstantExpression.cpp` — port to Rust.
- `src/sksl/analysis/SkSLIsDynamicallyUniformExpression.cpp` — port to Rust.
- `src/sksl/analysis/SkSLIsSameExpressionTree.cpp` — port to Rust.
- `src/sksl/analysis/SkSLIsTrivialExpression.cpp` — port to Rust.
- `src/sksl/analysis/SkSLNoOpErrorReporter.h` — port to Rust.
- `src/sksl/analysis/SkSLProgramUsage.cpp` — port to Rust.
- `src/sksl/analysis/SkSLProgramUsage.h` — port to Rust.
- `src/sksl/analysis/SkSLProgramVisitor.h` — port to Rust.
- `src/sksl/analysis/SkSLReturnsInputAlpha.cpp` — port to Rust.
- `src/sksl/analysis/SkSLSpecialization.cpp` — port to Rust.
- `src/sksl/analysis/SkSLSpecialization.h` — port to Rust.
- `src/sksl/analysis/SkSLSwitchCaseContainsExit.cpp` — port to Rust.
- `src/sksl/analysis/SkSLSymbolTableStackBuilder.cpp` — port to Rust.
- `src/sksl/generated/sksl_compute.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_compute.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_frag.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_frag.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_gpu.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_gpu.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_graphite_frag.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_graphite_frag.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_graphite_vert.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_graphite_vert.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_public.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_public.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_rt_shader.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_rt_shader.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_shared.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_shared.unoptimized.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_vert.minified.sksl` — preserve/generate input.
- `src/sksl/generated/sksl_vert.unoptimized.sksl` — preserve/generate input.
- `src/sksl/ir/SkSLBinaryExpression.cpp` — port to Rust.
- `src/sksl/ir/SkSLBinaryExpression.h` — port to Rust.
- `src/sksl/ir/SkSLBlock.cpp` — port to Rust.
- `src/sksl/ir/SkSLBlock.h` — port to Rust.
- `src/sksl/ir/SkSLBreakStatement.h` — port to Rust.
- `src/sksl/ir/SkSLChildCall.cpp` — port to Rust.
- `src/sksl/ir/SkSLChildCall.h` — port to Rust.
- `src/sksl/ir/SkSLConstructor.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructor.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorArray.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorArray.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorArrayCast.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorArrayCast.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorCompound.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorCompound.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorCompoundCast.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorCompoundCast.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorDiagonalMatrix.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorDiagonalMatrix.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorMatrixResize.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorMatrixResize.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorScalarCast.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorScalarCast.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorSplat.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorSplat.h` — port to Rust.
- `src/sksl/ir/SkSLConstructorStruct.cpp` — port to Rust.
- `src/sksl/ir/SkSLConstructorStruct.h` — port to Rust.
- `src/sksl/ir/SkSLContinueStatement.h` — port to Rust.
- `src/sksl/ir/SkSLDiscardStatement.cpp` — port to Rust.
- `src/sksl/ir/SkSLDiscardStatement.h` — port to Rust.
- `src/sksl/ir/SkSLDoStatement.cpp` — port to Rust.
- `src/sksl/ir/SkSLDoStatement.h` — port to Rust.
- `src/sksl/ir/SkSLEmptyExpression.h` — port to Rust.
- `src/sksl/ir/SkSLExpression.cpp` — port to Rust.
- `src/sksl/ir/SkSLExpression.h` — port to Rust.
- `src/sksl/ir/SkSLExpressionStatement.cpp` — port to Rust.
- `src/sksl/ir/SkSLExpressionStatement.h` — port to Rust.
- `src/sksl/ir/SkSLExtension.cpp` — port to Rust.
- `src/sksl/ir/SkSLExtension.h` — port to Rust.
- `src/sksl/ir/SkSLFieldAccess.cpp` — port to Rust.
- `src/sksl/ir/SkSLFieldAccess.h` — port to Rust.
- `src/sksl/ir/SkSLFieldSymbol.h` — port to Rust.
- `src/sksl/ir/SkSLForStatement.cpp` — port to Rust.
- `src/sksl/ir/SkSLForStatement.h` — port to Rust.
- `src/sksl/ir/SkSLFunctionCall.cpp` — port to Rust.
- `src/sksl/ir/SkSLFunctionCall.h` — port to Rust.
- `src/sksl/ir/SkSLFunctionDeclaration.cpp` — port to Rust.
- `src/sksl/ir/SkSLFunctionDeclaration.h` — port to Rust.
- `src/sksl/ir/SkSLFunctionDefinition.cpp` — port to Rust.
- `src/sksl/ir/SkSLFunctionDefinition.h` — port to Rust.
- `src/sksl/ir/SkSLFunctionPrototype.h` — port to Rust.
- `src/sksl/ir/SkSLFunctionReference.h` — port to Rust.
- `src/sksl/ir/SkSLIRHelpers.h` — port to Rust.
- `src/sksl/ir/SkSLIRNode.h` — port to Rust.
- `src/sksl/ir/SkSLIfStatement.cpp` — port to Rust.
- `src/sksl/ir/SkSLIfStatement.h` — port to Rust.
- `src/sksl/ir/SkSLIndexExpression.cpp` — port to Rust.
- `src/sksl/ir/SkSLIndexExpression.h` — port to Rust.
- `src/sksl/ir/SkSLInterfaceBlock.cpp` — port to Rust.
- `src/sksl/ir/SkSLInterfaceBlock.h` — port to Rust.
- `src/sksl/ir/SkSLLayout.cpp` — port to Rust.
- `src/sksl/ir/SkSLLayout.h` — port to Rust.
- `src/sksl/ir/SkSLLiteral.cpp` — port to Rust.
- `src/sksl/ir/SkSLLiteral.h` — port to Rust.
- `src/sksl/ir/SkSLMethodReference.h` — port to Rust.
- `src/sksl/ir/SkSLModifierFlags.cpp` — port to Rust.
- `src/sksl/ir/SkSLModifierFlags.h` — port to Rust.
- `src/sksl/ir/SkSLModifiers.h` — port to Rust.
- `src/sksl/ir/SkSLModifiersDeclaration.cpp` — port to Rust.
- `src/sksl/ir/SkSLModifiersDeclaration.h` — port to Rust.
- `src/sksl/ir/SkSLNop.h` — port to Rust.
- `src/sksl/ir/SkSLPoison.h` — port to Rust.
- `src/sksl/ir/SkSLPostfixExpression.cpp` — port to Rust.
- `src/sksl/ir/SkSLPostfixExpression.h` — port to Rust.
- `src/sksl/ir/SkSLPrefixExpression.cpp` — port to Rust.
- `src/sksl/ir/SkSLPrefixExpression.h` — port to Rust.
- `src/sksl/ir/SkSLProgram.cpp` — port to Rust.
- `src/sksl/ir/SkSLProgram.h` — port to Rust.
- `src/sksl/ir/SkSLProgramElement.h` — port to Rust.
- `src/sksl/ir/SkSLReturnStatement.h` — port to Rust.
- `src/sksl/ir/SkSLSetting.cpp` — port to Rust.
- `src/sksl/ir/SkSLSetting.h` — port to Rust.
- `src/sksl/ir/SkSLStatement.h` — port to Rust.
- `src/sksl/ir/SkSLStructDefinition.cpp` — port to Rust.
- `src/sksl/ir/SkSLStructDefinition.h` — port to Rust.
- `src/sksl/ir/SkSLSwitchCase.cpp` — port to Rust.
- `src/sksl/ir/SkSLSwitchCase.h` — port to Rust.
- `src/sksl/ir/SkSLSwitchStatement.cpp` — port to Rust.
- `src/sksl/ir/SkSLSwitchStatement.h` — port to Rust.
- `src/sksl/ir/SkSLSwizzle.cpp` — port to Rust.
- `src/sksl/ir/SkSLSwizzle.h` — port to Rust.
- `src/sksl/ir/SkSLSymbol.cpp` — port to Rust.
- `src/sksl/ir/SkSLSymbol.h` — port to Rust.
- `src/sksl/ir/SkSLSymbolTable.cpp` — port to Rust.
- `src/sksl/ir/SkSLSymbolTable.h` — port to Rust.
- `src/sksl/ir/SkSLTernaryExpression.cpp` — port to Rust.
- `src/sksl/ir/SkSLTernaryExpression.h` — port to Rust.
- `src/sksl/ir/SkSLType.cpp` — port to Rust.
- `src/sksl/ir/SkSLType.h` — port to Rust.
- `src/sksl/ir/SkSLTypeReference.cpp` — port to Rust.
- `src/sksl/ir/SkSLTypeReference.h` — port to Rust.
- `src/sksl/ir/SkSLVarDeclarations.cpp` — port to Rust.
- `src/sksl/ir/SkSLVarDeclarations.h` — port to Rust.
- `src/sksl/ir/SkSLVariable.cpp` — port to Rust.
- `src/sksl/ir/SkSLVariable.h` — port to Rust.
- `src/sksl/ir/SkSLVariableReference.cpp` — port to Rust.
- `src/sksl/ir/SkSLVariableReference.h` — port to Rust.
- `src/sksl/lex/DFA.h` — port to Rust.
- `src/sksl/lex/DFAState.h` — port to Rust.
- `src/sksl/lex/LexUtil.h` — port to Rust.
- `src/sksl/lex/Main.cpp` — port to Rust.
- `src/sksl/lex/NFA.cpp` — port to Rust.
- `src/sksl/lex/NFA.h` — port to Rust.
- `src/sksl/lex/NFAState.h` — port to Rust.
- `src/sksl/lex/NFAtoDFA.h` — port to Rust.
- `src/sksl/lex/RegexNode.cpp` — port to Rust.
- `src/sksl/lex/RegexNode.h` — port to Rust.
- `src/sksl/lex/RegexParser.cpp` — port to Rust.
- `src/sksl/lex/RegexParser.h` — port to Rust.
- `src/sksl/lex/TransitionTable.cpp` — port to Rust.
- `src/sksl/lex/TransitionTable.h` — port to Rust.
- `src/sksl/lex/sksl.lex` — preserve/generate input.
- `src/sksl/sksl_compute.sksl` — preserve/generate input.
- `src/sksl/sksl_frag.sksl` — preserve/generate input.
- `src/sksl/sksl_gpu.sksl` — preserve/generate input.
- `src/sksl/sksl_graphite_frag.sksl` — preserve/generate input.
- `src/sksl/sksl_graphite_vert.sksl` — preserve/generate input.
- `src/sksl/sksl_public.sksl` — preserve/generate input.
- `src/sksl/sksl_rt_shader.sksl` — preserve/generate input.
- `src/sksl/sksl_shared.sksl` — preserve/generate input.
- `src/sksl/sksl_vert.sksl` — preserve/generate input.
- `src/sksl/spirv.h` — port to Rust.
- `src/sksl/tracing/SkSLDebugTracePlayer.cpp` — port to Rust.
- `src/sksl/tracing/SkSLDebugTracePlayer.h` — port to Rust.
- `src/sksl/tracing/SkSLDebugTracePriv.cpp` — port to Rust.
- `src/sksl/tracing/SkSLDebugTracePriv.h` — port to Rust.
- `src/sksl/tracing/SkSLTraceHook.cpp` — port to Rust.
- `src/sksl/tracing/SkSLTraceHook.h` — port to Rust.
- `src/sksl/transform/SkSLAddConstToVarModifiers.cpp` — port to Rust.
- `src/sksl/transform/SkSLEliminateDeadFunctions.cpp` — port to Rust.
- `src/sksl/transform/SkSLEliminateDeadGlobalVariables.cpp` — port to Rust.
- `src/sksl/transform/SkSLEliminateDeadLocalVariables.cpp` — port to Rust.
- `src/sksl/transform/SkSLEliminateEmptyStatements.cpp` — port to Rust.
- `src/sksl/transform/SkSLEliminateUnnecessaryBraces.cpp` — port to Rust.
- `src/sksl/transform/SkSLEliminateUnreachableCode.cpp` — port to Rust.
- `src/sksl/transform/SkSLFindAndDeclareBuiltinFunctions.cpp` — port to Rust.
- `src/sksl/transform/SkSLFindAndDeclareBuiltinStructs.cpp` — port to Rust.
- `src/sksl/transform/SkSLFindAndDeclareBuiltinVariables.cpp` — port to Rust.
- `src/sksl/transform/SkSLHoistSwitchVarDeclarationsAtTopLevel.cpp` — port to Rust.
- `src/sksl/transform/SkSLProgramWriter.h` — port to Rust.
- `src/sksl/transform/SkSLRenamePrivateSymbols.cpp` — port to Rust.
- `src/sksl/transform/SkSLReplaceConstVarsWithLiterals.cpp` — port to Rust.
- `src/sksl/transform/SkSLReplaceSplatCastsWithSwizzles.cpp` — port to Rust.
- `src/sksl/transform/SkSLRewriteIndexedSwizzle.cpp` — port to Rust.
- `src/sksl/transform/SkSLTransform.cpp` — port to Rust.
- `src/sksl/transform/SkSLTransform.h` — port to Rust.

### Adapt these build/tooling files (Deepseek)

- `include/sksl/BUILD.bazel`.
- `src/sksl/BUILD.bazel`.
- `src/sksl/analysis/BUILD.bazel`.
- `src/sksl/ir/BUILD.bazel`.
- `src/sksl/lex/BUILD.bazel`.
- `src/sksl/tracing/BUILD.bazel`.
- `src/sksl/transform/BUILD.bazel`.

## Codex: test-writing contract

Codex writes the tests and minimum required test support only, including supplemental checks in the initial pass, then stops. Deepseek subsequently executes all tests and references, performs discovery and coverage audits, records evidence, fixes production defects and updates acceptance/status. Execution, hashing, resource verification and acceptance instructions below belong to Deepseek; Codex does not run them. Follow `../NEXT-STEPS.md` for the three prompts.

Translate frontend/type/error/layout tests and relevant shader fixtures. Preserve diagnostic text and source locations where asserted. All fixture cases must have IDs; generated or macro-expanded tests require discovery from the built reference runner, not only a regex count.

- Translate the original reachable test bodies and helpers, preserving inputs, order, seeds, assertions, tolerances, expected failures and conditional skips. Retain original names and source locations in the correspondence record.
- Discover compiled C++ test registrations under the matching macOS feature configuration. The registration CSV is an initial source scan, not a substitute for expanded macro/test discovery. Resolve conditional/custom suites before accepting this milestone.
- For files split across milestones, translate only the assigned registrations now, including their helper closure; record the rest under their later gate. Do not mark a whole file complete merely because its first tests pass.
- Port original test utilities needed by these cases, even if their general tool packaging has a later owner. Keep fixture-only implementations in test support and never substitute them for production APIs being checked.
- Where upstream provides no direct unit test, add clearly labeled supplemental contract/differential tests, separate from faithful translations. Run C++ as an external reference where needed; do not link it as the Rust implementation.
- Preserve resources and shader/golden inputs by hash. Resource metadata and required downloads must be recorded before running; absent inputs or backend hardware are BLOCKED, not a skip that counts as PASS.

### Port these test/runner/support files (Codex)

- `tests/SkRasterPipelineTest.cpp`.
- `tests/SkSLErrorTest.cpp`.
- `tests/SkSLMemoryLayoutTest.cpp`.
- `tests/SkSLTypeTest.cpp`.

### Original test registrations assigned to this gate

- `SkRasterPipeline_TraceVar` — `tests/SkRasterPipelineTest.cpp:908`; CPU; `DEF_TEST`.
- `SkRasterPipeline_TraceLine` — `tests/SkRasterPipelineTest.cpp:1021`; CPU; `DEF_TEST`.
- `SkRasterPipeline_TraceEnterExit` — `tests/SkRasterPipelineTest.cpp:1067`; CPU; `DEF_TEST`.
- `SkRasterPipeline_TraceScope` — `tests/SkRasterPipelineTest.cpp:1119`; CPU; `DEF_TEST`.
- `SkSLErrorTest` — `tests/SkSLErrorTest.cpp:177`; CPU; `DEF_TEST`.
- `SkSLComputeErrorTest` — `tests/SkSLErrorTest.cpp:183`; CPU; `DEF_TEST`.
- `SkSLMeshVertexErrorTest` — `tests/SkSLErrorTest.cpp:189`; CPU; `DEF_TEST`.
- `SkSLMeshFragmentErrorTest` — `tests/SkSLErrorTest.cpp:195`; CPU; `DEF_TEST`.
- `SkSLRuntimeShaderErrorTest` — `tests/SkSLErrorTest.cpp:201`; CPU; `DEF_TEST`.
- `SkSLPrivateRuntimeShaderErrorTest` — `tests/SkSLErrorTest.cpp:210`; CPU; `DEF_TEST`.
- `SkSLRuntimeColorFilterErrorTest` — `tests/SkSLErrorTest.cpp:219`; CPU; `DEF_TEST`.
- `SkSLRuntimeBlenderErrorTest` — `tests/SkSLErrorTest.cpp:225`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_std140` — `tests/SkSLMemoryLayoutTest.cpp:28`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_std430` — `tests/SkSLMemoryLayoutTest.cpp:127`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_WGSLUniform_Base` — `tests/SkSLMemoryLayoutTest.cpp:226`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_WGSLUniform_EnableF16` — `tests/SkSLMemoryLayoutTest.cpp:518`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_WGSLStorage_Base` — `tests/SkSLMemoryLayoutTest.cpp:810`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_WGSLStorage_EnableF16` — `tests/SkSLMemoryLayoutTest.cpp:1100`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_WGSLUnsupportedTypes` — `tests/SkSLMemoryLayoutTest.cpp:1390`; CPU; `DEF_TEST`.
- `SkSLMemoryLayoutTest_WGSLSupportedTypes` — `tests/SkSLMemoryLayoutTest.cpp:1417`; CPU; `DEF_TEST`.
- `SkSLTypeLimits` — `tests/SkSLTypeTest.cpp:16`; CPU; `DEF_TEST`.

### Later integration obligations for these APIs

The following original test files directly reference production headers owned here but contain cases scheduled later. They remain mandatory at the listed gates; they are not excluded or already passed. Keep earlier milestone acceptance limited to the tests/feature contract above.

- `tests/GrGpuBufferTest.cpp` — M38.
- `tests/GrMeshTest.cpp` — M38.
- `tests/GrPipelineDynamicStateTest.cpp` — M38.
- `tests/PrimitiveProcessorTest.cpp` — M38.
- `tests/ProcessorTest.cpp` — M34, M38.
- `tests/RasterPipelineBuilderTest.cpp` — M25.
- `tests/RasterPipelineCodeGeneratorTest.cpp` — M25.
- `tests/SkRasterPipelineTest.cpp` — M31.
- `tests/SkRuntimeEffectTest.cpp` — M35, M38.
- `tests/SkSLCross.cpp` — M38.
- `tests/SkSLDebugTracePlayerTest.cpp` — M26.
- `tests/SkSLDebugTraceTest.cpp` — M26.
- `tests/SkSLGLSLTestbed.cpp` — M25.
- `tests/SkSLMetalTestbed.cpp` — M25.
- `tests/SkSLPipelineStageTestbed.cpp` — M25.
- `tests/SkSLSPIRVTestbed.cpp` — M25.
- `tests/SkSLTest.cpp` — M37, M38.
- `tests/SkSLWGSLTestbed.cpp` — M25.
- `tests/graphite/UniformManagerTest.cpp` — M35.
- `tests/graphite/UniformOffsetCalculatorTest.cpp` — M35.

### Preserve these fixtures, metadata and references

- `src/sksl/README.md` — preserve.

## Dependency and integration notes

The initial scan found 0 quoted includes from this milestone into later-owned production files. See `../inventory/forward-includes.csv`, filtered by `source_milestone=24`. Header inclusion alone does not force implementation of the entire later subsystem; inspect the original registered test bodies and called production paths.

Interfaces with later consumers may be declared now, but the accepted feature set must execute real implementations for every reached call. Where a source spans several features (CPU/GPU, serialization, runtime shaders), record completed and pending symbols separately. Deferred portions have explicit integration gates M14, M22, M26, M38 and M47; no pending portion may disappear from the ledger.

## Commands, required evidence and stop condition

Deepseek runs the milestone tests and all previously accepted tests in debug and release with the exact feature list from the handoff. The standard baseline commands are:

```sh
cargo test --locked
cargo test --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
```

For feature-specific crates/backends, record the additional exact package/target/features commands; do not use an unexamined `--all-features` combination. Native backend, GM, browser, fuzz and benchmark commands are added when their harnesses exist and must be copied into the acceptance report.

Acceptance requires: all assigned macOS cases enumerated; every applicable assertion passing; all earlier accepted tests passing; every source row mapped to implemented Rust or a documented non-code disposition; no unexplained missing resources, untested required backends or unsupported required API; and a Deepseek-authored evidence report. Formatting/build success alone never constitutes acceptance.

Record PASS, FAIL, BLOCKED or NOT_APPLICABLE per test/configuration. NOT_APPLICABLE requires an original platform/capability condition and a reason. All milestones require Apple Silicon macOS evidence only. Intel/x86_64 implementation, builds and execution are excluded by user direction, including M46/M47. Intel-specific inventory paths are reference accounting only, not pending work or acceptance blockers.

Only after Deepseek runs the tests, completes the audit and marks M24 ACCEPTED may the user hand Deepseek M25.
