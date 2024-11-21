export default {
  plain: {
    color: "#c9d1d9",
    backgroundColor: "#161b22",
  },
  styles: [
    {
      types: ["namespace"],
      style: {
        opacity: "0.7",
      },
    },
    {
      types: ["comment", "prolog", "doctype", "cdata"],
      style: {
        color: "#8b949e",
      },
    },
    {
      types: ["punctuation"],
      style: {
        color: "#c9d1d9",
      },
    },
    {
      types: [
        "property",
        "tag",
        "boolean",
        "number",
        "constant",
        "symbol",
        "deleted",
      ],
      style: {
        color: "#79c0ff",
      },
    },
    {
      types: ["selector", "attr-name", "string", "char", "builtin", "inserted"],
      style: {
        color: "#a5d6ff",
      },
    },
    {
      types: ["operator", "entity", "url"],
      style: {
        color: "#a5d6ff",
        background: "#161b22",
      },
    },
    {
      types: ["atrule", "attr-value", "keyword"],
      style: {
        color: "#03DAC5",
      },
    },
    {
      types: ["function"],
      style: {
        color: "#d2a8ff",
      },
    },
    {
      types: ["regex", "important", "variable"],
      style: {
        color: "#a8daff",
      },
    },
    {
      types: ["important", "bold"],
      style: {
        fontWeight: "bold",
      },
    },
    {
      types: ["italic"],
      style: {
        fontStyle: "italic",
      },
    },
    {
      types: ["entity"],
      style: {
        cursor: "help",
      },
    },
  ],
};
