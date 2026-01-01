import { motion } from 'motion/react';
import { Button } from './ui/button';
import { Menu, X } from 'lucide-react';
import { useState } from 'react';

export function Header() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const navItems = [
    { label: 'Features', href: '#features' },
    { label: 'Use Cases', href: '#use-cases' },
    { label: 'Pricing', href: '#pricing' },
    { label: 'Documentation', href: '#docs' },
  ];

  return (
    <motion.header
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      className="fixed top-0 left-0 right-0 z-50 bg-[#0A0A0B]/80 backdrop-blur-xl border-b border-white/10"
    >
      <div className="max-w-7xl mx-auto px-4 md:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-[#00F5FF] to-[#D1FF26] flex items-center justify-center">
              <span className="text-[#0A0A0B]" style={{ fontFamily: 'Inter Tight, sans-serif' }}>L</span>
            </div>
            <span 
              className="text-xl"
              style={{ fontFamily: 'Inter Tight, sans-serif', letterSpacing: '-0.02em' }}
            >
              Lornu
            </span>
          </div>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex items-center gap-8">
            {navItems.map((item) => (
              <a
                key={item.label}
                href={item.href}
                className="text-sm text-gray-400 hover:text-[#00F5FF] transition-colors"
                style={{ fontFamily: 'Inter, sans-serif' }}
              >
                {item.label}
              </a>
            ))}
          </nav>

          {/* CTA Buttons */}
          <div className="hidden md:flex items-center gap-4">
            <Button 
              variant="ghost" 
              className="text-gray-400 hover:text-[#00F5FF]"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              Sign In
            </Button>
            <Button 
              className="bg-[#00F5FF] hover:bg-[#00F5FF]/90 text-[#0A0A0B]"
              style={{ fontFamily: 'Inter, sans-serif' }}
            >
              Start Free Trial
            </Button>
          </div>

          {/* Mobile Menu Button */}
          <button
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            className="md:hidden p-2 text-gray-400 hover:text-[#00F5FF] transition-colors"
          >
            {mobileMenuOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
          </button>
        </div>

        {/* Mobile Menu */}
        {mobileMenuOpen && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="md:hidden py-4 border-t border-white/10"
          >
            <nav className="flex flex-col gap-4">
              {navItems.map((item) => (
                <a
                  key={item.label}
                  href={item.href}
                  className="text-gray-400 hover:text-[#00F5FF] transition-colors py-2"
                  style={{ fontFamily: 'Inter, sans-serif' }}
                  onClick={() => setMobileMenuOpen(false)}
                >
                  {item.label}
                </a>
              ))}
              <div className="pt-4 border-t border-white/10 space-y-3">
                <Button 
                  variant="ghost" 
                  className="w-full text-gray-400 hover:text-[#00F5FF]"
                  style={{ fontFamily: 'Inter, sans-serif' }}
                >
                  Sign In
                </Button>
                <Button 
                  className="w-full bg-[#00F5FF] hover:bg-[#00F5FF]/90 text-[#0A0A0B]"
                  style={{ fontFamily: 'Inter, sans-serif' }}
                >
                  Start Free Trial
                </Button>
              </div>
            </nav>
          </motion.div>
        )}
      </div>
    </motion.header>
  );
}
