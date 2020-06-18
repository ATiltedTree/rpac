module.exports = {
  title: "RPac",
  description: "A pacman re-implementation with AUR support",
  theme: "default-prefers-color-scheme",
  themeConfig: {
    domain: "https://rpac.netlify.app",
    logo: "/assets/img/logo.svg",
    repo: "ATiltedTree/rpac",
    docsDir: "docs",
    editLinks: true,
    lastUpdated: "Last Updated",
    smoothScroll: true,
    type: "website",
    postcss: {
      plugins: [require("css-prefers-color-scheme/postcss")],
    },
  },
  plugins: ["@vuepress/back-to-top", "seo"],
  evergreen: true,
};
