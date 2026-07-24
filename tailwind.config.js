/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.html"],
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["light", "dark", "cupcake", "retro"],
  },
}
