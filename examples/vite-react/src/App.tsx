import { Button } from './components/Button.pc';
import { Card } from './components/Card.pc';
import { Hero } from './components/Hero.pc';
import { Feature } from './components/Feature.pc';
import './App.css';

function App() {
  return (
    <div className="app">
      {/* Hero Section */}
      <Hero />

      {/* Features Grid */}
      <div className="features-grid">
        <Feature>
          <div slot="icon">⚡️</div>
          <div slot="title">Lightning Fast</div>
          <div slot="description">
            WASM-powered compilation in under 100 microseconds. Instant hot module
            replacement for the best developer experience.
          </div>
        </Feature>

        <Feature>
          <div slot="icon">🎨</div>
          <div slot="title">Visual Design</div>
          <div slot="description">
            Build UI components visually with a canvas. No more switching between
            code and design tools.
          </div>
        </Feature>

        <Feature>
          <div slot="icon">🔒</div>
          <div slot="title">Type Safe</div>
          <div slot="description">
            First-class TypeScript support with auto-generated type definitions
            for all components.
          </div>
        </Feature>
      </div>

      {/* Cards Section */}
      <div className="cards-container">
        <Card>
          <div slot="header">🎯 Components</div>
          <div slot="content">
            Build reusable components with slots, styles, and full TypeScript
            support. Components are compiled to React at build time.
          </div>
          <div slot="footer">
            <Button>Learn More</Button>
          </div>
        </Card>

        <Card>
          <div slot="header">🚀 Performance</div>
          <div slot="content">
            WebAssembly compilation means zero runtime overhead. Your components
            compile to pure React code.
          </div>
          <div slot="footer">
            <Button>View Benchmarks</Button>
          </div>
        </Card>

        <Card>
          <div slot="header">🛠️ Tooling</div>
          <div slot="content">
            Works with Vite, Webpack, Rollup, and esbuild. Hot module replacement
            for instant updates during development.
          </div>
          <div slot="footer">
            <Button>Get Started</Button>
          </div>
        </Card>
      </div>

      {/* Button Examples */}
      <div className="buttons-section">
        <h2>Interactive Components</h2>
        <div className="buttons-row">
          <Button>Default Button</Button>
          <Button>
            <span>🎉</span> With Icon
          </Button>
          <Button>
            <strong>Bold Text</strong>
          </Button>
        </div>
      </div>
    </div>
  );
}

export default App;
