module.exports = {
    content: [
        "./index.html",
        "./src/**/*.rs"
    ],
    theme: {
        extend: {
            // JPL-compliant color system with fault tolerance
            colors: {
                jpl: {
                    dark: '#0a0e14',
                    navy: '#171f2e',
                    cyan: '#66d9ef',
                    green: '#a6e22e',
                    magenta: '#f92672',
                    gray: '#f8fafc',
                },
            },
            // Radiation-hardened spacing system
            spacing: {
                'safe-bottom': 'max(env(safe-area-inset-bottom), 2rem)',
                'safe-top': 'max(env(safe-area-inset-top), 1rem)',
            },
            // Fault-tolerant height utilities
            height: {
                'screen-safe': 'calc(var(--vh, 1vh) * 100)',
            },
            // Mission-critical fonts
            fontFamily: {
                mono: ['JetBrains Mono', 'Fira Code', 'Courier New', 'monospace'],
            },
            // Bounded animation system
            animation: {
                'blink': 'blink 1.1s step-end infinite',
                'glow': 'metallic-pulse 2.5s infinite ease-in-out',
                'fade-in': 'fadeIn 0.6s ease-out forwards',
                'data-pulse': 'data-transmission 8s infinite',
                'grid-fade': 'grid-fade 15s infinite alternate ease-in-out',
            },
            // SEU-resistant box shadows
            boxShadow: {
                'glow-cyan': '0 0 12px rgba(102, 217, 239, 0.4)',
                'glow-green': '0 0 12px rgba(166, 226, 46, 0.4)',
                'focus-ring': '0 0 0 2px #66d9ef, 0 0 0 4px rgba(102, 217, 239, 0.3)',
            },
            // Radiation-hardened transform utilities
            transformOrigin: {
                'center-baseline': 'center baseline',
            },
            translate: {
                'method-call': '0 0.4em',
            },
            // Mobile-specific adjustments
            screens: {
                'xs': '380px', // Extra small screen breakpoint
            },
        },
    },
    plugins: [
        // Inject the iOS viewport height fix
        function({ addBase }) {
            addBase({
                ':root': {
                    '--vh': '1vh',
                }
            });
        },
    ],
}
