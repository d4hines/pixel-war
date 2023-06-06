import './globals.css'
import { Inter } from 'next/font/google'

const inter = Inter({ subsets: ['latin'] })

export const metadata = {
  title: 'Tezos Pixel War',
  description: 'A real-time collaborative pixel-art canvas. Inspired by r/place. Powered by Tezos Smart Rollups.',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <script defer data-domain="pixelwar.tez.page" src="https://plausible.io/js/script.js"></script>
      <body className={inter.className}>{children}</body>
    </html>
  )
}
