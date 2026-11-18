import { defineConfig, type DefaultTheme } from 'vitepress'

// Replace OWNER/REPO with the real GitHub repository.
const REPO = 'https://github.com/soteenstudio/fenneth'

type Labels = {
  guide: string
  introduction: string
  installation: string
  cli: string
  examples: string
  language: string
  basics: string
  controlFlow: string
  functions: string
  data: string
  keywords: string
  builtins: string
  stdlib: string
  modulesFfi: string
  limitations: string
  project: string
  architecture: string
  releasing: string
  changelog: string
}

// `prefix` is '' for English (site root) and '/id' for Indonesian.
function sidebar(prefix: string, t: Labels): DefaultTheme.Sidebar {
  return [
    {
      text: t.guide,
      items: [
        { text: t.introduction, link: `${prefix}/guide/introduction` },
        { text: t.installation, link: `${prefix}/guide/installation` },
        { text: t.cli, link: `${prefix}/guide/cli` },
        { text: t.examples, link: `${prefix}/guide/examples` },
      ],
    },
    {
      text: t.language,
      items: [
        { text: t.basics, link: `${prefix}/language/basics` },
        { text: t.controlFlow, link: `${prefix}/language/control-flow` },
        { text: t.functions, link: `${prefix}/language/functions` },
        { text: t.data, link: `${prefix}/language/data` },
        { text: t.keywords, link: `${prefix}/language/keywords` },
        { text: t.builtins, link: `${prefix}/language/builtins` },
        { text: t.stdlib, link: `${prefix}/language/stdlib` },
        { text: t.modulesFfi, link: `${prefix}/language/modules-ffi` },
        { text: t.limitations, link: `${prefix}/language/limitations` },
      ],
    },
    {
      text: t.project,
      items: [
        { text: t.architecture, link: `${prefix}/project/architecture` },
        { text: t.releasing, link: `${prefix}/project/releasing` },
        { text: t.changelog, link: `${prefix}/project/changelog` },
      ],
    },
  ]
}

const en: Labels = {
  guide: 'Guide',
  introduction: 'Introduction',
  installation: 'Installation',
  cli: 'Command line',
  examples: 'Code examples',
  language: 'Language',
  basics: 'Basics',
  controlFlow: 'Control flow',
  functions: 'Functions',
  data: 'Structs, arrays and strings',
  keywords: 'Keywords',
  builtins: 'Built-in functions',
  stdlib: 'Standard library',
  modulesFfi: 'Modules and FFI',
  limitations: 'Limitations and runtime checks',
  project: 'Project',
  architecture: 'Architecture',
  releasing: 'Releases and docs hosting',
  changelog: 'Changelog',
}

const id: Labels = {
  guide: 'Panduan',
  introduction: 'Pengantar',
  installation: 'Instalasi',
  cli: 'Baris perintah',
  examples: 'Contoh kode',
  language: 'Bahasa',
  basics: 'Dasar-dasar',
  controlFlow: 'Alur kontrol',
  functions: 'Fungsi',
  data: 'Struct, array, dan string',
  keywords: 'Keyword',
  builtins: 'Fungsi bawaan',
  stdlib: 'Pustaka standar',
  modulesFfi: 'Modul dan FFI',
  limitations: 'Batasan dan pemeriksaan runtime',
  project: 'Proyek',
  architecture: 'Arsitektur',
  releasing: 'Rilis dan hosting docs',
  changelog: 'Changelog',
}

export default defineConfig({
  title: 'Fenneth',
  cleanUrls: true,
  lastUpdated: true,
  head: [['link', { rel: 'icon', href: '/favicon.svg' }]],

  // Fenneth has no Shiki grammar; Rust highlighting fits the syntax well.
  markdown: {
    languageAlias: { fenneth: 'rust', fen: 'rust' },
  },

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      description: 'A small statically typed language that compiles to C.',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/guide/introduction', activeMatch: '/guide/' },
          { text: 'Language', link: '/language/basics', activeMatch: '/language/' },
          { text: 'Changelog', link: '/project/changelog' },
        ],
        sidebar: sidebar('', en),
        editLink: {
          pattern: `${REPO}/edit/main/docs/:path`,
          text: 'Edit this page on GitHub',
        },
      },
    },
    id: {
      label: 'Indonesia',
      lang: 'id',
      link: '/id/',
      description: 'Bahasa pemrograman bertipe statis kecil yang dikompilasi ke C.',
      themeConfig: {
        nav: [
          { text: 'Panduan', link: '/id/guide/introduction', activeMatch: '/id/guide/' },
          { text: 'Bahasa', link: '/id/language/basics', activeMatch: '/id/language/' },
          { text: 'Changelog', link: '/id/project/changelog' },
        ],
        sidebar: sidebar('/id', id),
        editLink: {
          pattern: `${REPO}/edit/main/docs/:path`,
          text: 'Edit halaman ini di GitHub',
        },
        docFooter: { prev: 'Sebelumnya', next: 'Berikutnya' },
        outline: { label: 'Di halaman ini' },
        lastUpdated: { text: 'Terakhir diperbarui' },
        langMenuLabel: 'Ganti bahasa',
        returnToTopLabel: 'Kembali ke atas',
        sidebarMenuLabel: 'Menu',
        darkModeSwitchLabel: 'Tema',
        lightModeSwitchTitle: 'Beralih ke tema terang',
        darkModeSwitchTitle: 'Beralih ke tema gelap',
      },
    },
  },

  themeConfig: {
    socialLinks: [{ icon: 'github', link: REPO }],
    search: {
      provider: 'local',
      options: {
        locales: {
          id: {
            translations: {
              button: { buttonText: 'Cari', buttonAriaLabel: 'Cari' },
              modal: {
                displayDetails: 'Tampilkan detail',
                resetButtonTitle: 'Hapus pencarian',
                backButtonTitle: 'Tutup pencarian',
                noResultsText: 'Tidak ada hasil untuk',
                footer: {
                  selectText: 'pilih',
                  navigateText: 'pindah',
                  closeText: 'tutup',
                },
              },
            },
          },
        },
      },
    },
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Fenneth',
    },
  },
})
