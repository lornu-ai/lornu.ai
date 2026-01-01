import { motion } from 'motion/react';
import { Button } from './ui/button';
import { ArrowRight, Sparkles } from 'lucide-react';

export function CTASection() {
  return (
    <section className="relative py-32 px-4 md:px-8 overflow-hidden">
      {/* Background effects */}
      <div className="absolute inset-0 bg-gradient-to-br from-[#00F5FF]/10 via-transparent to-[#D1FF26]/10" />
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-[#00F5FF]/20 rounded-full blur-3xl" />
      
      <div className="max-w-4xl mx-auto relative z-10">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-center space-y-8"
        >
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            whileInView={{ opacity: 1, scale: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-[#1A1A1E] border border-[#D1FF26]/30"
          >
            <Sparkles className="h-4 w-4 text-[#D1FF26]" />
            <span className="text-sm text-[#D1FF26]" style={{ fontFamily: 'Inter, sans-serif' }}>
              Join 500+ teams building with Lornu
            </span>
          </motion.div>

          {/* Headline */}
          <h2 
            className="text-4xl md:text-6xl max-w-3xl mx-auto"
            style={{ 
              fontFamily: 'Inter Tight, sans-serif',
              letterSpacing: '-0.02em',
              lineHeight: '1.1'
            }}
          >
            Ready to Build the Future of{' '}
            <span className="text-[#00F5FF]">AI Agents</span>?
          </h2>

          {/* Subheadline */}
          <p 
            className="text-xl text-gray-400 max-w-2xl mx-auto"
            style={{ fontFamily: 'Inter, sans-serif' }}
          >
            Start your 14-day free trial today. No credit card required. 
            Deploy your first autonomous agent in minutes.
          </p>

          {/* CTA Buttons */}
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-4">
            <Button 
              size="lg"
              className="bg-[#00F5FF] hover:bg-[#00F5FF]/90 text-[#0A0A0B] px-8 group"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              Start Free Trial
              <ArrowRight className="ml-2 h-5 w-5 group-hover:translate-x-1 transition-transform" />
            </Button>
            <Button 
              size="lg"
              variant="outline"
              className="border-[#00F5FF]/30 hover:bg-[#00F5FF]/10 px-8"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              Schedule Demo
            </Button>
          </div>

          {/* Trust Indicators */}
          <div className="pt-12 flex flex-wrap items-center justify-center gap-8 text-sm text-gray-500">
            <div className="flex items-center gap-2">
              <div className="h-6 w-6 rounded-full bg-[#D1FF26]/20 flex items-center justify-center">
                <span className="text-[#D1FF26] text-xs">✓</span>
              </div>
              <span>No credit card required</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-6 w-6 rounded-full bg-[#D1FF26]/20 flex items-center justify-center">
                <span className="text-[#D1FF26] text-xs">✓</span>
              </div>
              <span>14-day free trial</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-6 w-6 rounded-full bg-[#D1FF26]/20 flex items-center justify-center">
                <span className="text-[#D1FF26] text-xs">✓</span>
              </div>
              <span>Cancel anytime</span>
            </div>
          </div>
        </motion.div>

        {/* Decorative code snippet */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.4 }}
          className="mt-16 bg-[#0A0A0B]/60 backdrop-blur-xl border border-white/10 rounded-2xl p-6 max-w-2xl mx-auto"
        >
          <div className="flex items-center gap-2 mb-4">
            <div className="h-3 w-3 rounded-full bg-red-500" />
            <div className="h-3 w-3 rounded-full bg-yellow-500" />
            <div className="h-3 w-3 rounded-full bg-green-500" />
            <span className="ml-auto text-xs text-gray-500" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
              Quick Start
            </span>
          </div>
          <pre className="text-sm text-[#00F5FF] overflow-x-auto" style={{ fontFamily: 'JetBrains Mono, monospace' }}>
{`import { Orchestrator } from '@lornu/agent-sdk';

const orchestrator = new Orchestrator({
  agents: ['triage', 'sre', 'gac'],
  qualityGate: 'standard'
});

await orchestrator.deploy();
// ✓ Your autonomous agent is live!`}
          </pre>
        </motion.div>
      </div>
    </section>
  );
}
