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
    }
  },
  variants: {
    extend: {
      borderColor: ['hover', 'focus'],
      borderWidth: ['hover', 'focus'],
    },
  },
  plugins: [
    require('flowbite/plugin')
  ],
  darkMode: 'class'
}