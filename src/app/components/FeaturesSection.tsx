import { motion } from 'motion/react';
import { GitBranch, CheckCircle, Sparkles, Cloud } from 'lucide-react';
import { useState } from 'react';

export function FeaturesSection() {
  const [showJson, setShowJson] = useState(true);

  return (
    <section className="relative py-32 px-4 md:px-8">
      <div className="max-w-7xl mx-auto">
        {/* Section Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center mb-16 space-y-4"
        >
          <h2 
            className="text-4xl md:text-5xl"
            style={{ 
              fontFamily: 'Inter Tight, sans-serif',
              letterSpacing: '-0.02em'
            }}
          >
            Built for <span className="text-[#00F5FF]">Production</span>
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto" style={{ fontFamily: 'Inter, sans-serif' }}>
            Enterprise-grade capabilities delivered through a modern, composable platform
          </p>
        </motion.div>

        {/* Bento Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {/* Box 1 - Multi-Agent Orchestration */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="lg:col-span-2 bg-[#1A1A1E]/40 backdrop-blur-sm border border-white/10 rounded-2xl p-8 hover:border-[#00F5FF]/30 transition-colors"
          >
            <div className="flex items-start gap-4 mb-6">
              <div className="h-12 w-12 rounded-xl bg-[#00F5FF]/20 flex items-center justify-center flex-shrink-0">
                <GitBranch className="h-6 w-6 text-[#00F5FF]" />
              </div>
              <div>
                <h3 className="mb-2" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Multi-Agent Orchestration
                </h3>
                <p className="text-gray-400" style={{ fontFamily: 'Inter, sans-serif' }}>
                  Coordinate multiple specialized agents with intelligent handoffs and context sharing
                </p>
              </div>
            </div>

            {/* Timeline visualization */}
            <div className="space-y-3">
              {[
                { agent: 'Triage Agent', action: 'Incident analyzed', status: 'complete', delay: 0 },
                { agent: 'SRE Agent', action: 'Mitigation deployed', status: 'complete', delay: 0.2 },
                { agent: 'GAC Agent', action: 'Compliance verified', status: 'active', delay: 0.4 },
              ].map((step, i) => (
                <motion.div
                  key={i}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: step.delay }}
                  className="flex items-center gap-4 p-3 bg-[#0A0A0B]/40 rounded-lg border border-white/5"
                >
                  <div className={`h-2 w-2 rounded-full ${step.status === 'complete' ? 'bg-[#D1FF26]' : 'bg-[#00F5FF] animate-pulse'}`} />
                  <div className="flex-1">
                    <p className="text-sm" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                      {step.agent}
                    </p>
                    <p className="text-xs text-gray-500">{step.action}</p>
                  </div>
                  <span className="text-xs text-gray-600" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                    {i + 1}50ms
                  </span>
                </motion.div>
              ))}
            </div>
          </motion.div>

          {/* Box 2 - Quality Gates */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="bg-[#1A1A1E]/40 backdrop-blur-sm border border-white/10 rounded-2xl p-8 hover:border-[#00F5FF]/30 transition-colors"
          >
            <div className="flex items-start gap-4 mb-6">
              <div className="h-12 w-12 rounded-xl bg-[#D1FF26]/20 flex items-center justify-center flex-shrink-0">
                <CheckCircle className="h-6 w-6 text-[#D1FF26]" />
              </div>
              <div>
                <h3 className="mb-2" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Quality Gates
                </h3>
                <p className="text-sm text-gray-400" style={{ fontFamily: 'Inter, sans-serif' }}>
                  Configurable validation at every step
                </p>
              </div>
            </div>

            <div className="space-y-4">
              {['Minimal', 'Standard', 'Rigorous'].map((level, i) => (
                <motion.div
                  key={level}
                  initial={{ opacity: 0, scale: 0.9 }}
                  whileInView={{ opacity: 1, scale: 1 }}
                  viewport={{ once: true }}
                  transition={{ delay: 0.3 + i * 0.1 }}
                  className="flex items-center justify-between p-3 bg-[#0A0A0B]/40 rounded-lg border border-white/5"
                >
                  <span className="text-sm" style={{ fontFamily: 'Inter, sans-serif' }}>{level}</span>
                  <div className="flex items-center gap-2">
                    <div className="h-1.5 w-16 bg-[#1A1A1E] rounded-full overflow-hidden">
                      <motion.div
                        initial={{ width: 0 }}
                        whileInView={{ width: `${(i + 1) * 33}%` }}
                        viewport={{ once: true }}
                        transition={{ delay: 0.5 + i * 0.1, duration: 0.5 }}
                        className="h-full bg-[#D1FF26]"
                      />
                    </div>
                    <CheckCircle className="h-4 w-4 text-[#D1FF26]" />
                  </div>
                </motion.div>
              ))}
            </div>
          </motion.div>

          {/* Box 3 - Generative UI */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.3 }}
            className="lg:col-span-2 bg-[#1A1A1E]/40 backdrop-blur-sm border border-white/10 rounded-2xl p-8 hover:border-[#00F5FF]/30 transition-colors"
          >
            <div className="flex items-start justify-between mb-6">
              <div className="flex items-start gap-4">
                <div className="h-12 w-12 rounded-xl bg-[#8B5CF6]/20 flex items-center justify-center flex-shrink-0">
                  <Sparkles className="h-6 w-6 text-[#8B5CF6]" />
                </div>
                <div>
                  <h3 className="mb-2" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                    Generative UI
                  </h3>
                  <p className="text-gray-400" style={{ fontFamily: 'Inter, sans-serif' }}>
                    Transform agent outputs into beautiful, interactive components
                  </p>
                </div>
              </div>
              <button
                onClick={() => setShowJson(!showJson)}
                className="px-3 py-1 bg-[#00F5FF]/10 text-[#00F5FF] rounded-lg text-xs hover:bg-[#00F5FF]/20 transition-colors"
                style={{ fontFamily: 'Inter, sans-serif' }}
              >
                {showJson ? 'Show UI' : 'Show JSON'}
              </button>
            </div>

            <div className="grid md:grid-cols-2 gap-4">
              {/* Before - JSON */}
              <div className={`space-y-2 ${!showJson ? 'opacity-30' : ''}`}>
                <p className="text-xs text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>Raw Output</p>
                <div className="bg-[#0A0A0B] rounded-lg p-4 border border-white/5">
                  <pre className="text-xs text-[#00F5FF] overflow-x-auto" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
{`{
  "severity": "high",
  "type": "outage",
  "affected": 1247,
  "status": "mitigated"
}`}
                  </pre>
                </div>
              </div>

              {/* After - UI Component */}
              <div className={`space-y-2 ${showJson ? 'opacity-30' : ''}`}>
                <p className="text-xs text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>Generated UI</p>
                <motion.div
                  animate={{ scale: showJson ? 0.95 : 1 }}
                  className="bg-gradient-to-br from-[#D1FF26]/10 to-[#00F5FF]/10 rounded-lg p-4 border border-[#D1FF26]/20"
                >
                  <div className="flex items-center gap-2 mb-3">
                    <div className="h-2 w-2 rounded-full bg-[#D1FF26]" />
                    <span className="text-xs text-[#D1FF26]" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                      MITIGATED
                    </span>
                  </div>
                  <h4 className="mb-1" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                    Service Outage
                  </h4>
                  <p className="text-sm text-gray-400" style={{ fontFamily: 'Inter, sans-serif' }}>
                    1,247 users affected
                  </p>
                </motion.div>
              </div>
            </div>
          </motion.div>

          {/* Box 4 - Multi-Cloud */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.4 }}
            className="bg-[#1A1A1E]/40 backdrop-blur-sm border border-white/10 rounded-2xl p-8 hover:border-[#00F5FF]/30 transition-colors"
          >
            <div className="flex items-start gap-4 mb-6">
              <div className="h-12 w-12 rounded-xl bg-[#00F5FF]/20 flex items-center justify-center flex-shrink-0">
                <Cloud className="h-6 w-6 text-[#00F5FF]" />
              </div>
              <div>
                <h3 className="mb-2" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Multi-Cloud Ready
                </h3>
                <p className="text-sm text-gray-400" style={{ fontFamily: 'Inter, sans-serif' }}>
                  Deploy anywhere, manage from one place
                </p>
              </div>
            </div>

            <div className="space-y-3">
              {[
                { name: 'AWS EKS', status: 'active' },
                { name: 'GCP Cloud Run', status: 'active' },
                { name: 'Azure AKS', status: 'standby' },
              ].map((cloud, i) => (
                <motion.div
                  key={cloud.name}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: 0.5 + i * 0.1 }}
                  className="flex items-center justify-between p-3 bg-[#0A0A0B]/40 rounded-lg border border-white/5"
                >
                  <span className="text-sm" style={{ fontFamily: 'Inter, sans-serif' }}>{cloud.name}</span>
                  <span 
                    className={`text-xs px-2 py-1 rounded ${cloud.status === 'active' ? 'bg-[#D1FF26]/20 text-[#D1FF26]' : 'bg-gray-500/20 text-gray-500'}`}
                    style={{ fontFamily: 'JetBrains Mono, monospace' }}
                  >
                    {cloud.status}
                  </span>
                </motion.div>
              ))}
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
