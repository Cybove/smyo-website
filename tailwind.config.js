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
        'pattern': "url('../assets/image/test3.svg')",
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
    }
  },
  variants: {
    extend: {
      borderColor: ['hover', 'focus'],
      borderWidth: ['hover', 'focus'],
      backdropBlur: ['hover', 'focus'],
    },
  },
  plugins: [
    require('flowbite/plugin')
  ],
}