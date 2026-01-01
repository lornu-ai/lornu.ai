import { motion } from 'motion/react';
import { useState } from 'react';
import { Terminal, Activity, ChevronRight, Brain } from 'lucide-react';

export function DashboardPreview() {
  const [evidenceOpen, setEvidenceOpen] = useState(false);

  const logs = [
    { time: '14:32:01', agent: 'Orchestrator', message: 'Analyzing incident context...', type: 'info' },
    { time: '14:32:02', agent: 'Triage', message: 'Severity: HIGH | Affected users: 1,247', type: 'warning' },
    { time: '14:32:03', agent: 'SRE', message: 'Deploying mitigation strategy...', type: 'info' },
    { time: '14:32:05', agent: 'SRE', message: 'Service restored | Latency normalized', type: 'success' },
    { time: '14:32:06', agent: 'GAC', message: 'Compliance check: PASSED', type: 'success' },
  ];

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
            The <span className="text-[#00F5FF]">Agent HUD</span>
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto" style={{ fontFamily: 'Inter, sans-serif' }}>
            Real-time visibility into agent reasoning and decision-making
          </p>
        </motion.div>

        {/* Dashboard Preview */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="relative"
        >
          {/* Main Dashboard Container */}
          <div className="bg-[#1A1A1E]/60 backdrop-blur-xl border border-white/10 rounded-2xl overflow-hidden shadow-2xl">
            {/* Dashboard Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-[#0A0A0B]/40">
              <div className="flex items-center gap-3">
                <Activity className="h-5 w-5 text-[#00F5FF]" />
                <h3 style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Agent Dashboard
                </h3>
              </div>
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <div className="h-2 w-2 rounded-full bg-[#D1FF26] animate-pulse" />
                  <span className="text-xs text-gray-400" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                    3 agents active
                  </span>
                </div>
                <button
                  onClick={() => setEvidenceOpen(!evidenceOpen)}
                  className="flex items-center gap-2 px-3 py-1.5 bg-[#00F5FF]/10 hover:bg-[#00F5FF]/20 text-[#00F5FF] rounded-lg text-sm transition-colors"
                  style={{ fontFamily: 'Inter, sans-serif' }}
                >
                  <Terminal className="h-4 w-4" />
                  Evidence Panel
                  <ChevronRight className={`h-4 w-4 transition-transform ${evidenceOpen ? 'rotate-90' : ''}`} />
                </button>
              </div>
            </div>

            {/* Main Content Area */}
            <div className="grid lg:grid-cols-3 divide-x divide-white/5">
              {/* Agent Status Cards */}
              <div className="lg:col-span-2 p-6 space-y-4">
                <h4 className="text-sm text-gray-500 uppercase tracking-wide mb-4" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Active Agents
                </h4>
                
                {[
                  { name: 'Orchestrator Agent', status: 'coordinating', activity: 'Routing to SRE', color: '#00F5FF' },
                  { name: 'SRE Agent', status: 'executing', activity: 'Deploying fix to prod-us-east-1', color: '#D1FF26' },
                  { name: 'GAC Agent', status: 'monitoring', activity: 'Compliance validation', color: '#8B5CF6' }
                ].map((agent, i) => (
                  <motion.div
                    key={agent.name}
                    initial={{ opacity: 0, x: -20 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{ delay: i * 0.1 }}
                    className="flex items-center justify-between p-4 bg-[#0A0A0B]/60 rounded-xl border border-white/5 hover:border-white/10 transition-colors"
                  >
                    <div className="flex items-center gap-4">
                      <div className="relative">
                        <div 
                          className="h-10 w-10 rounded-lg flex items-center justify-center"
                          style={{ backgroundColor: `${agent.color}20` }}
                        >
                          <Brain className="h-5 w-5" style={{ color: agent.color }} />
                        </div>
                        {/* Heartbeat pulse */}
                        <motion.div
                          animate={{ 
                            scale: [1, 1.3, 1],
                            opacity: [0.5, 0, 0.5]
                          }}
                          transition={{ 
                            duration: 2,
                            repeat: Infinity,
                            ease: "easeOut"
                          }}
                          className="absolute inset-0 rounded-lg"
                          style={{ border: `2px solid ${agent.color}` }}
                        />
                      </div>
                      
                      <div>
                        <p style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                          {agent.name}
                        </p>
                        <p className="text-sm text-gray-500" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                          {agent.activity}
                        </p>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <div className="h-1.5 w-1.5 rounded-full animate-pulse" style={{ backgroundColor: agent.color }} />
                      <span className="text-xs uppercase tracking-wide" style={{ color: agent.color, fontFamily: 'Inter Tight, sans-serif' }}>
                        {agent.status}
                      </span>
                    </div>
                  </motion.div>
                ))}
              </div>

              {/* Stats Panel */}
              <div className="p-6 bg-[#0A0A0B]/20">
                <h4 className="text-sm text-gray-500 uppercase tracking-wide mb-4" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Session Stats
                </h4>
                
                <div className="space-y-4">
                  {[
                    { label: 'Total Invocations', value: '1,247', trend: '+12%' },
                    { label: 'Avg Response Time', value: '234ms', trend: '-8%' },
                    { label: 'Success Rate', value: '99.2%', trend: '+0.3%' }
                  ].map((stat, i) => (
                    <motion.div
                      key={stat.label}
                      initial={{ opacity: 0, y: 10 }}
                      whileInView={{ opacity: 1, y: 0 }}
                      viewport={{ once: true }}
                      transition={{ delay: 0.3 + i * 0.1 }}
                      className="space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
                          {stat.label}
                        </span>
                        <span className="text-xs text-[#D1FF26]" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                          {stat.trend}
                        </span>
                      </div>
                      <p className="text-2xl" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                        {stat.value}
                      </p>
                      <div className="h-1 bg-[#1A1A1E] rounded-full overflow-hidden">
                        <motion.div
                          initial={{ width: 0 }}
                          whileInView={{ width: '75%' }}
                          viewport={{ once: true }}
                          transition={{ delay: 0.5 + i * 0.1, duration: 0.8 }}
                          className="h-full bg-[#00F5FF]"
                        />
                      </div>
                    </motion.div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* Evidence Panel (Slide-out) */}
          <motion.div
            initial={false}
            animate={{ 
              height: evidenceOpen ? 'auto' : 0,
              opacity: evidenceOpen ? 1 : 0
            }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <div className="mt-4 bg-[#0A0A0B]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-6">
              <div className="flex items-center gap-3 mb-4">
                <Terminal className="h-5 w-5 text-[#00F5FF]" />
                <h4 style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  Real-time Evidence Stream
                </h4>
              </div>
              
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {logs.map((log, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: i * 0.05 }}
                    className="flex items-start gap-3 p-3 bg-[#1A1A1E]/40 rounded-lg border border-white/5 hover:border-white/10 transition-colors"
                  >
                    <span className="text-xs text-gray-600 font-mono flex-shrink-0">
                      {log.time}
                    </span>
                    <span 
                      className="text-xs px-2 py-0.5 rounded flex-shrink-0"
                      style={{ 
                        backgroundColor: log.type === 'success' ? '#D1FF2620' : log.type === 'warning' ? '#FFA50020' : '#00F5FF20',
                        color: log.type === 'success' ? '#D1FF26' : log.type === 'warning' ? '#FFA500' : '#00F5FF',
                        fontFamily: 'Inter Tight, sans-serif'
                      }}
                    >
                      {log.agent}
                    </span>
                    <span className="text-xs text-gray-300" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                      {log.message}
                    </span>
                  </motion.div>
                ))}
              </div>
            </div>
          </motion.div>

          {/* Glow effect */}
          <div className="absolute inset-0 bg-gradient-to-br from-[#00F5FF]/10 to-[#D1FF26]/10 rounded-2xl blur-3xl -z-10 opacity-30" />
        </motion.div>
      </div>
    </section>
  );
}
