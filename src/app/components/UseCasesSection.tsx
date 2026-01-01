import { motion } from 'motion/react';
import { Shield, CheckCircle, Code2, Zap, Database, GitBranch } from 'lucide-react';

interface UseCase {
  title: string;
  description: string;
  icon: any;
  color: string;
  stats: { label: string; value: string }[];
}

export function UseCasesSection() {
  const useCases: UseCase[] = [
    {
      title: 'Autonomous SRE',
      description: 'AI-powered incident response and resolution with automatic triage, diagnosis, and mitigation',
      icon: Shield,
      color: '#00F5FF',
      stats: [
        { label: 'Faster MTTR', value: '73%' },
        { label: 'Auto-Resolved', value: '45%' }
      ]
    },
    {
      title: 'Code Governance',
      description: 'Automated code review, security scanning, and compliance enforcement across your entire codebase',
      icon: CheckCircle,
      color: '#D1FF26',
      stats: [
        { label: 'Vulnerabilities Found', value: '2.4K' },
        { label: 'Policy Compliance', value: '99%' }
      ]
    },
    {
      title: 'Multi-IDE Sync',
      description: 'Seamlessly sync context and code across Cursor, Windsurf, and other development environments',
      icon: Code2,
      color: '#8B5CF6',
      stats: [
        { label: 'Context Switches', value: '0ms' },
        { label: 'Sync Accuracy', value: '100%' }
      ]
    }
  ];

  return (
    <section className="relative py-32 px-4 md:px-8 overflow-hidden">
      {/* Background accents */}
      <div className="absolute top-0 right-0 w-96 h-96 bg-[#00F5FF]/5 rounded-full blur-3xl" />
      
      <div className="max-w-7xl mx-auto relative z-10">
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
            What You Can <span className="text-[#00F5FF]">Build</span>
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto" style={{ fontFamily: 'Inter, sans-serif' }}>
            Real-world applications powered by the Lornu Agentic Framework
          </p>
        </motion.div>

        {/* Use Case Cards - Horizontal Scroll on Mobile, Grid on Desktop */}
        <div className="grid md:grid-cols-3 gap-6">
          {useCases.map((useCase, index) => {
            const Icon = useCase.icon;
            
            return (
              <motion.div
                key={useCase.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.15 }}
                className="group relative bg-[#1A1A1E]/40 backdrop-blur-sm border border-white/10 rounded-2xl p-8 hover:border-[#00F5FF]/30 transition-all overflow-hidden"
              >
                {/* Gradient overlay on hover */}
                <div 
                  className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500"
                  style={{
                    background: `linear-gradient(135deg, ${useCase.color}10 0%, transparent 100%)`
                  }}
                />

                <div className="relative z-10">
                  {/* Icon */}
                  <div 
                    className="h-14 w-14 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform"
                    style={{ backgroundColor: `${useCase.color}20` }}
                  >
                    <Icon className="h-7 w-7" style={{ color: useCase.color }} />
                  </div>

                  {/* Content */}
                  <h3 
                    className="text-2xl mb-3"
                    style={{ fontFamily: 'Inter Tight, sans-serif' }}
                  >
                    {useCase.title}
                  </h3>
                  
                  <p className="text-gray-400 mb-6" style={{ fontFamily: 'Inter, sans-serif' }}>
                    {useCase.description}
                  </p>

                  {/* Stats */}
                  <div className="grid grid-cols-2 gap-4 pt-6 border-t border-white/10">
                    {useCase.stats.map((stat, i) => (
                      <div key={i}>
                        <div 
                          className="text-2xl mb-1"
                          style={{ 
                            fontFamily: 'Inter Tight, sans-serif',
                            color: useCase.color
                          }}
                        >
                          {stat.value}
                        </div>
                        <div className="text-xs text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
                          {stat.label}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </motion.div>
            );
          })}
        </div>

        {/* Additional Use Cases Grid */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
          className="mt-12 grid grid-cols-2 md:grid-cols-4 gap-4"
        >
          {[
            { icon: Zap, label: 'Performance Monitoring' },
            { icon: Database, label: 'Data Pipeline Ops' },
            { icon: GitBranch, label: 'Release Automation' },
            { icon: Shield, label: 'Security Posture' }
          ].map((item, i) => {
            const Icon = item.icon;
            return (
              <div
                key={i}
                className="flex items-center gap-3 p-4 bg-[#1A1A1E]/20 backdrop-blur-sm border border-white/5 rounded-xl hover:border-[#00F5FF]/20 transition-colors"
              >
                <Icon className="h-5 w-5 text-[#00F5FF]" />
                <span className="text-sm" style={{ fontFamily: 'Inter, sans-serif' }}>
                  {item.label}
                </span>
              </div>
            );
          })}
        </motion.div>
      </div>
    </section>
  );
}
