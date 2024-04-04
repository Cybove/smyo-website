/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./node_modules/flowbite/**/*.js",
    "./src/**/*.{html,js}",
    "./public/**/*.html"
  ],
  theme: {
    extend: {
      backgroundImage: {
        'pattern': "url('../assets/image/bg.svg')",
      },
      backgroundSize: {
        'cover': 'cover',
      },
      backgroundPosition: {
        'center': 'center',
      },
      backdropBlur: {
        'none': '0',
        'sm': '6px',
        'DEFAULT': '8px',
        'md': '10px',
        'lg': '12px',
        'xl': '16px',
        '2xl': '24px',
        '3xl': '48px',
      }
    },
    screens: {
      'xs': '360px',
      'sm': '640px',
      'md': '768px',
      'lg': '1024px',
      'xl': '1280px',
      '2xl': '1536px',
    },
    display: ['responsive']
  },
  variants: {
    extend: {
      borderColor: ['hover', 'focus'],
      borderWidth: ['hover', 'focus'],
      backdropBlur: ['hover', 'focus'],
      display: ['responsive', 'group-hover', 'group-focus'],
    },
  },
  plugins: [
    require('flowbite/plugin')
  ],


}