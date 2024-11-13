import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "Gerust",
  favicon: "img/favicon.ico",

  url: "https://docs.gerust.rs",
  baseUrl: "/",

  organizationName: "mainmatter",
  projectName: "gerust",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "throw",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: "https://github.com/mainmatter/gerust/tree/main/docs/",
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "img/docusaurus-social-card.jpg",
    navbar: {
      title: "Gerust",
      logo: {
        alt: "Gerust Logo",
        src: "img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "tutorialSidebar",
          position: "left",
          label: "Tutorial",
        },
        {
          href: "https://github.com/mainmatter/gerust/",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            {
              label: "Intro",
              to: "/docs/",
            },
          ],
        },
        {
          title: "More",
          items: [
            {
              href: "https://mainmatter.com/rust/",
              label: "Mainmatter",
            },
            {
              label: "GitHub",
              href: "https://github.com/mainmatter/gerust",
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Mainmatter GmbH and contributors`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      magicComments: [
        // Code Diff Higlights
        // See: https://github.com/facebook/docusaurus/issues/3318#issuecomment-1909563681
        {
          className: "code-block-diff-add-line",
          line: "diff-add",
        },
        {
          className: "code-block-diff-remove-line",
          line: "diff-remove",
        },
      ],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
