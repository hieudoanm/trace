import type { Metadata, NextPage } from 'next';
import type { FC } from 'react';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuActivity,
  LuBot,
  LuBoxes,
  LuCrosshair,
  LuGlobe,
  LuLink,
  LuZap,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/tracers';

const FEATURES: Feature[] = [
  {
    icon: LuGlobe,
    title: '20+ languages',
    body: 'TypeScript, Python, Go, Rust, Kotlin, Swift, Java, C#, PHP, Ruby, C, C++, Dart, Scala, R, Zig, and more — all through maintained Tree-sitter grammars.',
  },
  {
    icon: LuLink,
    title: 'Honest resolution',
    body: 'Every relationship is marked resolved_internal, resolved_external, unresolved, or ambiguous — tracers never pretends a guess is a fact.',
  },
  {
    icon: LuActivity,
    title: 'Call & dependency graphs',
    body: 'Direct and transitive dependencies, callers and callees, references, inheritance — bounded, deterministic traversals over indexed graph data.',
  },
  {
    icon: LuCrosshair,
    title: 'Impact analysis',
    body: 'See exactly which symbols depend on a target, with the path that connects them — static reachability, not runtime guesswork.',
  },
  {
    icon: LuBoxes,
    title: 'Interactive explorer',
    body: 'Every scan emits a standalone graph.html — a D3-force canvas explorer with search, filtering, and ego-network inspection. Works from file://.',
  },
  {
    icon: LuZap,
    title: 'Fast & incremental',
    body: 'Parallel parsing with per-worker parsers and a content-addressed syntax cache — rescans only re-parse what changed.',
  },
  {
    icon: LuBot,
    title: 'MCP server',
    body: 'tracers mcp exposes the whole graph — callers, callees, deps, impact, cycles — to MCP clients like Claude Desktop and Cursor over stdio.',
  },
];

const HERO_LINES = [
  '$ tracers scan .',
  '=== Tracers scan summary ===',
  'Files scanned: 4',
  'Nodes: 59',
  'Edges: 142',
  'Wrote graph export to .tracers',
];

const DEMO_TERMINALS: {
  caption: string;
  lines: string[];
  wide?: boolean;
}[] = [
  {
    caption: 'Scan a repository',
    wide: true,
    lines: [
      '$ tracers scan fixtures/languages/typescript',
      '=== Tracers scan summary ===',
      'Files scanned: 4',
      'Nodes: 59',
      'Edges: 142',
      'Symbols: 44',
      'Unresolved edges: 12',
      'Wrote graph export to .tracers',
    ],
  },
  {
    caption: 'Who calls this function?',
    lines: [
      '$ tracers callers twice',
      'Callers of `twice`:',
      '  - run [function] via calls',
      '    (resolved_internal), lexical',
      '    depth 1 — src/math.ts:11',
    ],
  },
  {
    caption: 'What depends on it?',
    lines: [
      '$ tracers impact twice',
      'Impact of `twice`:',
      '  - run [function] depth 1',
      '    path: run -> twice',
      '    via: calls',
    ],
  },
];

const STEPS = [
  {
    title: 'Scan',
    body: 'tracers discovers source files (respecting .gitignore and your tracers.toml), parses them with Tree-sitter in parallel, and caches syntax trees by content hash.',
    snippet: 'tracers scan .',
  },
  {
    title: 'Resolve & build',
    body: 'A layered resolver matches symbols through lexical scope, imports, qualified names, and package boundaries — recording exactly how confident each edge is.',
    snippet: '.tracers/graph.json',
  },
  {
    title: 'Query & explore',
    body: 'Ask callers, callees, refs, deps, dependents, reach, impact, path, roots, cycles — or open the interactive explorer. All queries accept --graph to skip rescanning.',
    snippet: 'tracers callers myFunc',
  },
];

const INSTALL_LINES = [
  '$ curl -fsSL https://raw.githubusercontent.com/hieudoanm/tracers/main/scripts/install.sh | sh',
  'Installed tracers to ~/.local/bin/tracers',
  '$ tracers scan .',
  'Wrote graph export to .tracers',
  '$ tracers --version',
  'tracers 0.0.1 (build 2026.11.08.13.12.56)',
];

const StepCard: FC<{ step: (typeof STEPS)[number]; index: number }> = ({
  step,
  index,
}) => (
  <article className="card border-base-300 bg-base-100 rounded-2xl border p-6">
    <span className="text-accent mb-2 block font-mono text-sm font-semibold">
      {String(index + 1).padStart(2, '0')}
    </span>
    <h3 className="text-base-content m-0 mb-2 text-lg font-bold">
      {step.title}
    </h3>
    <p className="text-base-content/60 text-sm">{step.body}</p>
    <code className="border-base-300 bg-base-200 text-base-content/60 mt-4 block overflow-x-auto rounded-lg border px-3 py-2 font-mono text-xs">
      {step.snippet}
    </code>
  </article>
);

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="See your codebase as a graph."
      description="tracers parses 20+ languages with Tree-sitter and builds a unified graph of symbols, calls, imports, and dependencies. Ask who calls a function, what breaks if it changes, or where the cycles are — all locally, in one Rust binary."
      primary={{ label: 'Get started', href: '#install' }}
      secondary={{ label: 'See it in action', href: '#demo' }}
      stats={[
        { value: '20+', label: 'languages parsed' },
        { value: '0', label: 'cloud calls' },
        { value: 'deterministic', label: 'reproducible exports' },
      ]}>
      <Terminal lines={HERO_LINES} />
    </Hero>

    <Features
      title="Features"
      description="A layered analysis pipeline that turns source code into a queryable graph — and is honest about what it cannot resolve."
      features={FEATURES}
    />

    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="See it in action"
          description={
            <>
              One scan builds the graph; every query command runs against it —
              or against an exported <code>graph.json</code> without rescanning.
            </>
          }
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal
              key={t.caption}
              lines={t.lines}
              className={t.wide ? 'lg:col-span-2' : ''}
            />
          ))}
        </div>
      </div>
    </section>

    <section id="how-it-works" className="py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="How it works"
          description="Discover, parse, resolve, query — a deterministic pipeline with explicit uncertainty at every step."
        />
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {STEPS.map((step, i) => (
            <StepCard key={step.title} step={step} index={i} />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-3xl px-5">
        <SectionHead
          title="Get started"
          description="Grab a prebuilt binary from GitHub Releases, or build from source with Cargo. No runtime dependencies, no daemon, no account."
        />
        <Terminal lines={INSTALL_LINES} />
        <p className="text-base-content/60 mt-6 text-center text-sm">
          Or build from source:{' '}
          <code className="bg-base-300 rounded px-1.5 py-0.5 font-mono text-xs">
            ./scripts/install-local.sh
          </code>{' '}
          — see the{' '}
          <a
            href={`${GITHUB}/blob/master/README.md`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            README
          </a>{' '}
          for details.
        </p>
      </div>
    </section>

    <CTA
      title="Ready to map your codebase?"
      description="One scan, one binary — see every call, import, and dependency."
      primary={{ label: 'View on GitHub', href: GITHUB, external: true }}
      secondary={{
        label: 'Read the README',
        href: `${GITHUB}#readme`,
        external: true,
      }}
    />
  </main>
);

export default HomePage;
