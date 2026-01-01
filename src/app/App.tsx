import { Header } from './components/Header';
import { HeroSection } from './components/HeroSection';
import { FeaturesSection } from './components/FeaturesSection';
import { UseCasesSection } from './components/UseCasesSection';
import { PricingSection } from './components/PricingSection';
import { DashboardPreview } from './components/DashboardPreview';
import { CTASection } from './components/CTASection';
import { Footer } from './components/Footer';

export default function App() {
  return (
    <div className="min-h-screen bg-[#0A0A0B] text-white dark">
      <Header />
      
      {/* Main Content */}
      <main className="pt-16">
        <HeroSection />
        
        <div id="features">
          <FeaturesSection />
        </div>
        
        <div id="use-cases">
          <UseCasesSection />
        </div>
        
        <DashboardPreview />
        
        <div id="pricing">
          <PricingSection />
        </div>
        
        <CTASection />
      </main>
      
      <Footer />
    </div>
  );
}