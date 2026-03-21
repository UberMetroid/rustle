module.exports = {
  content: ['./index.html', './src/**/*.rs'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        'app-bg': 'var(--app-bg)',
        'app-text': 'var(--app-text)',
        'key-bg': 'var(--key-bg)',
        'key-text': 'var(--key-text)',
        'theme-primary': 'var(--theme-primary)',
      },
      screens: {
        short: { raw: '(max-height: 650px)' },
        xshort: { raw: '(max-height: 560px)' },
        xxshort: { raw: '(max-height: 490px)' },
      },
    },
  },
  plugins: [require('@tailwindcss/forms')],
}
