import { motion } from 'motion/react';
import { Button } from './ui/button';
import { ArrowRight, BookOpen } from 'lucide-react';
import { AgentOrchestrator } from './AgentOrchestrator';

export function HeroSection() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden px-4 md:px-8 py-20">
      {/* Background gradient effects */}
      <div className="absolute inset-0 bg-gradient-to-br from-[#00F5FF]/5 via-transparent to-[#D1FF26]/5 pointer-events-none" />
      <div className="absolute top-20 -left-20 w-96 h-96 bg-[#00F5FF]/10 rounded-full blur-3xl" />
      <div className="absolute bottom-20 -right-20 w-96 h-96 bg-[#D1FF26]/10 rounded-full blur-3xl" />

      <div className="max-w-7xl mx-auto w-full grid lg:grid-cols-[1fr_0.9fr] gap-12 items-center relative z-10">
        {/* Left Side - Copy */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="space-y-8"
        >
          <div className="space-y-4">
            <motion.div
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.2 }}
              className="inline-block"
            >
              <span className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-[#1A1A1E] border border-[#00F5FF]/20">
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#D1FF26] opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-[#D1FF26]"></span>
                </span>
                <span className="text-sm text-[#00F5FF]" style={{ fontFamily: 'Inter, sans-serif' }}>
                  Platform Now Live
                </span>
              </span>
            </motion.div>

            <h1 
              className="text-5xl md:text-6xl lg:text-7xl tracking-tight"
              style={{ 
                fontFamily: 'Inter Tight, sans-serif',
                letterSpacing: '-0.02em'
              }}
            >
              The AI Agent Platform{' '}
              <span className="block mt-2">
                for <span className="text-[#00F5FF]">Enterprise Teams</span>
              </span>
            </h1>

            <p 
              className="text-xl text-gray-400 max-w-2xl"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              Build, deploy, and orchestrate autonomous agents with a fully customizable, 
              off-the-shelf platform. Ship production-ready AI solutions in days, not months.
            </p>
          </div>

          <div className="flex flex-col sm:flex-row gap-4">
            <Button 
              size="lg"
              className="bg-[#00F5FF] hover:bg-[#00F5FF]/90 text-[#0A0A0B] group"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              Start Free Trial
              <ArrowRight className="ml-2 h-4 w-4 group-hover:translate-x-1 transition-transform" />
            </Button>
            <Button 
              size="lg"
              variant="outline"
              className="border-[#00F5FF]/30 hover:bg-[#00F5FF]/10"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              <BookOpen className="mr-2 h-4 w-4" />
              View Documentation
            </Button>
          </div>

          {/* Trust indicators */}
          <div className="pt-8 flex items-center gap-8 text-sm text-gray-500">
            <div className="flex items-center gap-2">
              <div className="h-8 w-8 rounded-full bg-[#D1FF26]/20 flex items-center justify-center">
                <span className="text-[#D1FF26]">✓</span>
              </div>
              <span>SOC 2 Compliant</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="h-8 w-8 rounded-full bg-[#D1FF26]/20 flex items-center justify-center">
                <span className="text-[#D1FF26]">✓</span>
              </div>
              <span>99.9% Uptime SLA</span>
            </div>
          </div>
        </motion.div>

        {/* Right Side - Visual Proof */}
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.6, delay: 0.3 }}
          className="relative"
        >
          <AgentOrchestrator />
        </motion.div>
      </div>
    </section>
  );
}
