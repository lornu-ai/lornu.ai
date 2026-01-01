import { motion } from 'motion/react';
import { Button } from './ui/button';
import { Check, ArrowRight, Sparkles } from 'lucide-react';

interface PricingTier {
  name: string;
  price: string;
  period: string;
  description: string;
  features: string[];
  highlighted?: boolean;
  cta: string;
}

export function PricingSection() {
  const tiers: PricingTier[] = [
    {
      name: 'Starter',
      price: '$199',
      period: '/month',
      description: 'Perfect for individual developers and small projects',
      features: [
        'Single Developer Access',
        'Up to 5 Agents',
        'Standard Quality Gates',
        '10K Agent Invocations/mo',
        'Community Support',
        'Basic Analytics'
      ],
      cta: 'Start Free Trial'
    },
    {
      name: 'Professional',
      price: '$999',
      period: '/month',
      description: 'Built for growing teams shipping production AI',
      features: [
        'Teams up to 20',
        'Unlimited Agents',
        'A2A Protocol Access',
        '100K Agent Invocations/mo',
        'Priority Support',
        'Advanced Analytics',
        'Custom Quality Gates',
        'SSO & RBAC'
      ],
      highlighted: true,
      cta: 'Start Free Trial'
    },
    {
      name: 'Enterprise',
      price: 'Custom',
      period: '',
      description: 'Full platform access with dedicated support',
      features: [
        'Unlimited Team Members',
        'Unlimited Everything',
        'GAC Governance Agent',
        'Custom SLA',
        'Dedicated Support',
        'Professional Services',
        'On-Premise Deployment',
        'Custom Integrations'
      ],
      cta: 'Contact Sales'
    }
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
            <span className="text-[#00F5FF]">Simple</span>, Transparent Pricing
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto" style={{ fontFamily: 'Inter, sans-serif' }}>
            Start free, scale as you grow. All plans include our core platform capabilities.
          </p>
        </motion.div>

        {/* Pricing Cards */}
        <div className="grid md:grid-cols-3 gap-8 max-w-6xl mx-auto">
          {tiers.map((tier, index) => (
            <motion.div
              key={tier.name}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
              className={`relative rounded-2xl p-8 ${
                tier.highlighted
                  ? 'bg-gradient-to-br from-[#00F5FF]/10 to-[#D1FF26]/10 border-2 border-[#00F5FF]/50'
                  : 'bg-[#1A1A1E]/40 border border-white/10'
              } backdrop-blur-sm hover:border-[#00F5FF]/30 transition-all ${
                tier.highlighted ? 'md:scale-105' : ''
              }`}
            >
              {/* Most Popular Badge */}
              {tier.highlighted && (
                <div className="absolute -top-4 left-1/2 -translate-x-1/2">
                  <div className="flex items-center gap-2 px-4 py-1.5 bg-[#D1FF26] text-[#0A0A0B] rounded-full">
                    <Sparkles className="h-3 w-3" />
                    <span className="text-xs" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                      MOST POPULAR
                    </span>
                  </div>
                </div>
              )}

              {/* Tier Name */}
              <div className="mb-6">
                <h3 
                  className="text-2xl mb-2"
                  style={{ fontFamily: 'Inter Tight, sans-serif' }}
                >
                  {tier.name}
                </h3>
                <p className="text-sm text-gray-400" style={{ fontFamily: 'Inter, sans-serif' }}>
                  {tier.description}
                </p>
              </div>

              {/* Price */}
              <div className="mb-8">
                <div className="flex items-baseline gap-1">
                  <span 
                    className={`${tier.price === 'Custom' ? 'text-4xl' : 'text-5xl'}`}
                    style={{ fontFamily: 'Inter Tight, sans-serif' }}
                  >
                    {tier.price}
                  </span>
                  {tier.period && (
                    <span className="text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
                      {tier.period}
                    </span>
                  )}
                </div>
              </div>

              {/* CTA Button */}
              <Button 
                className={`w-full mb-8 ${
                  tier.highlighted
                    ? 'bg-[#00F5FF] hover:bg-[#00F5FF]/90 text-[#0A0A0B]'
                    : 'bg-[#1A1A1E] hover:bg-[#1A1A1E]/80 border border-[#00F5FF]/30'
                }`}
                style={{ fontFamily: 'Inter, sans-serif' }}
              >
                {tier.cta}
                <ArrowRight className="ml-2 h-4 w-4" />
              </Button>

              {/* Features List */}
              <div className="space-y-3">
                <p className="text-xs text-gray-500 uppercase tracking-wide mb-4" style={{ fontFamily: 'Inter Tight, sans-serif' }}>
                  What's Included
                </p>
                {tier.features.map((feature, i) => (
                  <motion.div
                    key={i}
                    initial={{ opacity: 0, x: -10 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{ delay: 0.3 + i * 0.05 }}
                    className="flex items-start gap-3"
                  >
                    <div className={`h-5 w-5 rounded-full flex items-center justify-center flex-shrink-0 ${
                      tier.highlighted ? 'bg-[#D1FF26]/20' : 'bg-[#00F5FF]/20'
                    }`}>
                      <Check className={`h-3 w-3 ${tier.highlighted ? 'text-[#D1FF26]' : 'text-[#00F5FF]'}`} />
                    </div>
                    <span className="text-sm" style={{ fontFamily: 'Inter, sans-serif' }}>
                      {feature}
                    </span>
                  </motion.div>
                ))}
              </div>

              {/* Glow effect for highlighted tier */}
              {tier.highlighted && (
                <div className="absolute inset-0 bg-gradient-to-br from-[#00F5FF]/20 to-[#D1FF26]/20 rounded-2xl blur-2xl -z-10 opacity-50" />
              )}
            </motion.div>
          ))}
        </div>

        {/* Additional Info */}
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
          className="text-center mt-12 text-sm text-gray-500"
          style={{ fontFamily: 'Inter, sans-serif' }}
        >
          All plans include 14-day free trial • No credit card required • Cancel anytime
        </motion.div>
      </div>
    </section>
  );
}
