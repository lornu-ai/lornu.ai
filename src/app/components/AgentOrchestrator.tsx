import { motion } from 'motion/react';
import { useEffect, useState } from 'react';
import { Brain, Shield, Terminal, CheckCircle2, Activity } from 'lucide-react';

interface Agent {
  id: string;
  name: string;
  role: string;
  icon: any;
  status: 'idle' | 'processing' | 'complete';
  color: string;
}

export function AgentOrchestrator() {
  const [activeAgent, setActiveAgent] = useState(0);
  
  const agents: Agent[] = [
    { id: 'triage', name: 'Triage Agent', role: 'Analysis', icon: Brain, status: 'complete', color: '#D1FF26' },
    { id: 'sre', name: 'SRE Agent', role: 'Operations', icon: Shield, status: 'processing', color: '#00F5FF' },
    { id: 'gac', name: 'GAC Agent', role: 'Governance', icon: CheckCircle2, status: 'idle', color: '#8B5CF6' }
  ];

  useEffect(() => {
    const interval = setInterval(() => {
      setActiveAgent((prev) => (prev + 1) % agents.length);
    }, 2500);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="relative w-full max-w-xl mx-auto">
      {/* Glassmorphic container */}
      <div className="relative bg-[#1A1A1E]/40 backdrop-blur-xl border border-white/10 rounded-2xl p-8 shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-lg bg-[#00F5FF]/20 flex items-center justify-center">
              <Activity className="h-5 w-5 text-[#00F5FF]" />
            </div>
            <div>
              <h3 style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                Orchestrator Agent
              </h3>
              <p className="text-sm text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
                Multi-Agent Workflow
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <div className="h-2 w-2 rounded-full bg-[#D1FF26] animate-pulse" />
            <span className="text-xs text-[#D1FF26]" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
              ACTIVE
            </span>
          </div>
        </div>

        {/* Central Orchestrator */}
        <div className="relative flex flex-col items-center gap-8 py-6">
          {/* Orchestrator Hub */}
          <motion.div
            animate={{ 
              scale: [1, 1.05, 1],
            }}
            transition={{ 
              duration: 2,
              repeat: Infinity,
              ease: "easeInOut"
            }}
            className="relative"
          >
            <div className="h-24 w-24 rounded-2xl bg-gradient-to-br from-[#00F5FF]/30 to-[#D1FF26]/30 border border-[#00F5FF]/50 flex items-center justify-center backdrop-blur-sm">
              <Brain className="h-10 w-10 text-[#00F5FF]" />
            </div>
            {/* Pulse rings */}
            <motion.div
              animate={{ 
                scale: [1, 1.5, 1],
                opacity: [0.5, 0, 0.5]
              }}
              transition={{ 
                duration: 2,
                repeat: Infinity,
                ease: "easeOut"
              }}
              className="absolute inset-0 rounded-2xl border-2 border-[#00F5FF]"
            />
          </motion.div>

          {/* Connecting Lines & Agent Cards */}
          <div className="grid grid-cols-3 gap-4 w-full">
            {agents.map((agent, index) => {
              const Icon = agent.icon;
              const isActive = index === activeAgent;
              
              return (
                <motion.div
                  key={agent.id}
                  animate={{
                    scale: isActive ? 1.05 : 1,
                    borderColor: isActive ? agent.color : 'rgba(255,255,255,0.1)'
                  }}
                  className="relative bg-[#0A0A0B]/60 backdrop-blur-sm border rounded-xl p-4 transition-all"
                >
                  {/* Connection line to orchestrator */}
                  <svg 
                    className="absolute -top-12 left-1/2 -translate-x-1/2 w-1 h-12"
                    style={{ overflow: 'visible' }}
                  >
                    <motion.line
                      x1="2" y1="0" x2="2" y2="48"
                      stroke={isActive ? agent.color : 'rgba(255,255,255,0.2)'}
                      strokeWidth="2"
                      strokeDasharray="4 4"
                      animate={{
                        strokeDashoffset: isActive ? [0, -8] : 0,
                        opacity: isActive ? 1 : 0.3
                      }}
                      transition={{
                        strokeDashoffset: {
                          duration: 1,
                          repeat: Infinity,
                          ease: "linear"
                        }
                      }}
                    />
                  </svg>

                  {/* Agent Icon */}
                  <div className="flex justify-center mb-2">
                    <div 
                      className="h-12 w-12 rounded-lg flex items-center justify-center"
                      style={{ 
                        backgroundColor: `${agent.color}20`,
                      }}
                    >
                      <Icon className="h-6 w-6" style={{ color: agent.color }} />
                    </div>
                  </div>

                  {/* Agent Info */}
                  <div className="text-center space-y-1">
                    <p className="text-xs" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                      {agent.name}
                    </p>
                    <p className="text-xs text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
                      {agent.role}
                    </p>
                  </div>

                  {/* Status indicator */}
                  <div className="mt-2 flex justify-center">
                    {isActive && (
                      <motion.div
                        initial={{ scale: 0 }}
                        animate={{ scale: 1 }}
                        className="flex items-center gap-1 text-xs"
                        style={{ color: agent.color, fontFamily: 'JetBrains Mono, monospace' }}
                      >
                        <div className="h-1.5 w-1.5 rounded-full animate-pulse" style={{ backgroundColor: agent.color }} />
                        Processing
                      </motion.div>
                    )}
                  </div>
                </motion.div>
              );
            })}
          </div>
        </div>

        {/* A2A Protocol Footer */}
        <div className="mt-6 pt-4 border-t border-white/10">
          <div className="flex items-center justify-between text-xs">
            <span className="text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
              A2A Protocol v2.1
            </span>
            <div className="flex items-center gap-2">
              <Terminal className="h-3 w-3 text-[#00F5FF]" />
              <span className="text-gray-500" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
                3 agents • 12ms latency
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Glow effect */}
      <div className="absolute inset-0 bg-gradient-to-br from-[#00F5FF]/20 to-[#D1FF26]/20 rounded-2xl blur-3xl -z-10 opacity-50" />
    </div>
  );
}
