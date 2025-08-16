import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom'
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare'
import { BackpackWalletAdapter } from '@solana/wallet-adapter-backpack'

// Default styles that can be overridden by your app
import '@solana/wallet-adapter-react-ui/styles.css'

const endpoint = import.meta.env.VITE_SOLANA_RPC_ENDPOINT || 'https://api.devnet.solana.com'
const wallets = [
	new PhantomWalletAdapter(),
	new SolflareWalletAdapter({ network: 'devnet' }),
	new BackpackWalletAdapter(),
]

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<ConnectionProvider endpoint={endpoint}>
			<WalletProvider wallets={wallets} autoConnect>
				<WalletModalProvider>
					<App />
				</WalletModalProvider>
			</WalletProvider>
		</ConnectionProvider>
	</StrictMode>,
)
