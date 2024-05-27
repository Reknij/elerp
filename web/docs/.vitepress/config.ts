import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  head: [
    ['link',
      { rel: 'icon', href: '/elerp_logo.svg' }]
  ],
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    siteTitle: 'Elerp',
    logo: '/elerp_logo.svg',
    search: {
      provider: 'local'
    }
  },

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      title: "Elerp | Inventory & Warehouse Management System",
      description: "Easily manage inventory and orders across multiple warehouses.",
      themeConfig: {
        nav: [
          { text: 'Tutorial', link: '/tutorial/quick-start' },
          { text: 'Shopee', link: 'https://shopee.com.my/product/37589189/24765018827/' }
        ],
        sidebar: [
          {
            text: 'Basic',
            items: [
              {
                text: "Quick Start",
                link: '/tutorial/quick-start'
              },
              {
                text: "Add Order",
                link: '/tutorial/add-order'
              },
              {
                text: "Check Inventory",
                link: '/tutorial/check-inventory'
              },
              {
                text: 'Admin & General User',
                link: '/tutorial/user'
              },
              {
                text: 'Personal Configuration',
                link: '/tutorial/personal-configuration'
              }
            ]
          }
        ]
      }
    },
    ms: {
      label: 'Bahasa Melayu',
      lang: 'ms',
      title: "Elerp | Sistem Pengurusan Inventori & Gudang",
      description: "Urus inventori dan pesanan dengan mudah merentas berbilang gudang.",
      themeConfig: {
        nav: [
          { text: 'Tutorial', link: '/ms/tutorial/quick-start' },
          { text: 'Shopee', link: 'https://shopee.com.my/product/37589189/24765018827/' }
        ],
        sidebar: [
          {
            text: 'Asas',
            items: [
              {
                text: "Permulaan Pantas",
                link: '/ms/tutorial/quick-start'
              },
              {
                text: "Tambah Pesanan",
                link: '/ms/tutorial/add-order'
              },
              {
                text: "Semak Inventori",
                link: '/ms/tutorial/check-inventory'
              },
              {
                text: 'Pentadbir & Pengguna Umum',
                link: '/ms/tutorial/user'
              },
              {
                text: 'Konfigurasi Peribadi',
                link: '/ms/tutorial/personal-configuration'
              }
            ]
          }
        ]
      }
    },
    zh: {
      label: "简体中文",
      lang: 'zh',
      title: "Elerp | 库存 & 仓库管理系统",
      description: "轻松管理多个仓库中的库存和订单。",
      themeConfig: {
        nav: [
          { text: '教程', link: '/zh/tutorial/quick-start' },
          { text: 'Shopee', link: 'https://shopee.com.my/product/37589189/24765018827/' }
        ],
        sidebar: [
          {
            text: '基础',
            items: [
              {
                text: "快速开始",
                link: '/zh/tutorial/quick-start'
              },
              {
                text: "添加订单",
                link: '/zh/tutorial/add-order'
              },
              {
                text: "检查库存",
                link: '/zh/tutorial/check-inventory'
              },
              {
                text: '管理员和普通用户',
                link: '/zh/tutorial/user'
              },
              {
                text: '个人配置',
                link: '/zh/tutorial/personal-configuration'
              }
            ]
          }
        ]
      }
    }
  }
})
