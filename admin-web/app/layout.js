import { Outfit, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const outfit = Outfit({
  variable: "--font-main",
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700", "800"],
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["400", "600", "700"],
});

export const metadata = {
  title: "NHK License Portal | Admin Management",
  description: "Next.js Security-First Admin Management for NodeHotKeyV4 Serial Keys & Licenses",
};

export default function RootLayout({ children }) {
  return (
    <html lang="th" className={`${outfit.variable} ${jetbrainsMono.variable} h-full`}>
      <body className="min-h-full flex flex-col bg-[#080c14] text-[#f1f5f9]">
        {children}
      </body>
    </html>
  );
}
