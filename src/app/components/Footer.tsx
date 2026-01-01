import { Github, Twitter, Linkedin, Mail } from 'lucide-react';

export function Footer() {
  const footerSections = [
    {
      title: 'Product',
      links: ['Features', 'Pricing', 'Documentation', 'API Reference', 'Changelog']
    },
    {
      title: 'Use Cases',
      links: ['Autonomous SRE', 'Code Governance', 'Multi-IDE Sync', 'Performance Monitoring', 'Security']
    },
    {
      title: 'Company',
      links: ['About', 'Blog', 'Careers', 'Contact', 'Partners']
    },
    {
      title: 'Resources',
      links: ['Community', 'Support', 'Status', 'Terms', 'Privacy']
    }
  ];

  return (
    <footer className="relative border-t border-white/10 bg-[#0A0A0B]">
      <div className="max-w-7xl mx-auto px-4 md:px-8 py-16">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          {/* Logo & Description */}
          <div className="col-span-2 md:col-span-1">
            <div className="flex items-center gap-2 mb-4">
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
            <p className="text-sm text-gray-500 mb-4" style={{ fontFamily: 'Inter, sans-serif' }}>
              The AI Agent Platform for Enterprise Teams
            </p>
            <div className="flex items-center gap-3">
              <a 
                href="#" 
                className="h-9 w-9 rounded-lg bg-[#1A1A1E] flex items-center justify-center hover:bg-[#00F5FF]/20 hover:text-[#00F5FF] transition-colors"
              >
                <Twitter className="h-4 w-4" />
              </a>
              <a 
                href="#" 
                className="h-9 w-9 rounded-lg bg-[#1A1A1E] flex items-center justify-center hover:bg-[#00F5FF]/20 hover:text-[#00F5FF] transition-colors"
              >
                <Github className="h-4 w-4" />
              </a>
              <a 
                href="#" 
                className="h-9 w-9 rounded-lg bg-[#1A1A1E] flex items-center justify-center hover:bg-[#00F5FF]/20 hover:text-[#00F5FF] transition-colors"
              >
                <Linkedin className="h-4 w-4" />
              </a>
              <a 
                href="#" 
                className="h-9 w-9 rounded-lg bg-[#1A1A1E] flex items-center justify-center hover:bg-[#00F5FF]/20 hover:text-[#00F5FF] transition-colors"
              >
                <Mail className="h-4 w-4" />
              </a>
            </div>
          </div>

          {/* Footer Links */}
          {footerSections.map((section) => (
            <div key={section.title}>
              <h4 
                className="text-sm mb-4"
                style={{ fontFamily: 'Inter Tight, sans-serif' }}
              >
                {section.title}
              </h4>
              <ul className="space-y-3">
                {section.links.map((link) => (
                  <li key={link}>
                    <a
                      href="#"
                      className="text-sm text-gray-500 hover:text-[#00F5FF] transition-colors"
                      style={{ fontFamily: 'Inter, sans-serif' }}
                    >
                      {link}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom Bar */}
        <div className="pt-8 border-t border-white/10 flex flex-col md:flex-row items-center justify-between gap-4">
          <p className="text-sm text-gray-500" style={{ fontFamily: 'Inter, sans-serif' }}>
            © 2026 Lornu AI. All rights reserved.
          </p>
          <div className="flex items-center gap-6 text-sm text-gray-500">
            <a href="#" className="hover:text-[#00F5FF] transition-colors" style={{ fontFamily: 'Inter, sans-serif' }}>
              Privacy Policy
            </a>
            <a href="#" className="hover:text-[#00F5FF] transition-colors" style={{ fontFamily: 'Inter, sans-serif' }}>
              Terms of Service
            </a>
            <a href="#" className="hover:text-[#00F5FF] transition-colors" style={{ fontFamily: 'Inter, sans-serif' }}>
              Cookie Policy
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
