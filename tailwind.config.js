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
        'pattern': "url('../assets/image/topography.svg')",
      }
    }
  },
  plugins: [
    require('flowbite/plugin')
  ],
  darkMode: 'class'
}
